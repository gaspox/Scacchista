//! Scacchista — A UCI-compliant chess engine written in Rust.
//!
//! This crate provides a complete bitboard-based chess engine with
//! alpha-beta search, transposition tables, and hand-crafted evaluation.

pub mod board;
pub mod eval;
pub mod magic;
pub mod search;
pub mod time;
pub mod uci;
pub mod utils;
pub mod zobrist;

// Re-export move utilities for the perft binary
pub use board::{
    move_captured, move_flag, move_from_sq, move_piece, move_to_sq, move_to_uci, parse_uci_move,
    Board, Color, PieceKind, FLAG_PROMOTION,
};

/// Initialize global lookup tables (attack tables, Zobrist keys, etc.).
///
/// This function is idempotent; calling it multiple times is safe.
pub fn init() {
    utils::init_attack_tables();
    zobrist::init_zobrist();
    magic::init();
}
