//! Search engine for Scacchista chess engine
//!
//!
//!

pub mod params;
pub mod search;
pub mod stats;
pub mod tt;
pub mod thread_mgr;

pub use self::params::SearchParams;
pub use self::search::Search;
pub use self::stats::SearchStats;
pub use self::tt::TranspositionTable;
pub use self::thread_mgr::ThreadManager;
pub use crate::board::Move;
