//! Search parameters and configuration
//!
//! Controls search behavior including time limits, depth limits,
//! and optimization thresholds.

/// Search parameters for the engine
#[derive(Debug, Clone)]
pub struct SearchParams {
    /// Maximum search depth in plies
    pub max_depth: u8,

    /// Time limit in milliseconds (0 = unlimited)
    pub time_limit_ms: u64,

    /// Node limit (0 = unlimited)
    pub node_limit: u64,

    /// Alpha-beta aspiration window size in centipawns
    pub aspiration_window: i16,

    /// Enable null-move pruning
    pub enable_null_move_pruning: bool,

    /// Minimum depth for null-move pruning
    pub null_move_min_depth: u8,

    /// Enable late move reduction
    pub enable_lmr: bool,

    /// LMR reduction table size and parameters
    pub lmr_min_depth: u8,
    pub lmr_base_reduction: u8,

    /// Enable futility pruning
    pub enable_futility_pruning: bool,

    /// Futility margin for pruning
    pub futility_margin: i16,

    /// Number of killer move slots
    pub killer_moves_count: usize,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            max_depth: 8,
            time_limit_ms: 5000,
            node_limit: 0,
            aspiration_window: 50, // 0.5 pawn
            enable_null_move_pruning: true,
            null_move_min_depth: 2,
            enable_lmr: true,
            lmr_min_depth: 3,
            lmr_base_reduction: 2,
            enable_futility_pruning: true,
            futility_margin: 100, // 1.0 pawn
            killer_moves_count: 2,
        }
    }
}

impl SearchParams {
    /// Create new search params with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum depth in plies
    pub fn max_depth(mut self, depth: u8) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set time limit in milliseconds
    pub fn time_limit(mut self, ms: u64) -> Self {
        self.time_limit_ms = ms;
        self
    }

    /// Set aspiration window size in centipawns
    pub fn aspiration_window(mut self, window: i16) -> Self {
        self.aspiration_window = window;
        self
    }

    /// Set node limit
    pub fn node_limit(mut self, limit: u64) -> Self {
        self.node_limit = limit;
        self
    }
}

/// Search time management parameters
#[derive(Debug, Clone)]
pub struct TimeManagement {
    pub time_left_ms: u64,
    pub moves_left: u8,
    pub moves_to_go_left: u8,
    pub inc_ms: u64,
    pub msec_per_move: u64,
}

impl TimeManagement {
    pub fn new() -> Self {
        Self {
            time_left_ms: 0,
            moves_left: 40,
            moves_to_go_left: 40,
            inc_ms: 0,
            msec_per_move: 5000, // Default 5 seconds per move
        }
    }

    /// Calculate time to allocate for current move
    pub fn allocate_time(&self) -> u64 {
        if self.time_left_ms == 0 {
            return self.msec_per_move;
        }

        // Simple time allocation: time_left / moves_to_go * 1.2
        let allocated = (self.time_left_ms / self.moves_to_go_left as u64) * 120 / 100;

        allocated.min(self.msec_per_move.max(self.inc_ms * 2))
    }
}
