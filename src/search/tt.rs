//! Transposition Table for Scacchista
//!
//! Simple fixed-size transposition table using a single entry per index

use crate::board::Move;

/// Node type for transposition table entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Exact = 0,
    LowerBound = 1,
    UpperBound = 2,
}

/// Single TT entry (compact)
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

/// Transposition table: single-probe direct-mapped table
pub struct TranspositionTable {
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
        Self {
            entries: vec![TTEntry::empty(); final_entries],
            mask,
            age: 0,
        }
    }

    /// Probe returns a reference to an entry if the stored key matches and entry is recent enough
    pub fn probe(&self, key: u64) -> Option<&TTEntry> {
        let index = (key & self.mask) as usize;
        let e = &self.entries[index];
        if e.key == key && self.age.wrapping_sub(e.age) < 8 {
            Some(e)
        } else {
            None
        }
    }

    /// Store an entry using an improved replacement policy
    ///
    /// Replacement priorities:
    /// 1. Empty slots: always replace
    /// 2. Old entries: replace if age difference >= 2 (more aggressive aging)
    /// 3. Exact scores: replace at same depth if new entry is Exact
    /// 4. Deeper searches: always replace shallow with deeper
    pub fn store(&mut self, key: u64, score: i16, depth: u8, node_type: NodeType, best_move: Move) {
        let index = (key & self.mask) as usize;
        let existing = &self.entries[index];

        // Improved replacement scheme: balance freshness, depth, and node quality
        let replace = existing.is_empty()
            // More aggressive aging: replace if entry is 2+ searches old
            || (self.age != existing.age && self.age.wrapping_sub(existing.age) >= 2)
            // Prefer exact scores even at same depth
            || (depth >= existing.depth && node_type == NodeType::Exact)
            // Always replace with deeper search
            || depth > existing.depth;

        if replace {
            self.entries[index] = TTEntry::new(key, score, depth, node_type, best_move, self.age);
        }
    }

    /// Increment search age (call at start of each new root search)
    pub fn new_search(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    pub fn fill_percentage(&self) -> f64 {
        let filled = self.entries.iter().filter(|e| !e.is_empty()).count();
        (filled as f64 / self.entries.len() as f64) * 100.0
    }

    pub fn clear(&mut self) {
        for e in &mut self.entries {
            *e = TTEntry::empty();
        }
        self.age = 0;
    }

    pub fn size(&self) -> usize {
        self.entries.len()
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
        // Test that the age wraparound (255 -> 0) is handled correctly
        // in the replacement policy
        let mut tt = TranspositionTable::new(1); // Small TT for testing

        // Set age to 254
        tt.age = 254;

        // Store an entry at age 254
        tt.store(0x1234, 100, 5, NodeType::Exact, 0x1111);

        // Verify entry was stored
        let entry = tt.probe(0x1234).expect("Entry should exist");
        assert_eq!(entry.age, 254);
        assert_eq!(entry.depth, 5);

        // Advance to age 255
        tt.new_search();
        assert_eq!(tt.age, 255);

        // Try to store with same depth and Exact - WILL replace due to Exact priority
        // (even though age diff = 1, the condition "depth >= existing.depth && node_type == Exact" triggers)
        tt.store(0x1234, 200, 5, NodeType::Exact, 0x2222);
        let entry = tt.probe(0x1234).expect("Entry should exist");
        assert_eq!(
            entry.age, 255,
            "Exact node replaces at same depth regardless of age"
        );
        assert_eq!(entry.score, 200, "Score should be updated");

        // Advance again (wraps to 0)
        tt.new_search();
        assert_eq!(tt.age, 0);

        // Now age diff is 0 - 255 = wrapping_sub = 1, still < 2
        // Store a NON-exact entry with shallow depth - should NOT replace
        tt.store(0x1234, 250, 3, NodeType::UpperBound, 0x2233);
        let entry = tt.probe(0x1234).expect("Entry should exist");
        assert_eq!(
            entry.age, 255,
            "Should not replace with shallow UpperBound and age diff = 1"
        );
        assert_eq!(entry.score, 200, "Score unchanged");

        // Advance again (age = 1)
        tt.new_search();
        assert_eq!(tt.age, 1);

        // Now age diff is 1 - 255 = wrapping_sub = 2, >= 2!
        // Should replace even with same depth
        tt.store(0x1234, 300, 5, NodeType::Exact, 0x3333);
        let entry = tt.probe(0x1234).expect("Entry should exist");
        assert_eq!(
            entry.age, 1,
            "Should replace with age diff >= 2 after wraparound"
        );
        assert_eq!(entry.score, 300, "Score should be updated");
    }

    #[test]
    fn test_tt_replacement_depth_priority() {
        // Test that deep entries are not replaced by shallow searches
        let mut tt = TranspositionTable::new(1);

        // Store a deep entry (depth 10)
        tt.store(0x5678, 100, 10, NodeType::Exact, 0x1111);

        // Verify stored
        let entry = tt.probe(0x5678).expect("Entry should exist");
        assert_eq!(entry.depth, 10);
        assert_eq!(entry.score, 100);

        // Try to replace with shallow search (depth 5) at same age
        tt.store(0x5678, 200, 5, NodeType::Exact, 0x2222);

        // Should NOT replace (depth 5 < depth 10)
        let entry = tt.probe(0x5678).expect("Entry should exist");
        assert_eq!(
            entry.depth, 10,
            "Deep entry should not be replaced by shallow"
        );
        assert_eq!(entry.score, 100, "Score should be unchanged");

        // Try to replace with same depth (depth 10) and Exact node type
        tt.store(0x5678, 300, 10, NodeType::Exact, 0x3333);

        // SHOULD replace (depth >= existing.depth && node_type == Exact)
        let entry = tt.probe(0x5678).expect("Entry should exist");
        assert_eq!(entry.depth, 10);
        assert_eq!(
            entry.score, 300,
            "Should replace at same depth with Exact node"
        );

        // Try to replace with deeper search (depth 15)
        tt.store(0x5678, 400, 15, NodeType::UpperBound, 0x4444);

        // SHOULD replace (depth > existing.depth)
        let entry = tt.probe(0x5678).expect("Entry should exist");
        assert_eq!(entry.depth, 15, "Should replace with deeper search");
        assert_eq!(entry.score, 400);
    }

    #[test]
    fn test_tt_replacement_exact_node_priority() {
        // Test that Exact nodes are preferred over bounds
        let mut tt = TranspositionTable::new(1);

        // Store an UpperBound entry at depth 5
        tt.store(0xABCD, 100, 5, NodeType::UpperBound, 0x1111);

        let entry = tt.probe(0xABCD).expect("Entry should exist");
        assert_eq!(entry.node_type, NodeType::UpperBound);
        assert_eq!(entry.depth, 5);

        // Try to replace with Exact node at SAME depth
        tt.store(0xABCD, 200, 5, NodeType::Exact, 0x2222);

        // SHOULD replace (Exact is preferred)
        let entry = tt.probe(0xABCD).expect("Entry should exist");
        assert_eq!(
            entry.node_type,
            NodeType::Exact,
            "Exact should replace bound at same depth"
        );
        assert_eq!(entry.score, 200);

        // Try to replace Exact with LowerBound at same depth
        tt.store(0xABCD, 300, 5, NodeType::LowerBound, 0x3333);

        // Should NOT replace (Exact is only replaced by depth or Exact)
        let entry = tt.probe(0xABCD).expect("Entry should exist");
        assert_eq!(
            entry.node_type,
            NodeType::Exact,
            "Exact should not be replaced by bound"
        );
        assert_eq!(entry.score, 200, "Score unchanged");
    }

    #[test]
    fn test_tt_basic_store_probe() {
        // Basic sanity test
        let mut tt = TranspositionTable::new(16);

        tt.store(0x1111, 42, 3, NodeType::Exact, 0xAAAA);

        let entry = tt.probe(0x1111).expect("Entry should exist");
        assert_eq!(entry.score, 42);
        assert_eq!(entry.depth, 3);
        assert_eq!(entry.node_type, NodeType::Exact);
        assert_eq!(entry.best_move, 0xAAAA);

        // Probe non-existent
        assert!(tt.probe(0x9999).is_none());
    }
}
