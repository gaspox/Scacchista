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

    /// Store an entry using a simple replacement policy
    pub fn store(&mut self, key: u64, score: i16, depth: u8, node_type: NodeType, best_move: Move) {
        let index = (key & self.mask) as usize;
        let existing = &self.entries[index];

        let replace = existing.is_empty()
            || depth > existing.depth
            || (node_type == NodeType::Exact && existing.node_type != NodeType::Exact)
            || (existing.age != self.age);

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
