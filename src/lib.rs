pub mod board;
pub mod zobrist;
pub mod utils;

// Re-export move utilities for the perft binary
pub use board::{
    move_from_sq, move_to_sq, move_piece, move_flag,
    PieceKind, Color, Board
};

pub fn init() {
    utils::init_attack_tables();
    zobrist::init_zobrist();
}