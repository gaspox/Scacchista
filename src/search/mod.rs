//! Search engine for Scacchista chess engine
//!
//!
//!

pub mod params;
// The module name matches its parent directory (`search/search.rs`), which is
// a standard Rust pattern for the primary module file in a directory.
#[allow(clippy::module_inception)]
pub mod search;
pub mod stats;
pub mod thread_mgr;
pub mod tt;

pub use self::params::SearchParams;
pub use self::search::Search;
pub use self::stats::SearchStats;
pub use self::thread_mgr::ThreadManager;
pub use self::tt::TranspositionTable;
pub use crate::board::Move;

/// Result of a completed search job, including move, score, PV and stats.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Move,
    pub score: i16,
    pub completed_depth: u8,
    pub pv: Vec<Move>,
    pub nodes: u64,
    pub nps: u64,
    pub seldepth: u8,
    pub hashfull: u8,
}
