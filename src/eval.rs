//! Modulo di valutazione - Tapered Evaluation (MG/EG interpolation)
//!
//! Implementa la Tapered Evaluation con Piece-Square Tables PeSTO-style:
//! ogni pezzo ha valori separati per middlegame (MG) e endgame (EG).
//! La valutazione finale interpola tra i due in base alla fase di gioco,
//! determinata dal materiale presente sulla scacchiera.

use crate::board::{Board, Color, PieceKind};

// ============================================================================
// SCORE STRUCT (MG/EG pair)
// ============================================================================

/// A pair of middlegame and endgame scores, used throughout the evaluation.
/// Tapered evaluation accumulates both phases simultaneously, then interpolates
/// at the end based on game phase.
#[derive(Clone, Copy, Default)]
struct Score {
    mg: i32,
    eg: i32,
}

impl core::ops::Add for Score {
    type Output = Score;
    fn add(self, rhs: Score) -> Score {
        Score {
            mg: self.mg + rhs.mg,
            eg: self.eg + rhs.eg,
        }
    }
}

impl core::ops::Sub for Score {
    type Output = Score;
    fn sub(self, rhs: Score) -> Score {
        Score {
            mg: self.mg - rhs.mg,
            eg: self.eg - rhs.eg,
        }
    }
}

impl core::ops::AddAssign for Score {
    fn add_assign(&mut self, rhs: Score) {
        self.mg += rhs.mg;
        self.eg += rhs.eg;
    }
}

impl core::ops::SubAssign for Score {
    fn sub_assign(&mut self, rhs: Score) {
        self.mg -= rhs.mg;
        self.eg -= rhs.eg;
    }
}

impl core::ops::Neg for Score {
    type Output = Score;
    fn neg(self) -> Score {
        Score {
            mg: -self.mg,
            eg: -self.eg,
        }
    }
}

impl core::ops::Mul<i32> for Score {
    type Output = Score;
    fn mul(self, rhs: i32) -> Score {
        Score {
            mg: self.mg * rhs,
            eg: self.eg * rhs,
        }
    }
}

/// Helper to create a Score from middlegame and endgame values.
const fn s(mg: i32, eg: i32) -> Score {
    Score { mg, eg }
}

// ============================================================================
// GAME PHASE
// ============================================================================

/// Phase weights per piece type (excluding pawns and king).
const PHASE_KNIGHT: i32 = 1;
const PHASE_BISHOP: i32 = 1;
const PHASE_ROOK: i32 = 2;
const PHASE_QUEEN: i32 = 4;
/// Maximum phase value (all pieces on board): 4N + 4B + 4R + 2Q = 4+4+8+8 = 24
const PHASE_TOTAL: i32 = 24;

/// Compute the game phase from the board.
/// Returns a value in [0, PHASE_TOTAL] where PHASE_TOTAL = full middlegame, 0 = pure endgame.
fn game_phase(board: &Board) -> i32 {
    let mut phase = 0;

    // Count all knights (both colors)
    phase += board.piece_bb(PieceKind::Knight, Color::White).count_ones() as i32 * PHASE_KNIGHT;
    phase += board.piece_bb(PieceKind::Knight, Color::Black).count_ones() as i32 * PHASE_KNIGHT;

    // Count all bishops
    phase += board.piece_bb(PieceKind::Bishop, Color::White).count_ones() as i32 * PHASE_BISHOP;
    phase += board.piece_bb(PieceKind::Bishop, Color::Black).count_ones() as i32 * PHASE_BISHOP;

    // Count all rooks
    phase += board.piece_bb(PieceKind::Rook, Color::White).count_ones() as i32 * PHASE_ROOK;
    phase += board.piece_bb(PieceKind::Rook, Color::Black).count_ones() as i32 * PHASE_ROOK;

    // Count all queens
    phase += board.piece_bb(PieceKind::Queen, Color::White).count_ones() as i32 * PHASE_QUEEN;
    phase += board.piece_bb(PieceKind::Queen, Color::Black).count_ones() as i32 * PHASE_QUEEN;

    // Clamp to PHASE_TOTAL (should not exceed, but safety)
    phase.min(PHASE_TOTAL)
}

/// Interpolate between middlegame and endgame scores based on game phase.
/// phase = PHASE_TOTAL -> pure MG, phase = 0 -> pure EG.
fn tapered_score(score: Score, phase: i32) -> i32 {
    (score.mg * phase + score.eg * (PHASE_TOTAL - phase)) / PHASE_TOTAL
}

// ============================================================================
// MATERIAL VALUES (PeSTO-like)
// ============================================================================

const PAWN_VALUE: Score = s(82, 94);
const KNIGHT_VALUE: Score = s(337, 281);
const BISHOP_VALUE: Score = s(365, 297);
const ROOK_VALUE: Score = s(477, 512);
const QUEEN_VALUE: Score = s(1025, 936);

/// King value for SEE (Static Exchange Evaluation) in search.rs.
/// The king has no material value in the tapered eval (cancels out),
/// but SEE needs a large value to represent king captures.
pub const KING_VALUE_SEE: i16 = 20000;

const SIMPLE_ENDGAME_BONUS: i16 = 10000;

// ============================================================================
// PIECE-SQUARE TABLES (PeSTO values, from White's perspective)
// ============================================================================
// Index layout: rank 1 = indices 0-7 (A1..H1), rank 8 = indices 56-63 (A8..H8)
// For Black, we mirror vertically: flip_sq = sq XOR 56

// NOTE: PeSTO tables are provided visually as rank 8 (top) to rank 1 (bottom),
// but our indexing uses rank 1 = indices 0-7, rank 8 = indices 56-63.
// So the rows below are reversed from the standard PeSTO visual layout.

#[rustfmt::skip]
const PAWN_PSQT_MG: [i32; 64] = [
    // Rank 1 (indices 0-7) - impossible for pawns
      0,   0,   0,   0,   0,   0,   0,   0,
    // Rank 2 (indices 8-15)
    -35,  -1, -20, -23, -15,  24,  38, -22,
    // Rank 3
    -26,  -4,  -4, -10,   3,   3,  33, -12,
    // Rank 4
    -27,  -2,  -5,  12,  17,   6,  10, -25,
    // Rank 5
    -14,  13,   6,  21,  23,  12,  17, -23,
    // Rank 6
     -6,   7,  26,  31,  65,  56,  25, -20,
    // Rank 7
     98, 134,  61,  95,  68, 126,  34, -11,
    // Rank 8 (indices 56-63) - impossible for pawns
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const PAWN_PSQT_EG: [i32; 64] = [
    // Rank 1
      0,   0,   0,   0,   0,   0,   0,   0,
    // Rank 2
     13,   8,   8, -10,  13,   0,   2,  -7,
    // Rank 3
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
    // Rank 4
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
    // Rank 5
     32,  24,  13,   5,  -2,   4,  17,  17,
    // Rank 6
     94, 100,  85,  67,  56,  53,  82,  84,
    // Rank 7
    178, 173, 158, 134, 147, 132, 165, 187,
    // Rank 8
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const KNIGHT_PSQT_MG: [i32; 64] = [
    // Rank 1
   -105, -21, -58, -33, -17, -28, -19, -23,
    // Rank 2
    -29, -53, -12,  -3,  -1,  18, -14, -19,
    // Rank 3
    -23,  -9,  12,  10,  19,  17,  25, -16,
    // Rank 4
    -13,   4,  16,  13,  28,  19,  21,  -8,
    // Rank 5
     -9,  17,  19,  53,  37,  69,  18,  22,
    // Rank 6
    -47,  60,  37,  65,  84, 129,  73,  44,
    // Rank 7
    -73, -41,  72,  36,  23,  62,   7, -17,
    // Rank 8
   -167, -89, -34, -49,  61, -97, -15,-107,
];

#[rustfmt::skip]
const KNIGHT_PSQT_EG: [i32; 64] = [
    // Rank 1
    -29, -51, -23, -15, -22, -18, -50, -64,
    // Rank 2
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    // Rank 3
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    // Rank 4
    -18,  -6,  16,  25,  16,  17,   4, -18,
    // Rank 5
    -17,   3,  22,  22,  22,  11,   8, -18,
    // Rank 6
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    // Rank 7
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    // Rank 8
    -58, -38, -13, -28, -31, -27, -63, -99,
];

#[rustfmt::skip]
const BISHOP_PSQT_MG: [i32; 64] = [
    // Rank 1
    -33,  -3, -14, -21, -13, -12, -39, -21,
    // Rank 2
      4,  15,  16,   0,   7,  21,  33,   1,
    // Rank 3
      0,  15,  15,  15,  14,  27,  18,  10,
    // Rank 4
     -6,  13,  13,  26,  34,  12,  10,   4,
    // Rank 5
     -4,   5,  19,  50,  37,  37,   7,  -2,
    // Rank 6
    -16,  37,  43,  40,  35,  50,  37,  -2,
    // Rank 7
    -26,  16, -18, -13,  30,  59,  18, -47,
    // Rank 8
    -29,   4, -82, -37, -25, -42,   7,  -8,
];

#[rustfmt::skip]
const BISHOP_PSQT_EG: [i32; 64] = [
    // Rank 1
    -23,  -9, -23,  -5,  -9, -16,  -5, -17,
    // Rank 2
    -14, -18,  -7,  -1,   4,  -9, -15, -27,
    // Rank 3
    -12,  -3,   8,  10,  13,   3,  -7, -15,
    // Rank 4
     -6,   3,  13,  19,   7,  10,  -3,  -9,
    // Rank 5
     -3,   9,  12,   9,  14,  10,   3,   2,
    // Rank 6
      2,  -8,   0,  -1,  -2,   6,   0,   4,
    // Rank 7
     -8,  -4,   7, -12,  -3, -13,  -4, -14,
    // Rank 8
    -14, -21, -11,  -8,  -7,  -9, -17, -24,
];

#[rustfmt::skip]
const ROOK_PSQT_MG: [i32; 64] = [
    // Rank 1
    -19, -13,   1,  17,  16,   7, -37, -26,
    // Rank 2
    -44, -16, -20,  -9,  -1,  11,  -6, -71,
    // Rank 3
    -45, -25, -16, -17,   3,   0,  -5, -33,
    // Rank 4
    -36, -26, -12,  -1,   9,  -7,   6, -23,
    // Rank 5
    -24, -11,   7,  26,  24,  35,  -8, -20,
    // Rank 6
     -5,  19,  26,  36,  17,  45,  61,  16,
    // Rank 7
     27,  32,  58,  62,  80,  67,  26,  44,
    // Rank 8
     32,  42,  32,  51,  63,   9,  31,  43,
];

#[rustfmt::skip]
const ROOK_PSQT_EG: [i32; 64] = [
    // Rank 1
     -9,   2,   3,  -1,  -5, -13,   4, -20,
    // Rank 2
     -6,  -6,   0,   2,  -9,  -9, -11,  -3,
    // Rank 3
     -4,   0,  -5,  -1,  -7, -12,  -8, -16,
    // Rank 4
      3,   5,   8,   4,  -5,  -6,  -8, -11,
    // Rank 5
      4,   3,  13,   1,   2,   1,  -1,   2,
    // Rank 6
      7,   7,   7,   5,   4,  -3,  -5,  -3,
    // Rank 7
     11,  13,  13,  11,  -3,   3,   8,   3,
    // Rank 8
     13,  10,  18,  15,  12,  12,   8,   5,
];

#[rustfmt::skip]
const QUEEN_PSQT_MG: [i32; 64] = [
    // Rank 1
     -1, -18,  -9,  10, -15, -25, -31, -50,
    // Rank 2
    -35,  -8,  11,   2,   8,  15,  -3,   1,
    // Rank 3
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    // Rank 4
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    // Rank 5
    -27, -27, -16, -16,  -1,  17,  -2,   1,
    // Rank 6
    -13, -17,   7,   8,  29,  56,  47,  57,
    // Rank 7
    -24, -39,  -5,   1, -16,  57,  28,  54,
    // Rank 8
    -28,   0,  29,  12,  59,  44,  43,  45,
];

#[rustfmt::skip]
const QUEEN_PSQT_EG: [i32; 64] = [
    // Rank 1
    -33, -28, -22, -43,  -5, -32, -20, -41,
    // Rank 2
    -22, -23, -30, -16, -16, -23, -36, -32,
    // Rank 3
    -16, -27,  15,   6,   9,  17,  10,   5,
    // Rank 4
    -18,  28,  19,  47,  31,  34,  39,  23,
    // Rank 5
      3,  22,  24,  45,  57,  40,  57,  36,
    // Rank 6
    -20,   6,   9,  49,  47,  35,  19,   9,
    // Rank 7
    -17,  20,  32,  41,  58,  25,  30,   0,
    // Rank 8
     -9,  22,  22,  27,  27,  19,  10,  20,
];

#[rustfmt::skip]
const KING_PSQT_MG: [i32; 64] = [
    // Rank 1
    -15,  36,  12, -54,   8, -28,  24,  14,
    // Rank 2
      1,   7,  -8, -64, -43, -16,   9,   8,
    // Rank 3
    -14, -14, -22, -46, -44, -30, -15, -27,
    // Rank 4
    -49,  -1, -27, -39, -46, -44, -33, -51,
    // Rank 5
    -17, -20, -12, -27, -30, -25, -14, -36,
    // Rank 6
     -9,  24,   2, -16, -20,   6,  22, -22,
    // Rank 7
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
    // Rank 8
    -65,  23,  16, -15, -56, -34,   2,  13,
];

#[rustfmt::skip]
const KING_PSQT_EG: [i32; 64] = [
    // Rank 1
    -53, -34, -21, -11, -28, -14, -24, -43,
    // Rank 2
    -27, -11,   4,  13,  14,   4,  -5, -17,
    // Rank 3
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    // Rank 4
    -18,  -4,  21,  24,  27,  23,   9, -11,
    // Rank 5
     -8,  22,  24,  27,  26,  33,  26,   3,
    // Rank 6
     10,  17,  23,  15,  20,  45,  44,  13,
    // Rank 7
    -12,  17,  14,  17,  17,  38,  23,  11,
    // Rank 8
    -74, -35, -18, -18, -11,  15,   4, -17,
];

/// Combined PSQT lookup: returns Score(mg, eg) for a given piece on a given square.
fn psqt_score(kind: PieceKind, sq: usize) -> Score {
    match kind {
        PieceKind::Pawn => s(PAWN_PSQT_MG[sq], PAWN_PSQT_EG[sq]),
        PieceKind::Knight => s(KNIGHT_PSQT_MG[sq], KNIGHT_PSQT_EG[sq]),
        PieceKind::Bishop => s(BISHOP_PSQT_MG[sq], BISHOP_PSQT_EG[sq]),
        PieceKind::Rook => s(ROOK_PSQT_MG[sq], ROOK_PSQT_EG[sq]),
        PieceKind::Queen => s(QUEEN_PSQT_MG[sq], QUEEN_PSQT_EG[sq]),
        PieceKind::King => s(KING_PSQT_MG[sq], KING_PSQT_EG[sq]),
    }
}

/// Material value as Score for a piece kind. King returns Score(0,0) because
/// both sides always have exactly one king, so material cancels out.
fn material_score(kind: PieceKind) -> Score {
    match kind {
        PieceKind::Pawn => PAWN_VALUE,
        PieceKind::Knight => KNIGHT_VALUE,
        PieceKind::Bishop => BISHOP_VALUE,
        PieceKind::Rook => ROOK_VALUE,
        PieceKind::Queen => QUEEN_VALUE,
        PieceKind::King => s(0, 0), // King material cancels; only PSQT matters
    }
}

// ============================================================================
// SIMPLE ENDGAMES (unchanged logic)
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Default)]
struct MaterialCounts {
    pawns: u32,
    knights: u32,
    bishops: u32,
    rooks: u32,
    queens: u32,
}

impl MaterialCounts {
    fn is_empty(&self) -> bool {
        self.pawns == 0
            && self.knights == 0
            && self.bishops == 0
            && self.rooks == 0
            && self.queens == 0
    }
}

const SIMPLE_ENDGAME_SIGNATURES: [MaterialCounts; 3] = [
    MaterialCounts {
        pawns: 0,
        knights: 0,
        bishops: 0,
        rooks: 0,
        queens: 1,
    },
    MaterialCounts {
        pawns: 0,
        knights: 0,
        bishops: 0,
        rooks: 1,
        queens: 0,
    },
    MaterialCounts {
        pawns: 0,
        knights: 1,
        bishops: 1,
        rooks: 0,
        queens: 0,
    },
];

fn material_counts(board: &Board, color: Color) -> MaterialCounts {
    MaterialCounts {
        pawns: board.piece_bb(PieceKind::Pawn, color).count_ones(),
        knights: board.piece_bb(PieceKind::Knight, color).count_ones(),
        bishops: board.piece_bb(PieceKind::Bishop, color).count_ones(),
        rooks: board.piece_bb(PieceKind::Rook, color).count_ones(),
        queens: board.piece_bb(PieceKind::Queen, color).count_ones(),
    }
}

fn simple_endgame_score(attacker: Color, side: Color) -> i16 {
    if attacker == side {
        SIMPLE_ENDGAME_BONUS
    } else {
        -SIMPLE_ENDGAME_BONUS
    }
}

fn simple_endgame_bonus(board: &Board) -> Option<i16> {
    let white_counts = material_counts(board, Color::White);
    let black_counts = material_counts(board, Color::Black);

    for signature in &SIMPLE_ENDGAME_SIGNATURES {
        if white_counts == *signature && black_counts.is_empty() {
            return Some(simple_endgame_score(Color::White, board.side));
        }
        if black_counts == *signature && white_counts.is_empty() {
            return Some(simple_endgame_score(Color::Black, board.side));
        }
    }

    None
}

// ============================================================================
// KING SAFETY
// ============================================================================

/// Check if the king has castled by examining its position.
fn has_castled(board: &Board, color: Color) -> bool {
    let king_sq = board.king_sq(color);
    match color {
        Color::White => king_sq == 6 || king_sq == 2,
        Color::Black => king_sq == 62 || king_sq == 58,
    }
}

/// Count pawn shield squares (up to 3) in front of the king.
fn count_pawn_shield(board: &Board, king_sq: usize, color: Color) -> i32 {
    let pawns = board.piece_bb(PieceKind::Pawn, color);
    let mut count = 0;

    let file = king_sq % 8;
    let rank = king_sq / 8;

    let shield_files = [file.saturating_sub(1), file, (file + 1).min(7)];
    let shield_rank = if color == Color::White {
        rank + 1
    } else {
        rank.saturating_sub(1)
    };

    if shield_rank >= 8 {
        return 0;
    }

    for &f in &shield_files {
        let sq = shield_rank * 8 + f;
        if sq < 64 && (pawns & (1u64 << sq)) != 0 {
            count += 1;
        }
    }

    count
}

/// Count active minor and major pieces (N, B, R, Q) for a color.
fn count_active_pieces(board: &Board, color: Color) -> i32 {
    let mut count = 0;
    count += board.piece_bb(PieceKind::Knight, color).count_ones() as i32;
    count += board.piece_bb(PieceKind::Bishop, color).count_ones() as i32;
    count += board.piece_bb(PieceKind::Rook, color).count_ones() as i32;
    count += board.piece_bb(PieceKind::Queen, color).count_ones() as i32;
    count
}

/// Check if the king still has castling rights.
fn has_castling_rights(board: &Board, color: Color) -> bool {
    match color {
        Color::White => (board.castling & 0b1100u8) != 0,
        Color::Black => (board.castling & 0b0011u8) != 0,
    }
}

/// Check if the king is in check.
fn is_in_check(board: &Board, color: Color) -> bool {
    let king_sq = board.king_sq(color);
    let opponent_color = match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    board.is_square_attacked(king_sq, opponent_color)
}

/// Evaluate king safety. Returns a raw i32 value (positive = safe, negative = danger).
fn king_safety(board: &Board, color: Color) -> i32 {
    let mut safety: i32 = 0;

    let king_sq = board.king_sq(color);
    let file = king_sq % 8;
    let castled = has_castled(board, color);
    let has_rights = has_castling_rights(board, color);

    // Catastrophic penalty: lost castling rights in opening without castling
    if board.fullmove >= 5 && board.fullmove < 15 && !castled && !has_rights {
        safety -= 400;
    }

    // Threat evaluation: king in check with castling rights
    if is_in_check(board, color) && has_rights {
        safety -= 200;
    }

    // Dynamic penalty for king in center, proportional to opponent active pieces
    if (file == 3 || file == 4) && !castled {
        let opponent_color = match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
        let active_pieces = count_active_pieces(board, opponent_color);
        let base_penalty: i32 = 50;
        let multiplier = 100 + (active_pieces * 16);
        safety -= base_penalty * multiplier / 100;
    }

    // Pawn shield bonus
    let pawn_shield = count_pawn_shield(board, king_sq, color);
    safety += pawn_shield * 15;

    safety
}

/// Fast version of king_safety: only catastrophic penalties.
fn king_safety_critical_only(board: &Board, color: Color) -> i32 {
    let mut safety: i32 = 0;

    let castled = has_castled(board, color);
    let has_rights = has_castling_rights(board, color);

    if board.fullmove >= 5 && board.fullmove < 15 && !castled && !has_rights {
        safety -= 400;
    }

    if is_in_check(board, color) && has_rights {
        safety -= 200;
    }

    safety
}

// ============================================================================
// PAWN STRUCTURE
// ============================================================================

/// Bitboard mask for each file (A=0 .. H=7).
const FILE_MASKS: [u64; 8] = [
    0x0101_0101_0101_0101, // A-file
    0x0202_0202_0202_0202, // B-file
    0x0404_0404_0404_0404, // C-file
    0x0808_0808_0808_0808, // D-file
    0x1010_1010_1010_1010, // E-file
    0x2020_2020_2020_2020, // F-file
    0x4040_4040_4040_4040, // G-file
    0x8080_8080_8080_8080, // H-file
];

/// Bitboard mask for files adjacent to a given file.
const ADJACENT_FILES: [u64; 8] = [
    FILE_MASKS[1],                         // file A -> B
    FILE_MASKS[0] | FILE_MASKS[2],         // file B -> A, C
    FILE_MASKS[1] | FILE_MASKS[3],         // file C -> B, D
    FILE_MASKS[2] | FILE_MASKS[4],         // file D -> C, E
    FILE_MASKS[3] | FILE_MASKS[5],         // file E -> D, F
    FILE_MASKS[4] | FILE_MASKS[6],         // file F -> E, G
    FILE_MASKS[5] | FILE_MASKS[7],         // file G -> F, H
    FILE_MASKS[6],                         // file H -> G
];

/// Penalty (positive Score subtracted) for each isolated pawn.
const ISOLATED_PAWN_PENALTY: Score = s(5, 15);
/// Penalty for doubled pawns on a file (applied once per file with >1 pawn).
const DOUBLED_PAWN_PENALTY: Score = s(11, 20);
/// Penalty for a backward pawn.
const BACKWARD_PAWN_PENALTY: Score = s(6, 10);

/// Evaluate pawn structure for one color. Returns penalties (positive Score to subtract).
fn pawn_structure(board: &Board, color: Color) -> Score {
    let our_pawns = board.piece_bb(PieceKind::Pawn, color);
    let their_pawns = board.piece_bb(
        PieceKind::Pawn,
        match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        },
    );
    let mut penalty = Score::default();

    for file in 0..8u8 {
        let pawns_on_file = our_pawns & FILE_MASKS[file as usize];
        let count = pawns_on_file.count_ones();
        if count == 0 {
            continue;
        }

        // Doubled: more than one pawn on the same file
        if count > 1 {
            penalty += DOUBLED_PAWN_PENALTY;
        }

        // Isolated: no friendly pawns on adjacent files
        let adjacent = ADJACENT_FILES[file as usize];
        let is_isolated = our_pawns & adjacent == 0;
        if is_isolated {
            penalty += ISOLATED_PAWN_PENALTY * count as i32;
        }

        // Backward: only check if the pawn is NOT isolated (isolated already penalised)
        if !is_isolated {
            let mut bb = pawns_on_file;
            while bb != 0 {
                let sq = bb.trailing_zeros() as usize;
                let rank = sq / 8;

                // Mask for all ranks behind (inclusive of this rank) on adjacent files
                let stop_mask = match color {
                    Color::White => {
                        // For white, "behind" means lower ranks (rank-1 down to rank 1)
                        if rank <= 1 {
                            0u64
                        } else {
                            let below_mask = (1u64 << (rank * 8)) - 1;
                            adjacent & below_mask
                        }
                    }
                    Color::Black => {
                        // For black, "behind" means higher ranks
                        if rank >= 6 {
                            0u64
                        } else {
                            let above_mask = !((1u64 << ((rank + 1) * 8)) - 1);
                            adjacent & above_mask
                        }
                    }
                };

                // If no friendly pawns behind on adjacent files can support...
                if our_pawns & stop_mask == 0 {
                    // ...and the advance square is attacked by enemy pawns
                    let advance_sq = match color {
                        Color::White => {
                            if rank < 7 {
                                Some(sq + 8)
                            } else {
                                None
                            }
                        }
                        Color::Black => {
                            if rank > 0 {
                                Some(sq - 8)
                            } else {
                                None
                            }
                        }
                    };

                    if let Some(adv_sq) = advance_sq {
                        let adv_file = adv_sq % 8;
                        let enemy_attack_rank = match color {
                            Color::White => {
                                if (adv_sq / 8) < 7 {
                                    adv_sq / 8 + 1
                                } else {
                                    8
                                }
                            }
                            Color::Black => {
                                if (adv_sq / 8) > 0 {
                                    adv_sq / 8 - 1
                                } else {
                                    99
                                }
                            }
                        };

                        if enemy_attack_rank < 8 {
                            let mut attacked = false;
                            if adv_file > 0 {
                                let att_sq = enemy_attack_rank * 8 + adv_file - 1;
                                if their_pawns & (1u64 << att_sq) != 0 {
                                    attacked = true;
                                }
                            }
                            if adv_file < 7 {
                                let att_sq = enemy_attack_rank * 8 + adv_file + 1;
                                if their_pawns & (1u64 << att_sq) != 0 {
                                    attacked = true;
                                }
                            }
                            if attacked {
                                penalty += BACKWARD_PAWN_PENALTY;
                            }
                        }
                    }
                }

                bb &= bb - 1;
            }
        }
    }

    penalty
}

// ============================================================================
// DEVELOPMENT PENALTY
// ============================================================================

/// Penalty for undeveloped minor pieces after move 10.
fn development_penalty(board: &Board, color: Color) -> i32 {
    if board.fullmove <= 10 {
        return 0;
    }

    let mut penalty: i32 = 0;

    match color {
        Color::White => {
            const RANK_1_MASK: u64 = 0xFF;
            let white_knights = board.piece_bb(PieceKind::Knight, Color::White);
            penalty += (white_knights & RANK_1_MASK).count_ones() as i32 * 10;
            let white_bishops = board.piece_bb(PieceKind::Bishop, Color::White);
            penalty += (white_bishops & RANK_1_MASK).count_ones() as i32 * 10;
        }
        Color::Black => {
            const RANK_8_MASK: u64 = 0xFF00_0000_0000_0000;
            let black_knights = board.piece_bb(PieceKind::Knight, Color::Black);
            penalty += (black_knights & RANK_8_MASK).count_ones() as i32 * 10;
            let black_bishops = board.piece_bb(PieceKind::Bishop, Color::Black);
            penalty += (black_bishops & RANK_8_MASK).count_ones() as i32 * 10;
        }
    }

    penalty
}

// ============================================================================
// MAIN EVALUATION FUNCTIONS
// ============================================================================

/// Quick material count using MG values (average of MG and EG for stability).
/// Used for evaluate_lazy threshold check.
fn quick_material_count(board: &Board) -> i16 {
    // Use average of MG and EG for each piece type
    const PAWN_AVG: i32 = (82 + 94) / 2;     // 88
    const KNIGHT_AVG: i32 = (337 + 281) / 2;  // 309
    const BISHOP_AVG: i32 = (365 + 297) / 2;  // 331
    const ROOK_AVG: i32 = (477 + 512) / 2;    // 494
    const QUEEN_AVG: i32 = (1025 + 936) / 2;  // 980

    let white_material =
        board.piece_bb(PieceKind::Pawn, Color::White).count_ones() as i32 * PAWN_AVG
        + board.piece_bb(PieceKind::Knight, Color::White).count_ones() as i32 * KNIGHT_AVG
        + board.piece_bb(PieceKind::Bishop, Color::White).count_ones() as i32 * BISHOP_AVG
        + board.piece_bb(PieceKind::Rook, Color::White).count_ones() as i32 * ROOK_AVG
        + board.piece_bb(PieceKind::Queen, Color::White).count_ones() as i32 * QUEEN_AVG;

    let black_material =
        board.piece_bb(PieceKind::Pawn, Color::Black).count_ones() as i32 * PAWN_AVG
        + board.piece_bb(PieceKind::Knight, Color::Black).count_ones() as i32 * KNIGHT_AVG
        + board.piece_bb(PieceKind::Bishop, Color::Black).count_ones() as i32 * BISHOP_AVG
        + board.piece_bb(PieceKind::Rook, Color::Black).count_ones() as i32 * ROOK_AVG
        + board.piece_bb(PieceKind::Queen, Color::Black).count_ones() as i32 * QUEEN_AVG;

    let relative = (white_material - black_material).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    if board.side == Color::Black {
        -relative
    } else {
        relative
    }
}

/// Lazy evaluation: fast material check with threshold, fallback to full eval.
///
/// Strategy:
/// 1. Quick material-only count
/// 2. If |material| > threshold (3 pawns ~264cp), position is clearly won/lost -> return material
/// 3. Otherwise, do full evaluation
pub fn evaluate_lazy(board: &Board) -> i16 {
    if let Some(bonus) = simple_endgame_bonus(board) {
        return bonus;
    }

    let material = quick_material_count(board);

    const LAZY_THRESHOLD: i16 = 300;

    if material.abs() > LAZY_THRESHOLD {
        return material;
    }

    evaluate(board)
}

/// Fast evaluation: material + PSQT with tapered interpolation + critical king safety.
/// Used in quiescence search where speed is critical.
pub fn evaluate_fast(board: &Board) -> i16 {
    if let Some(bonus) = simple_endgame_bonus(board) {
        return bonus;
    }

    let phase = game_phase(board);
    let mut white_score = Score::default();
    let mut black_score = Score::default();

    let piece_kinds = [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ];

    for &kind in &piece_kinds {
        let mat = material_score(kind);

        // White pieces
        let mut bb = board.piece_bb(kind, Color::White);
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            white_score += mat + psqt_score(kind, sq);
            bb &= bb - 1;
        }

        // Black pieces (mirror vertically for PSQT)
        let mut bb = board.piece_bb(kind, Color::Black);
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            black_score += mat + psqt_score(kind, sq ^ 56);
            bb &= bb - 1;
        }
    }

    // Critical king safety penalties
    let white_king_penalty = king_safety_critical_only(board, Color::White);
    let black_king_penalty = king_safety_critical_only(board, Color::Black);

    // Convert king safety to Score: full weight in MG, half in EG
    white_score += s(white_king_penalty, white_king_penalty / 2);
    black_score += s(black_king_penalty, black_king_penalty / 2);

    let diff = white_score - black_score;
    let score = tapered_score(diff, phase).clamp(i16::MIN as i32, i16::MAX as i32) as i16;

    if board.side == Color::Black {
        -score
    } else {
        score
    }
}

/// Full evaluation: material + PSQT + king safety + development penalty.
///
/// Returns score from the side-to-move perspective (negamax convention).
pub fn evaluate(board: &Board) -> i16 {
    if let Some(bonus) = simple_endgame_bonus(board) {
        return bonus;
    }

    let phase = game_phase(board);
    let mut white_score = Score::default();
    let mut black_score = Score::default();

    let piece_kinds = [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ];

    for &kind in &piece_kinds {
        let mat = material_score(kind);

        // White pieces
        let mut bb = board.piece_bb(kind, Color::White);
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            white_score += mat + psqt_score(kind, sq);
            bb &= bb - 1;
        }

        // Black pieces (mirror vertically for PSQT)
        let mut bb = board.piece_bb(kind, Color::Black);
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            black_score += mat + psqt_score(kind, sq ^ 56);
            bb &= bb - 1;
        }
    }

    // Pawn structure penalties
    let white_pawn_penalty = pawn_structure(board, Color::White);
    let black_pawn_penalty = pawn_structure(board, Color::Black);
    white_score -= white_pawn_penalty;
    black_score -= black_pawn_penalty;

    // Development penalty (MG only)
    let white_dev = development_penalty(board, Color::White);
    let black_dev = development_penalty(board, Color::Black);
    white_score -= s(white_dev, 0);
    black_score -= s(black_dev, 0);

    // King safety (full weight in MG, half in EG)
    let white_ks = king_safety(board, Color::White);
    let black_ks = king_safety(board, Color::Black);
    white_score += s(white_ks, white_ks / 2);
    black_score += s(black_ks, black_ks / 2);

    let diff = white_score - black_score;
    let score = tapered_score(diff, phase).clamp(i16::MIN as i32, i16::MAX as i32) as i16;

    if board.side == Color::Black {
        -score
    } else {
        score
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_startpos() {
        let mut board = Board::new();
        board
            .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();

        let score = evaluate(&board);

        // Starting position should be roughly equal (small PSQT asymmetries)
        assert!(
            score.abs() < 100,
            "Startpos should be balanced, but score = {score}"
        );
    }

    #[test]
    fn test_evaluate_side_to_move_symmetry() {
        let mut board_white = Board::new();
        board_white
            .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();

        let mut board_black = Board::new();
        board_black
            .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1")
            .unwrap();

        let score_white = evaluate(&board_white);
        let score_black = evaluate(&board_black);

        assert_eq!(
            score_white, -score_black,
            "Negamax convention: white={score_white}, black={score_black}"
        );
    }

    #[test]
    fn test_evaluate_central_pawn_bonus() {
        // Central pawn (e4) should score better than edge pawn (a4) in a middlegame-like position
        // We need enough pieces on the board so that MG values dominate (phase > 0)
        let mut board_central = Board::new();
        board_central
            .set_from_fen("r1bqkbnr/pppppppp/2n5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3")
            .unwrap();

        let mut board_edge = Board::new();
        board_edge
            .set_from_fen("r1bqkbnr/pppppppp/2n5/8/P7/8/1PPP1PPP/RNBQKBNR w KQkq - 0 3")
            .unwrap();

        let score_central = evaluate(&board_central);
        let score_edge = evaluate(&board_edge);

        assert!(
            score_central > score_edge,
            "Central pawn (e4) should be better than edge pawn (a4): central={score_central}, edge={score_edge}"
        );
    }

    #[test]
    fn test_evaluate_knight_center_bonus() {
        // Central knight should score better than rim knight
        let mut board_central = Board::new();
        board_central
            .set_from_fen("4k3/8/8/8/4N3/8/8/4K3 w - - 0 1")
            .unwrap();

        let mut board_rim = Board::new();
        board_rim
            .set_from_fen("4k3/8/8/8/N7/8/8/4K3 w - - 0 1")
            .unwrap();

        let score_central = evaluate(&board_central);
        let score_rim = evaluate(&board_rim);

        assert!(
            score_central > score_rim,
            "Central knight (e4) should be better than rim knight (a4): central={score_central}, rim={score_rim}"
        );
    }

    #[test]
    fn test_evaluate_castled_king_bonus() {
        // Castled king (g1) should score better than uncastled king (e1)
        let mut board_castled = Board::new();
        board_castled
            .set_from_fen("rnbq1rk1/pppp1ppp/4pn2/3P4/4P3/5N2/PPP2PPP/RNBQ1RK1 w - - 0 1")
            .unwrap();
        let score_castled = evaluate(&board_castled);

        let mut board_uncastled = Board::new();
        board_uncastled
            .set_from_fen("rnbq1rk1/pppp1ppp/4pn2/3P4/4P3/5N2/PPP2PPP/RNBQK2R w - - 0 1")
            .unwrap();
        let score_uncastled = evaluate(&board_uncastled);

        assert!(
            score_castled > score_uncastled,
            "Castled king should be better: castled={score_castled}, uncastled={score_uncastled}"
        );
    }

    #[test]
    fn test_simple_endgame_kq_vs_k() {
        let mut board = Board::new();
        board
            .set_from_fen("4k3/8/8/8/8/8/8/4K2Q w - - 0 1")
            .unwrap();
        assert_eq!(evaluate(&board), SIMPLE_ENDGAME_BONUS);

        let mut board_black_to_move = Board::new();
        board_black_to_move
            .set_from_fen("4k3/8/8/8/8/8/8/4K2Q b - - 0 1")
            .unwrap();
        assert_eq!(evaluate(&board_black_to_move), -SIMPLE_ENDGAME_BONUS);
    }

    #[test]
    fn test_simple_endgame_kr_vs_k() {
        let mut board = Board::new();
        board
            .set_from_fen("4k3/8/8/8/8/8/8/4K2R w - - 0 1")
            .unwrap();
        assert_eq!(evaluate(&board), SIMPLE_ENDGAME_BONUS);
    }

    #[test]
    fn test_simple_endgame_knb_vs_k() {
        let mut board = Board::new();
        board
            .set_from_fen("4k3/8/8/8/8/8/4B1N1/4K3 w - - 0 1")
            .unwrap();
        assert_eq!(evaluate(&board), SIMPLE_ENDGAME_BONUS);
    }

    #[test]
    fn test_king_safety_lost_castling_rights_in_opening() {
        // King that lost castling rights in opening should get severe penalty
        let mut board = Board::new();
        board
            .set_from_fen("r5k1/pp2qppp/1n1p4/2pPb3/2P1P3/2N2N2/PP2BPPP/R1B2K1R b - - 1 14")
            .unwrap();

        let white_safety = king_safety(&board, Color::White);

        assert!(
            white_safety < -300,
            "King that lost castling rights in opening should have severe penalty: white_safety={white_safety}"
        );

        // Compare: with castling rights, safety should be better
        let mut board_with_rights = Board::new();
        board_with_rights
            .set_from_fen("r5k1/pp2qppp/1n1p4/2pPb3/2P1P3/2N2N2/PP2BPPP/R1B2K1R b KQ - 1 14")
            .unwrap();

        let white_safety_with_rights = king_safety(&board_with_rights, Color::White);

        assert!(
            white_safety_with_rights > white_safety,
            "King with castling rights should be safer: with_rights={white_safety_with_rights}, without={white_safety}"
        );
    }

    #[test]
    fn test_evaluate_fast_consistency() {
        // evaluate_fast should produce a reasonable score consistent with evaluate
        let mut board = Board::new();
        board
            .set_from_fen("r3k2r/pppb1ppp/2n1pn2/3q4/3P4/2B1PN2/PP3PPP/R2QKB1R w KQkq - 1 9")
            .unwrap();

        let fast_eval = evaluate_fast(&board);
        let full_eval = evaluate(&board);

        // Fast and full should be in the same ballpark (within ~200cp due to missing
        // development/full king safety terms)
        let diff = (fast_eval as i32 - full_eval as i32).abs();
        assert!(
            diff < 200,
            "evaluate_fast and evaluate should be close: fast={fast_eval}, full={full_eval}, diff={diff}"
        );
    }

    #[test]
    fn test_quick_material_count_balanced() {
        // Starting position should have ~0 material balance
        let mut board = Board::new();
        board
            .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
        let count = quick_material_count(&board);
        assert_eq!(count, 0, "Starting position material balance should be 0");
    }

    #[test]
    fn test_tapered_score_interpolation() {
        let score = s(100, 200);

        // Pure middlegame: phase = PHASE_TOTAL -> should return MG value
        assert_eq!(tapered_score(score, PHASE_TOTAL), 100);

        // Pure endgame: phase = 0 -> should return EG value
        assert_eq!(tapered_score(score, 0), 200);

        // Half phase: should return average
        assert_eq!(tapered_score(score, PHASE_TOTAL / 2), 150);
    }

    #[test]
    fn test_game_phase_startpos() {
        let mut board = Board::new();
        board
            .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();

        let phase = game_phase(&board);
        // 4N(4) + 4B(4) + 4R(8) + 2Q(8) = 24
        assert_eq!(phase, PHASE_TOTAL, "Starting position should have full phase");
    }

    #[test]
    fn test_game_phase_endgame() {
        let mut board = Board::new();
        board
            .set_from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1")
            .unwrap();

        let phase = game_phase(&board);
        assert_eq!(phase, 0, "Kings-only position should have phase 0");
    }

    #[test]
    fn test_isolated_pawn_penalty() {
        // White has an isolated e-pawn (no pawns on d or f files)
        // PPP3PP = a2,b2,c2 then gap d,e,f then g2,h2
        let mut board = Board::new();
        board
            .set_from_fen("4k3/pppppppp/8/8/4P3/8/PPP3PP/4K3 w - - 0 1")
            .unwrap();

        let penalty = pawn_structure(&board, Color::White);
        // The e-pawn is isolated (no pawns on d or f files)
        assert!(
            penalty.mg >= 5 && penalty.eg >= 15,
            "Isolated pawn should have penalty: mg={}, eg={}",
            penalty.mg,
            penalty.eg
        );
    }

    #[test]
    fn test_doubled_pawn_penalty() {
        // White has doubled pawns on e-file
        let mut board = Board::new();
        board
            .set_from_fen("4k3/pppppppp/8/4P3/4P3/8/PPPP1PPP/4K3 w - - 0 1")
            .unwrap();

        let penalty = pawn_structure(&board, Color::White);
        // Doubled pawns on e-file
        assert!(
            penalty.mg >= 11 && penalty.eg >= 20,
            "Doubled pawns should have penalty: mg={}, eg={}",
            penalty.mg,
            penalty.eg
        );
    }

    #[test]
    fn test_no_pawn_structure_penalty_startpos() {
        // Starting position: no isolated, no doubled, minimal backward
        let mut board = Board::new();
        board
            .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();

        let white_penalty = pawn_structure(&board, Color::White);
        let black_penalty = pawn_structure(&board, Color::Black);

        assert_eq!(
            white_penalty.mg, 0,
            "White should have no pawn structure penalties at start"
        );
        assert_eq!(
            white_penalty.eg, 0,
            "White should have no pawn structure penalties at start"
        );
        assert_eq!(
            black_penalty.mg, 0,
            "Black should have no pawn structure penalties at start"
        );
        assert_eq!(
            black_penalty.eg, 0,
            "Black should have no pawn structure penalties at start"
        );
    }
}
