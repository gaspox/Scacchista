#![allow(
    clippy::all,
    dead_code,
    unused_variables,
    unused_imports,
    unused_comparisons,
    non_camel_case_types
)]
#![allow(unused_parens, unused_mut)]

// Crate root for Scacchista

pub mod board;
pub mod eval;
pub mod search;
pub mod time;
pub mod uci;
pub mod utils;
pub mod zobrist; // Scacchista search module (was MyRustChessEngine search)

// Re-export move utilities for the perft binary
pub use board::{
    move_captured, move_flag, move_from_sq, move_piece, move_to_sq, move_to_uci, parse_uci_move,
    Board, Color, PieceKind, FLAG_PROMOTION,
};

pub fn init() {
    utils::init_attack_tables();
    zobrist::init_zobrist();
}
