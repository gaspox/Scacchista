pub mod board;
pub mod search;
pub mod utils;
pub mod zobrist; // Scacchista search module (was MyRustChessEngine search)

// Re-export move utilities for the perft binary
pub use board::{
    move_captured, move_flag, move_from_sq, move_piece, move_to_sq, Board, Color, PieceKind,
    FLAG_PROMOTION,
};

pub fn init() {
    utils::init_attack_tables();
    zobrist::init_zobrist();
}
