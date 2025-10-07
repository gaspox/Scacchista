//! Search statistics and performance metrics
//!
//! Tracks search performance including nodes searched, cutoffs,
//! hash table hits, and timing information.

use std::time::{Duration, Instant};

/// Search statistics
#[derive(Debug, Default, Clone)]
pub struct SearchStats {
    /// Total nodes searched
    pub nodes: u64,

    /// Nodes at root ply
    pub root_nodes: u64,

    /// Quiescence nodes searched
    pub qsearch_nodes: u64,

    /// Transposition table hits
    pub tt_hits: u64,

    /// Transposition table entries used
    pub tt_entries: u64,

    /// Alpha-beta cutoffs
    pub cutoffs: u64,

    /// Null-move cutoffs
    pub null_move_cutoffs: u64,

    /// Late move reductions
    pub lmr_reductions: u64,

    /// Futility pruned nodes
    pub futility_pruned: u64,

    /// SEE evaluations performed
    pub see_evals: u64,

    /// Search start time
    pub start_time: Option<Instant>,

    /// Current time
    pub current_time: Option<Instant>,

    /// Time spent searching
    pub search_time: Duration,

    /// Nodes per second rate
    pub nps: u64,
}

impl SearchStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Start timing
    pub fn start_timing(&mut self) {
        self.start_time = Some(Instant::now());
        self.current_time = self.start_time;
    }

    /// Update current time and calculate NPS
    pub fn update_timing(&mut self) {
        let now = Instant::now();
        self.current_time = Some(now);

        if let Some(start) = self.start_time {
            self.search_time = now.duration_since(start);
            let elapsed_ms = self.search_time.as_millis() as u64;
            if elapsed_ms > 0 {
                self.nps = (self.nodes * 1000) / elapsed_ms;
            }
        }
    }

    /// Increment node count
    pub fn inc_node(&mut self) {
        self.nodes += 1;
    }

    /// Increment root node count
    pub fn inc_root_node(&mut self) {
        self.root_nodes += 1;
    }

    /// Increment quiescence node count
    pub fn inc_qsearch_node(&mut self) {
        self.qsearch_nodes += 1;
    }

    /// Increment TT hit count
    pub fn inc_tt_hit(&mut self) {
        self.tt_hits += 1;
    }

    /// Increment TT entry used count
    pub fn inc_tt_entry(&mut self) {
        self.tt_entries += 1;
    }

    /// Increment cutoff count
    pub fn inc_cutoff(&mut self) {
        self.cutoffs += 1;
    }

    /// Increment null-move cutoff count
    pub fn inc_null_move_cutoff(&mut self) {
        self.null_move_cutoffs += 1;
    }

    /// Increment LMR reduction count
    pub fn inc_lmr_reduction(&mut self) {
        self.lmr_reductions += 1;
    }

    /// Increment futility pruning count
    pub fn inc_futility_pruned(&mut self) {
        self.futility_pruned += 1;
    }

    /// Increment SEE evaluation count
    pub fn inc_see_eval(&mut self) {
        self.see_evals += 1;
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Print formatted summary
    pub fn print_summary(&self) {
        println!("=== Search Statistics ===");
        println!("Nodes searched: {}", self.nodes);
        println!("Root nodes: {}", self.root_nodes);
        println!("QSearch nodes: {}", self.qsearch_nodes);
        println!(
            "TT hits: {} ({:.1}%)",
            self.tt_hits,
            if self.tt_entries > 0 {
                (self.tt_hits as f64 / self.tt_entries as f64) * 100.0
            } else {
                0.0
            }
        );
        println!("Alpha-Beta cutoffs: {}", self.cutoffs);
        println!("Null-move cutoffs: {}", self.null_move_cutoffs);
        println!("LMR reductions: {}", self.lmr_reductions);
        println!("Futility pruned: {}", self.futility_pruned);
        println!("SEE evaluations: {}", self.see_evals);
        println!("Search time: {} ms", self.search_time.as_millis());
        println!("Nodes per second: {}", self.nps);

        let avg_depth = if self.root_nodes > 0 {
            self.nodes as f64 / self.root_nodes as f64
        } else {
            0.0
        };
        println!("Average depth: {:.2}", avg_depth);
    }
}
