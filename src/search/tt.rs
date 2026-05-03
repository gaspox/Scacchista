//! Lock-free Transposition Table for Scacchista
//!
//! Each bucket holds a single entry composed of three `AtomicU64` fields:
//! `key`, packed `data` (score/depth/age/node_type), and `best_move`.
//! Writes are performed with `Ordering::Relaxed`; readers may observe
//! slightly stale data, which is harmless for a lossy cache.

use crate::board::Move;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

/// Node type for transposition table entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Exact = 0,
    LowerBound = 1,
    UpperBound = 2,
}

/// Single TT entry returned to callers.
#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    pub key: u64,
    pub score: i16,
    pub depth: u8,
    pub node_type: NodeType,
    pub best_move: Move,
    pub age: u8,
}

impl TTEntry {
    pub fn empty() -> Self {
        Self {
            key: 0,
            score: 0,
            depth: 0,
            node_type: NodeType::Exact,
            best_move: 0,
            age: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.key == 0
    }

    pub fn new(
        key: u64,
        score: i16,
        depth: u8,
        node_type: NodeType,
        best_move: Move,
        age: u8,
    ) -> Self {
        Self {
            key,
            score,
            depth,
            node_type,
            best_move,
            age,
        }
    }

    /// Return bounds for alpha-beta usage
    pub fn bound(&self) -> (i16, i16) {
        match self.node_type {
            NodeType::Exact => (self.score, self.score),
            NodeType::LowerBound => (self.score, i16::MAX),
            NodeType::UpperBound => (i16::MIN, self.score),
        }
    }
}

// ---------------------------------------------------------------------------
// Packed data layout (single u64)
//   bits 0-15  : score (i16 cast to u16)
//   bits 16-23 : depth
//   bits 24-31 : age
//   bits 32-33 : node_type (0=Exact, 1=LowerBound, 2=UpperBound)
// ---------------------------------------------------------------------------

#[inline]
fn pack_data(score: i16, depth: u8, age: u8, node_type: NodeType) -> u64 {
    ((score as u16) as u64)
        | ((depth as u64) << 16)
        | ((age as u64) << 24)
        | ((node_type as u64) << 32)
}

#[inline]
fn unpack_data(data: u64) -> (i16, u8, u8, NodeType) {
    let score = (data & 0xFFFF) as u16 as i16;
    let depth = ((data >> 16) & 0xFF) as u8;
    let age = ((data >> 24) & 0xFF) as u8;
    let node_type = match (data >> 32) & 0x3 {
        0 => NodeType::Exact,
        1 => NodeType::LowerBound,
        2 => NodeType::UpperBound,
        _ => NodeType::Exact,
    };
    (score, depth, age, node_type)
}

#[repr(C)]
struct AtomicTTEntry {
    key: AtomicU64,
    data: AtomicU64,
    best_move: AtomicU64,
}

impl AtomicTTEntry {
    fn empty() -> Self {
        Self {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0),
            best_move: AtomicU64::new(0),
        }
    }
}

/// Lock-free transposition table.
pub struct TranspositionTable {
    entries: Vec<AtomicTTEntry>,
    mask: usize,
    age: AtomicU8,
}

impl TranspositionTable {
    /// Create a TT with approximately `size_mb` megabytes.
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<AtomicTTEntry>();
        let mut num_entries = (size_mb * 1024 * 1024) / entry_size;
        if num_entries == 0 {
            num_entries = 1024;
        }
        let actual = num_entries.next_power_of_two();
        let final_entries = actual.max(1024);
        let mask = final_entries - 1;

        let entries: Vec<AtomicTTEntry> =
            (0..final_entries).map(|_| AtomicTTEntry::empty()).collect();

        Self {
            entries,
            mask,
            age: AtomicU8::new(0),
        }
    }

    /// Probe returns an entry if the stored key matches and the entry is recent enough.
    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let index = (key as usize) & self.mask;
        let entry = &self.entries[index];
        let k = entry.key.load(Ordering::Relaxed);
        if k == key {
            let table_age = self.age.load(Ordering::Relaxed);
            let data = entry.data.load(Ordering::Relaxed);
            let best_move = entry.best_move.load(Ordering::Relaxed) as Move;
            let (score, depth, entry_age, node_type) = unpack_data(data);
            if table_age.wrapping_sub(entry_age) < 8 {
                return Some(TTEntry {
                    key,
                    score,
                    depth,
                    node_type,
                    best_move,
                    age: entry_age,
                });
            }
        }
        None
    }

    /// Store an entry using replacement policy.
    pub fn store(&self, key: u64, score: i16, depth: u8, node_type: NodeType, best_move: Move) {
        let index = (key as usize) & self.mask;
        let entry = &self.entries[index];
        let current_age = self.age.load(Ordering::Relaxed);

        let existing_key = entry.key.load(Ordering::Relaxed);

        let replace = if existing_key == 0 {
            true
        } else {
            let existing_data = entry.data.load(Ordering::Relaxed);
            let (_, existing_depth, existing_age, _) = unpack_data(existing_data);
            existing_key == 0
                || (current_age != existing_age && current_age.wrapping_sub(existing_age) >= 2)
                || (depth >= existing_depth && node_type == NodeType::Exact)
                || depth > existing_depth
        };

        if replace {
            let packed = pack_data(score, depth, current_age, node_type);
            entry.data.store(packed, Ordering::Relaxed);
            entry.best_move.store(best_move as u64, Ordering::Relaxed);
            entry.key.store(key, Ordering::Relaxed);
        }
    }

    /// Increment search age.
    pub fn new_search(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
    }

    /// Approximate fill percentage.
    pub fn fill_percentage(&self) -> f64 {
        let filled = self
            .entries
            .iter()
            .filter(|e| e.key.load(Ordering::Relaxed) != 0)
            .count();
        (filled as f64 / self.entries.len() as f64) * 100.0
    }

    /// Clear all entries.
    pub fn clear(&self) {
        for entry in &self.entries {
            entry.key.store(0, Ordering::Relaxed);
            entry.data.store(0, Ordering::Relaxed);
            entry.best_move.store(0, Ordering::Relaxed);
        }
        self.age.store(0, Ordering::Relaxed);
    }

    /// Number of entries in the table.
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub fn set_age(&self, age: u8) {
        self.age.store(age, Ordering::Relaxed);
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_tt_replacement_age_wraparound() {
        let tt = TranspositionTable::new(1);

        // Set age to 254
        tt.set_age(254);

        // Store an entry at age 254
        tt.store(0x1234, 100, 5, NodeType::Exact, 0x1111);

        // Verify entry was stored
        let entry = tt.probe(0x1234).expect("Entry should exist");
        assert_eq!(entry.score, 100);

        // Increment age (wraps to 255)
        tt.new_search();

        // Entry should still be valid
        let entry = tt.probe(0x1234);
        assert!(entry.is_some(), "Entry should still be valid");
    }

    #[test]
    fn test_tt_collision_detection() {
        let tt = TranspositionTable::new(1);

        // Two positions with same low bits but different high bits
        let key1 = 0x00001_12345;
        let key2 = 0x00002_12345;

        tt.store(key1, 100, 5, NodeType::Exact, 0x1111);

        // key2 should be detected as collision
        let entry = tt.probe(key2);
        assert!(
            entry.is_none(),
            "Collision should be detected - different full keys"
        );

        // key1 should be found
        let entry = tt.probe(key1).expect("key1 should exist");
        assert_eq!(entry.score, 100);
    }

    #[test]
    fn test_tt_thread_safety() {
        let tt = Arc::new(TranspositionTable::new(16));
        let mut handles = vec![];

        // Spawn multiple threads reading and writing concurrently
        for i in 0..10 {
            let tt_clone = Arc::clone(&tt);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    let key = (i * 100 + j) as u64;
                    tt_clone.store(key, j as i16, 5, NodeType::Exact, 0);
                    let _ = tt_clone.probe(key);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // If we get here without panic, the lock-free table is thread-safe
        assert!(tt.fill_percentage() > 0.0);
    }
}
