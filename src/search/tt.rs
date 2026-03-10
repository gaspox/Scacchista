//! Transposition Table for Scacchista
//!
//! FIX v0.5.1: Tornata a Mutex TT per evitare race condition.
//! L'interfaccia è compatibile con il codice lock-free (senza .lock().unwrap())
//! ma internamente usa Mutex per thread-safety corretta.

use crate::board::Move;
use std::sync::{Arc, Mutex};

/// Node type for transposition table entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Exact = 0,
    LowerBound = 1,
    UpperBound = 2,
}

/// Single TT entry
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

/// Thread-safe Transposition table using Mutex internally
/// FIX v0.5.1: Interfaccia compatibile lock-free, ma usa Mutex per correttezza
pub struct TranspositionTable {
    // FIX: Usa Mutex per thread-safety corretta (evita race condition lock-free)
    inner: Mutex<TranspositionTableInner>,
}

struct TranspositionTableInner {
    entries: Vec<TTEntry>,
    mask: u64,
    age: u8,
}

impl TranspositionTable {
    /// Create a TT with approximately `size_mb` megabytes
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let mut entries = (size_mb * 1024 * 1024) / entry_size;
        if entries == 0 {
            entries = 1024;
        }
        let actual = entries.next_power_of_two();
        let final_entries = actual.max(1024);
        let mask = (final_entries - 1) as u64;

        let entries: Vec<TTEntry> = (0..final_entries)
            .map(|_| TTEntry::empty())
            .collect();

        Self {
            inner: Mutex::new(TranspositionTableInner {
                entries,
                mask,
                age: 0,
            }),
        }
    }

    /// Probe returns an entry if the stored key matches and entry is recent enough
    /// FIX v0.5.1: Usa Mutex per garantire consistenza
    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let inner = self.inner.lock().unwrap();
        let index = (key & inner.mask) as usize;
        let e = &inner.entries[index];
        
        if e.key == key && inner.age.wrapping_sub(e.age) < 8 {
            Some(*e)  // Ritorna copia, non riferimento
        } else {
            None
        }
    }

    /// Store an entry using replacement policy
    /// FIX v0.5.1: Usa Mutex per atomicità
    pub fn store(&self, key: u64, score: i16, depth: u8, node_type: NodeType, best_move: Move) {
        let mut inner = self.inner.lock().unwrap();
        let index = (key & inner.mask) as usize;
        let current_age = inner.age;
        let existing = &inner.entries[index];

        // Replacement policy
        let replace = if existing.is_empty() {
            true
        } else {
            existing.is_empty()
                || (current_age != existing.age && current_age.wrapping_sub(existing.age) >= 2)
                || (depth >= existing.depth && node_type == NodeType::Exact)
                || depth > existing.depth
        };

        if replace {
            inner.entries[index] = TTEntry::new(key, score, depth, node_type, best_move, current_age);
        }
    }

    /// Increment search age
    pub fn new_search(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.age = inner.age.wrapping_add(1);
    }

    pub fn fill_percentage(&self) -> f64 {
        let inner = self.inner.lock().unwrap();
        let filled = inner.entries.iter().filter(|e| !e.is_empty()).count();
        (filled as f64 / inner.entries.len() as f64) * 100.0
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        for e in &mut inner.entries {
            *e = TTEntry::empty();
        }
        inner.age = 0;
    }

    pub fn size(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.entries.len()
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

    #[test]
    fn test_tt_replacement_age_wraparound() {
        let tt = TranspositionTable::new(1);

        // Set age to 254
        tt.inner.lock().unwrap().age = 254;

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

        // Due posizioni con stessi bit bassi ma diversi bit alti
        let key1 = 0x00001_12345;
        let key2 = 0x00002_12345;

        tt.store(key1, 100, 5, NodeType::Exact, 0x1111);

        // key2 dovrebbe essere rilevata come collisione
        let entry = tt.probe(key2);
        assert!(entry.is_none(), "Collision should be detected - different full keys");

        // key1 dovrebbe essere trovata
        let entry = tt.probe(key1).expect("key1 should exist");
        assert_eq!(entry.score, 100);
    }

    #[test]
    fn test_tt_thread_safety() {
        use std::thread;
        
        let tt = Arc::new(TranspositionTable::new(16));
        let mut handles = vec![];

        // Spawn multiple threads che leggono e scrivono
        for i in 0..10 {
            let tt_clone = Arc::clone(&tt);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    let key = (i * 100 + j) as u64;
                    tt_clone.store(key, j as i16, 5, NodeType::Exact, 0);
                    let _ = tt_clone.probe(key); // Leggi
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Se arriviamo qui senza panico, il Mutex funziona correttamente
        assert!(tt.fill_percentage() > 0.0);
    }
}
