// Zobrist hashing with precomputed tables
use crate::board::{Board, Color, PieceKind};

// Global tables (initialized once via init_zobrist)
pub static mut ZOB_PIECE: [[u64; 64]; 12] = [[0; 64]; 12];
pub static mut ZOB_SIDE: u64 = 0;
pub static mut ZOB_CASTLING: [u64; 16] = [0; 16];
pub static mut ZOB_EP_FILE: [u64; 8] = [0; 8];
static mut INITIALIZED: bool = false;

fn split_mix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
pub fn init_zobrist() {
    unsafe {
        if INITIALIZED { return; }
        // Initialize piece-square keys
        for i in 0..12 {
            for j in 0..64 {
                ZOB_PIECE[i][j] = split_mix64((i as u64).wrapping_mul(0xad3) + (j as u64).wrapping_mul(0x47a1));
            }
        }
        // Side to move key
        ZOB_SIDE = split_mix64(0xdeadbeefdeadbeef);
        // Castling rights keys: index 0..15 (WK, WQ, BK, BQ as bits)
        for i in 0..16 {
            ZOB_CASTLING[i] = split_mix64((i as u64).wrapping_mul(0x1234_abcd));
        }
        // Ep file: 0 for no ep, or 1..7 for file A..H squares
        for i in 0..8 {
            ZOB_EP_FILE[i] = split_mix64((i as u64).wrapping_mul(0x3333_5555));
        }
        INITIALIZED = true;
    }
}
fn piece_index(kind: PieceKind, color: Color) -> usize {
    (color as usize) * 6 + (kind as usize)
}
pub fn recalc_zobrist_full(board: &Board) -> u64 {
    unsafe {
        if !INITIALIZED { init_zobrist(); }
        let mut h = 0u64;
        // Piece contribution
        for kind_index in 0..6 {
            let kind = match kind_index {
                0 => PieceKind::Pawn,
                1 => PieceKind::Knight,
                2 => PieceKind::Bishop,
                3 => PieceKind::Rook,
                4 => PieceKind::Queen,
                5 => PieceKind::King,
                _ => unreachable!(),
            };
            for color in [Color::White, Color::Black] {
                let bb = board.piece_bb(kind, color);
                let mut bb = bb;
                while let Some(sq) = crate::utils::pop_lsb(&mut bb) {
                    h ^= ZOB_PIECE[piece_index(kind, color)][sq];
                }
            }
        }
        // Side to move
        if board.side == Color::Black { h ^= ZOB_SIDE; }
        // Castling rights (bits mapping as in board struct)
        h ^= ZOB_CASTLING[board.castling as usize];
        // En-passant file (0 if None)
        if let Some(ep_sq) = board.ep {
            let file = ep_sq % 8;
            h ^= ZOB_EP_FILE[file as usize];
        }
        h
    }
}