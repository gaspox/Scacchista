//! Search engine for Scacchista chess engine
//!
//! Implements iterative deepening alpha-beta search with transposition table
//! and move ordering optimizations.
//!
//! Main components:
//! - `tt`: Transposition table for caching position evaluations
//! - `params`: Search parameters (time limits, depth limits)
//! - `stats`: Search statistics
//! - `search`: Main search struct and algorithms

pub mod tt;
pub mod params;
pub mod stats;
pub mod search;

pub use crate::board::Move;
pub use self::tt::TranspositionTable;
pub use self::params::SearchParams;
pub use self::stats::SearchStats;
pub use self::search::Search;