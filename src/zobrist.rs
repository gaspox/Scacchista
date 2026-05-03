//! Zobrist hashing with precomputed tables
//!
//! Tables are initialized lazily via [`std::sync::OnceLock`] and are
//! immutable after initialization, making all lookups completely safe.

use std::sync::OnceLock;

use crate::board::{Board, Color, PieceKind};

/// Precomputed Zobrist keys for all board states.
pub struct ZobristTables {
    piece: [[u64; 64]; 12],
    side: u64,
    castling: [u64; 16],
    ep_file: [u64; 8],
}

static ZOBRIST: OnceLock<ZobristTables> = OnceLock::new();

fn split_mix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// Initialize the global Zobrist tables.
///
/// This function is idempotent; subsequent calls are no-ops.
#[allow(clippy::needless_range_loop)]
pub fn init_zobrist() {
    let _ = ZOBRIST.get_or_init(|| {
        let mut piece = [[0u64; 64]; 12];
        for i in 0..12 {
            for j in 0..64 {
                piece[i][j] =
                    split_mix64((i as u64).wrapping_mul(0xad3) + (j as u64).wrapping_mul(0x47a1));
            }
        }

        let side = split_mix64(0xdeadbeefdeadbeef);

        let mut castling = [0u64; 16];
        for i in 0..16 {
            castling[i] = split_mix64((i as u64).wrapping_mul(0x1234_abcd));
        }

        let mut ep_file = [0u64; 8];
        for i in 0..8 {
            ep_file[i] = split_mix64((i as u64).wrapping_mul(0x3333_5555));
        }

        ZobristTables {
            piece,
            side,
            castling,
            ep_file,
        }
    });
}

fn piece_index(kind: PieceKind, color: Color) -> usize {
    (color as usize) * 6 + (kind as usize)
}

fn get() -> &'static ZobristTables {
    init_zobrist();
    ZOBRIST
        .get()
        .expect("Zobrist tables not initialized. Call init_zobrist() first.")
}

/// Return the Zobrist key for a piece on a given square.
#[inline]
pub fn piece_key(kind: PieceKind, color: Color, sq: usize) -> u64 {
    get().piece[piece_index(kind, color)][sq]
}

/// Return the Zobrist key for the side to move (XOR when Black to move).
#[inline]
pub fn side_key() -> u64 {
    get().side
}

/// Return the Zobrist key for the given castling rights (0..15).
#[inline]
pub fn castling_key(rights: usize) -> u64 {
    get().castling[rights]
}

/// Return the Zobrist key for an en-passant file (0..7).
#[inline]
pub fn ep_file_key(file: usize) -> u64 {
    get().ep_file[file]
}

/// Fully recompute the Zobrist hash for a board position.
pub fn recalc_zobrist_full(board: &Board) -> u64 {
    let z = get();
    let mut h = 0u64;

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
            let mut bb = board.piece_bb(kind, color);
            while let Some(sq) = crate::utils::pop_lsb(&mut bb) {
                h ^= z.piece[piece_index(kind, color)][sq];
            }
        }
    }

    if board.side == Color::Black {
        h ^= z.side;
    }

    h ^= z.castling[board.castling as usize];

    if let Some(ep_sq) = board.ep {
        let file = ep_sq % 8;
        h ^= z.ep_file[file as usize];
    }

    h
}
