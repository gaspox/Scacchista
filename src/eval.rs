//! Modulo di valutazione - Piece-Square Tables (PSQT)
//!
//! Le PSQT funzionano come una "mappa di calore" della scacchiera:
//! ogni casella ha un valore bonus/malus che incentiva posizioni strategicamente
//! migliori (es: pedoni centrali, cavalieri sviluppati, re protetto dopo arrocco).

use crate::board::{Board, Color, PieceKind};

// ============================================================================
// VALORI MATERIALI (in centipawn)
// ============================================================================
const PAWN_VALUE: i16 = 100;
const KNIGHT_VALUE: i16 = 320;
const BISHOP_VALUE: i16 = 330;
const ROOK_VALUE: i16 = 500;
const QUEEN_VALUE: i16 = 900;
const KING_VALUE: i16 = 20000;

// ============================================================================
// PIECE-SQUARE TABLES (dal punto di vista del BIANCO)
// ============================================================================
// Indici: rank 0 = prima traversa (A1..H1), rank 7 = ottava traversa (A8..H8)
// Per il Nero, specchiamo verticalmente: flip_sq = sq XOR 56

/// PSQT per i pedoni
/// Incentiva:
/// - Pedoni centrali (e4, d4, e5, d5): +20..+30 cp
/// - Avanzamento controllato
/// - Penalità per pedoni arretrati
const PAWN_PSQT: [i16; 64] = [
    // Rank 1 (impossibile per i pedoni, ma per simmetria)
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 2 (pedoni iniziali)
    5, 5, 5, 5, 5, 5, 5, 5, // Rank 3 (ridotto laterali a/b/g/h)
    5, 5, 10, 15, 15, 10, 5, 5, // Rank 4 (aumentato centro d/e, ridotto laterali)
    10, 10, 20, 30, 30, 20, 10, 10, // Rank 5 (avanzati)
    20, 20, 25, 30, 30, 25, 20, 20, // Rank 6 (molto avanzati)
    25, 25, 30, 35, 35, 30, 25, 25, // Rank 7 (vicini alla promozione)
    50, 50, 50, 50, 50, 50, 50, 50, // Rank 8 (impossibile)
    0, 0, 0, 0, 0, 0, 0, 0,
];

/// PSQT per i cavalieri
/// Incentiva:
/// - Posizione centrale (+15 cp come richiesto)
/// - Evita i bordi (rimming knights)
const KNIGHT_PSQT: [i16; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 5, 5, 0, -20, -40, -30, 5, 10, 15, 15, 10,
    5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 10, 15, 15, 10,
    0, -30, -40, -20, 0, 0, 0, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

/// PSQT per gli alfieri
/// Incentiva:
/// - Diagonali lunghe (+10 cp come richiesto)
/// - Posizioni centrali e semi-centrali
const BISHOP_PSQT: [i16; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 5, 0, 0, 0, 0, 5, -10, -10, 10, 10, 10, 10, 10,
    10, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 0, 0, 0, 0, 0, 0, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

/// PSQT per le torri
/// Incentiva:
/// - Colonne aperte/semiaperte (approssimato con bonus centrale)
/// - Settima traversa
const ROOK_PSQT: [i16; 64] = [
    0, 0, 0, 5, 5, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0,
    0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 5, 10, 10, 10, 10, 10, 10,
    5, // Settima traversa
    0, 0, 0, 0, 0, 0, 0, 0,
];

/// PSQT per la regina
/// Incentiva:
/// - Sviluppo centrale in medio-gioco
/// - Penalità per sviluppo prematuro (semplificato)
const QUEEN_PSQT: [i16; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5, 0, -10,
    0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0, 0, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

/// PSQT per il re
/// Incentiva:
/// - Sicurezza dopo arrocco (+30 cp come richiesto)
/// - Evita il centro in apertura/mediogioco
const KING_PSQT: [i16; 64] = [
    20, 30, 10, 0, 0, 10, 30, 20, // Rank 1: bonus arrocco
    20, 20, 0, 0, 0, 0, 20, 20, -10, -20, -20, -20, -20, -20, -20, -10, -20, -30, -30, -40, -40,
    -30, -30, -20, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30,
];

// ============================================================================
// KING SAFETY
// ============================================================================

/// Verifica se il Re ha arrocato controllando la sua posizione
///
/// Il Re è considerato "arrocato" se è nelle posizioni tipiche post-arrocco:
/// - Bianco: g1 (arrocco corto) o c1 (arrocco lungo)
/// - Nero: g8 (arrocco corto) o c8 (arrocco lungo)
///
/// # Argomenti
/// * `board` - La posizione da valutare
/// * `color` - Colore del Re da controllare
///
/// # Returns
/// `true` se il Re è in una posizione di arrocco, `false` altrimenti
fn has_castled(board: &Board, color: Color) -> bool {
    let king_sq = board.king_sq(color);

    match color {
        Color::White => {
            // Re bianco arrocato se in g1 (6) o c1 (2)
            king_sq == 6 || king_sq == 2
        }
        Color::Black => {
            // Re nero arrocato se in g8 (62) o c8 (58)
            king_sq == 62 || king_sq == 58
        }
    }
}

/// Conta i pedoni scudo davanti al Re
///
/// Controlla le 3 caselle immediatamente davanti al Re (sulla traversa successiva)
/// nelle colonne file-1, file, file+1 per contare quanti pedoni proteggono il Re.
///
/// # Argomenti
/// * `board` - La posizione da valutare
/// * `king_sq` - Indice della casella del Re (0-63)
/// * `color` - Colore del Re (e dei pedoni scudo)
///
/// # Returns
/// Numero di pedoni scudo (0-3)
fn count_pawn_shield(board: &Board, king_sq: usize, color: Color) -> i16 {
    let pawns = board.piece_bb(PieceKind::Pawn, color);
    let mut count = 0;

    let file = king_sq % 8;
    let rank = king_sq / 8;

    // Controlla 3 caselle davanti al Re (una traversa avanti)
    let shield_files = [file.saturating_sub(1), file, (file + 1).min(7)];
    let shield_rank = if color == Color::White {
        rank + 1
    } else {
        rank.saturating_sub(1)
    };

    // Evita di cercare pedoni fuori dalla scacchiera
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

/// Valuta il controllo del centro
///
/// Calcola uno score basato sul controllo delle caselle centrali:
/// - Centro: d4, e4, d5, e5 → +10 cp per casella controllata
/// - Centro esteso: c3-f3, c4, f4, c5, f5, c6-f6 → +3 cp per casella controllata
///
/// # Argomenti
/// * `board` - La posizione da valutare
///
/// # Returns
/// Score positivo = Bianco controlla centro, negativo = Nero
fn center_control(board: &Board) -> i16 {
    let mut score = 0;

    // Caselle del centro (d4, e4, d5, e5)
    const CENTER: [usize; 4] = [27, 28, 35, 36]; // d4, e4, d5, e5

    for &sq in &CENTER {
        let white_attacks = board.is_square_attacked(sq, Color::White);
        let black_attacks = board.is_square_attacked(sq, Color::Black);

        if white_attacks && !black_attacks {
            score += 10; // Bianco controlla
        } else if black_attacks && !white_attacks {
            score -= 10; // Nero controlla
        }
        // Se entrambi attaccano, si compensano (score += 0)
    }

    // Centro esteso (c3-f3, c4, f4, c5, f5, c6-f6)
    const EXTENDED: [usize; 12] = [
        18, 19, 20, 21, // c3, d3, e3, f3
        26, 29, // c4, f4
        34, 37, // c5, f5
        42, 43, 44, 45, // c6, d6, e6, f6
    ];

    for &sq in &EXTENDED {
        let white_attacks = board.is_square_attacked(sq, Color::White);
        let black_attacks = board.is_square_attacked(sq, Color::Black);

        if white_attacks && !black_attacks {
            score += 3; // Bianco controlla
        } else if black_attacks && !white_attacks {
            score -= 3; // Nero controlla
        }
    }

    score
}

/// Valuta la sicurezza del Re
///
/// Calcola uno score basato su:
/// 1. Penalità se il Re è al centro (colonne d,e) e non ha arrocato: -50 cp
/// 2. Bonus per ogni pedone scudo davanti al Re: +15 cp
///
/// # Argomenti
/// * `board` - La posizione da valutare
/// * `color` - Colore del Re da valutare
///
/// # Returns
/// Score positivo = Re sicuro, negativo = Re in pericolo
fn king_safety(board: &Board, color: Color) -> i16 {
    let mut safety = 0;

    // 1. Trova posizione Re
    let king_sq = board.king_sq(color);

    // 2. Penalità se Re al centro (files d,e = 3,4) prima di arroccare
    let file = king_sq % 8;
    if (file == 3 || file == 4) && !has_castled(board, color) {
        safety -= 50;
    }

    // 3. Bonus pedoni scudo (3x3 davanti al Re)
    let pawn_shield = count_pawn_shield(board, king_sq, color);
    safety += pawn_shield * 15;

    safety
}

// ============================================================================
// PENALITÀ SVILUPPO PEZZI MINORI
// ============================================================================

/// Calcola penalità per pezzi minori (Cavalieri, Alfieri) non sviluppati
///
/// Dopo la mossa 10, ogni cavaliere o alfiere sulla prima traversa
/// riceve una penalità di -10 cp.
///
/// Questo incentiva lo sviluppo attivo dei pezzi invece di mosse passive.
///
/// # Argomenti
/// * `board` - La posizione da valutare
/// * `color` - Colore per cui calcolare la penalità
///
/// # Returns
/// Penalità in centipawn per il colore specificato (sempre >= 0)
fn development_penalty(board: &Board, color: Color) -> i16 {
    // Prima di mossa 10, non applichiamo penalità (fase di apertura normale)
    if board.fullmove <= 10 {
        return 0;
    }

    let mut penalty = 0;

    match color {
        Color::White => {
            // Maschera per rank 1 (squares 0-7): bitboard con bit 0-7 settati
            const RANK_1_MASK: u64 = 0xFF; // 0b11111111

            // Cavalieri bianchi su rank 1
            let white_knights = board.piece_bb(PieceKind::Knight, Color::White);
            penalty += (white_knights & RANK_1_MASK).count_ones() as i16 * 10;

            // Alfieri bianchi su rank 1
            let white_bishops = board.piece_bb(PieceKind::Bishop, Color::White);
            penalty += (white_bishops & RANK_1_MASK).count_ones() as i16 * 10;
        }
        Color::Black => {
            // Maschera per rank 8 (squares 56-63): bitboard con bit 56-63 settati
            const RANK_8_MASK: u64 = 0xFF00_0000_0000_0000; // 0b11111111 << 56

            // Cavalieri neri su rank 8
            let black_knights = board.piece_bb(PieceKind::Knight, Color::Black);
            penalty += (black_knights & RANK_8_MASK).count_ones() as i16 * 10;

            // Alfieri neri su rank 8
            let black_bishops = board.piece_bb(PieceKind::Bishop, Color::Black);
            penalty += (black_bishops & RANK_8_MASK).count_ones() as i16 * 10;
        }
    }

    penalty
}

// ============================================================================
// FUNZIONE DI VALUTAZIONE PRINCIPALE
// ============================================================================

/// Quick material count only (no PSQT, no positional evaluation)
///
/// This is the fastest possible evaluation, counting only piece values
/// without considering position. Used as a threshold check in lazy evaluation.
///
/// # Arguments
/// * `board` - The position to evaluate
///
/// # Returns
/// Material balance in centipawns from the side-to-move perspective
fn quick_material_count(board: &Board) -> i16 {
    let mut white_material: i32 = 0;
    let mut black_material: i32 = 0;

    for sq in 0..64 {
        if let Some((kind, color)) = board.piece_on(sq) {
            let value = match kind {
                PieceKind::Pawn => PAWN_VALUE,
                PieceKind::Knight => KNIGHT_VALUE,
                PieceKind::Bishop => BISHOP_VALUE,
                PieceKind::Rook => ROOK_VALUE,
                PieceKind::Queen => QUEEN_VALUE,
                PieceKind::King => KING_VALUE,
            };

            match color {
                Color::White => white_material += value as i32,
                Color::Black => black_material += value as i32,
            }
        }
    }

    let relative = (white_material - black_material) as i16;
    if board.side == Color::Black {
        -relative
    } else {
        relative
    }
}

/// Lazy evaluation: fast material check with threshold, fallback to full eval
///
/// This is a "quick win" optimization that skips expensive positional evaluation
/// (king safety, development, center control) in clearly unbalanced positions.
///
/// Strategy:
/// 1. Quick material-only count (no PSQT lookup)
/// 2. If |material| > threshold (3 pawns), position is clearly won/lost → return material
/// 3. Otherwise, position is balanced → do full evaluation
///
/// # Arguments
/// * `board` - The position to evaluate
///
/// # Returns
/// Score in centipawns from the side-to-move perspective
///
/// # Performance
/// Expected ~10-20% speedup in tactical positions with material imbalances.
/// No slowdown in balanced positions (single extra material count is cheap).
pub fn evaluate_lazy(board: &Board) -> i16 {
    // Quick material-only evaluation
    let material = quick_material_count(board);

    // If position is clearly unbalanced (> 3 pawns advantage), skip expensive eval
    // Rationale: In such positions, king safety and development matter less than raw material
    const LAZY_THRESHOLD: i16 = 300; // 3 pawns (adjustable based on benchmarks)

    if material.abs() > LAZY_THRESHOLD {
        return material;
    }

    // Otherwise do full evaluation (material + PSQT + king safety + development + center)
    evaluate(board)
}

/// Fast evaluation: only material + PSQT (no king safety, development, etc.)
/// Used in quiescence search where speed is critical
pub fn evaluate_fast(board: &Board) -> i16 {
    let mut white_score: i32 = 0;
    let mut black_score: i32 = 0;

    for sq in 0..64 {
        if let Some((kind, color)) = board.piece_on(sq) {
            let material_value = match kind {
                PieceKind::Pawn => PAWN_VALUE,
                PieceKind::Knight => KNIGHT_VALUE,
                PieceKind::Bishop => BISHOP_VALUE,
                PieceKind::Rook => ROOK_VALUE,
                PieceKind::Queen => QUEEN_VALUE,
                PieceKind::King => KING_VALUE,
            };

            let psqt_index = match color {
                Color::White => sq,
                Color::Black => sq ^ 56,
            };

            let psqt_bonus = match kind {
                PieceKind::Pawn => PAWN_PSQT[psqt_index],
                PieceKind::Knight => KNIGHT_PSQT[psqt_index],
                PieceKind::Bishop => BISHOP_PSQT[psqt_index],
                PieceKind::Rook => ROOK_PSQT[psqt_index],
                PieceKind::Queen => QUEEN_PSQT[psqt_index],
                PieceKind::King => KING_PSQT[psqt_index],
            };

            let total_value = material_value as i32 + psqt_bonus as i32;

            match color {
                Color::White => white_score += total_value,
                Color::Black => black_score += total_value,
            }
        }
    }

    let relative_score = (white_score - black_score) as i16;

    if board.side == Color::Black {
        -relative_score
    } else {
        relative_score
    }
}

/// Valuta la posizione con materiale + PSQT + penalità sviluppo
///
/// Restituisce uno score dal punto di vista del **giocatore che muove** (convenzione negamax).
/// Positivo = buono per chi muove, negativo = cattivo per chi muove.
///
/// # Argomenti
/// * `board` - La posizione da valutare
///
/// # Returns
/// Score in centipawn dal punto di vista del side-to-move
pub fn evaluate(board: &Board) -> i16 {
    let mut white_score: i32 = 0;
    let mut black_score: i32 = 0;

    // Iterate directly on bitboards (much faster than piece_on() for each square)
    // This reduces 64*12 checks to ~30-40 actual piece lookups
    let piece_kinds = [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ];

    for &kind in &piece_kinds {
        let material_value = match kind {
            PieceKind::Pawn => PAWN_VALUE,
            PieceKind::Knight => KNIGHT_VALUE,
            PieceKind::Bishop => BISHOP_VALUE,
            PieceKind::Rook => ROOK_VALUE,
            PieceKind::Queen => QUEEN_VALUE,
            PieceKind::King => KING_VALUE,
        };

        let psqt_table = match kind {
            PieceKind::Pawn => &PAWN_PSQT,
            PieceKind::Knight => &KNIGHT_PSQT,
            PieceKind::Bishop => &BISHOP_PSQT,
            PieceKind::Rook => &ROOK_PSQT,
            PieceKind::Queen => &QUEEN_PSQT,
            PieceKind::King => &KING_PSQT,
        };

        // White pieces
        let mut white_bb = board.piece_bb(kind, Color::White);
        while white_bb != 0 {
            let sq = white_bb.trailing_zeros() as usize;
            white_bb &= white_bb - 1; // Clear LSB
            let psqt_bonus = psqt_table[sq];
            white_score += material_value as i32 + psqt_bonus as i32;
        }

        // Black pieces (flip vertically for PSQT index)
        let mut black_bb = board.piece_bb(kind, Color::Black);
        while black_bb != 0 {
            let sq = black_bb.trailing_zeros() as usize;
            black_bb &= black_bb - 1; // Clear LSB
            let psqt_index = sq ^ 56; // Flip verticale
            let psqt_bonus = psqt_table[psqt_index];
            black_score += material_value as i32 + psqt_bonus as i32;
        }
    }

    // Applica penalità per pezzi minori non sviluppati (separatamente per colore)
    let white_penalty = development_penalty(board, Color::White);
    let black_penalty = development_penalty(board, Color::Black);

    white_score -= white_penalty as i32;
    black_score -= black_penalty as i32;

    // King Safety: valuta sicurezza del Re per entrambi i colori
    let white_king_safety = king_safety(board, Color::White);
    let black_king_safety = king_safety(board, Color::Black);

    white_score += white_king_safety as i32;
    black_score += black_king_safety as i32;

    // Center Control: valuta controllo delle caselle centrali
    // (restituisce valore già dal punto di vista Bianco - Nero)
    let center = center_control(board);

    // Calcola lo score relativo (Bianco - Nero)
    let relative_score = (white_score - black_score) as i16 + center;

    // CRITICAL: Convenzione negamax - ritorna dal punto di vista del side-to-move
    // Se è il Nero a muovere, nego lo score (positivo = buono per il Nero)
    if board.side == Color::Black {
        -relative_score
    } else {
        relative_score
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

        // Posizione iniziale dovrebbe essere circa pari (±100 cp per PSQT asimmetrie minori)
        assert!(
            score.abs() < 100,
            "Startpos dovrebbe essere bilanciata, ma score = {score}"
        );
    }

    #[test]
    fn test_evaluate_side_to_move_symmetry() {
        // Testa che la valutazione venga correttamente negata per il Nero
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

        // Gli score devono essere opposti (convezione negamax)
        assert_eq!(
            score_white, -score_black,
            "Negamax convention: white={score_white}, black={score_black}"
        );
    }

    #[test]
    fn test_evaluate_central_pawn_bonus() {
        // Testa che un pedone centrale (e4) abbia bonus rispetto a un pedone laterale
        let mut board_central = Board::new();
        board_central
            .set_from_fen("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1")
            .unwrap();

        let mut board_edge = Board::new();
        board_edge
            .set_from_fen("4k3/8/8/8/P7/8/8/4K3 w - - 0 1")
            .unwrap();

        let score_central = evaluate(&board_central);
        let score_edge = evaluate(&board_edge);

        // Il pedone centrale dovrebbe avere score migliore
        assert!(
            score_central > score_edge,
            "Pedone centrale (e4) dovrebbe essere meglio di pedone laterale (a4): central={score_central}, edge={score_edge}"
        );
    }

    #[test]
    fn test_evaluate_knight_center_bonus() {
        // Testa che un cavaliere centrale abbia bonus rispetto a un cavaliere sul bordo
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

        // Il cavaliere centrale dovrebbe avere score migliore
        assert!(
            score_central > score_rim,
            "Cavaliere centrale (e4) dovrebbe essere meglio di cavaliere sul bordo (a4): central={score_central}, rim={score_rim}"
        );
    }

    #[test]
    fn test_evaluate_castled_king_bonus() {
        // Testa che il re dopo arrocco corto (g1) abbia bonus rispetto al re centrale
        let mut board_castled = Board::new();
        board_castled
            .set_from_fen("r3k2r/8/8/8/8/8/8/R4RK1 w kq - 0 1")
            .unwrap();

        let mut board_center = Board::new();
        board_center
            .set_from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
            .unwrap();

        let score_castled = evaluate(&board_castled);
        let score_center = evaluate(&board_center);

        // Il re dopo arrocco dovrebbe avere score migliore
        assert!(
            score_castled > score_center,
            "Re dopo arrocco (g1) dovrebbe essere più sicuro del re centrale (e1): castled={score_castled}, center={score_center}"
        );
    }

    #[test]
    fn test_king_safety_center_penalty() {
        // Re al centro (e1) non arrocato dovrebbe ricevere penalità -50 cp
        let mut board = Board::new();
        board.set_from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        let white_safety = king_safety(&board, Color::White);
        let black_safety = king_safety(&board, Color::Black);

        // Entrambi i Re sono al centro (e1, e8) e non arrocati: -50 cp
        assert_eq!(
            white_safety, -50,
            "Re bianco in e1 non arrocato dovrebbe avere -50 cp"
        );
        assert_eq!(
            black_safety, -50,
            "Re nero in e8 non arrocato dovrebbe avere -50 cp"
        );
    }

    #[test]
    fn test_king_safety_castled_with_pawns() {
        // Re arrocato corto con pedoni scudo completi: +45 cp (3 pedoni × 15)
        let mut board = Board::new();
        board
            .set_from_fen("4k3/8/8/8/8/8/5PPP/6K1 w - - 0 1")
            .unwrap();

        let white_safety = king_safety(&board, Color::White);

        // Re in g1 (arrocato), 3 pedoni in f2,g2,h2: +45 cp
        assert_eq!(
            white_safety, 45,
            "Re arrocato con 3 pedoni scudo dovrebbe avere +45 cp (3×15)"
        );
    }

    #[test]
    fn test_king_safety_castled_no_shield() {
        // Re arrocato ma senza pedoni scudo: 0 cp (né penalità né bonus)
        let mut board = Board::new();
        board.set_from_fen("4k3/8/8/8/8/8/8/6K1 w - - 0 1").unwrap();

        let white_safety = king_safety(&board, Color::White);

        // Re in g1 (arrocato), nessun pedone scudo: 0 cp
        assert_eq!(
            white_safety, 0,
            "Re arrocato senza pedoni scudo dovrebbe avere 0 cp"
        );
    }

    #[test]
    fn test_king_safety_partial_shield() {
        // Re arrocato con 2 pedoni scudo: +30 cp
        let mut board = Board::new();
        board
            .set_from_fen("4k3/8/8/8/8/8/5PP1/6K1 w - - 0 1")
            .unwrap();

        let white_safety = king_safety(&board, Color::White);

        // Re in g1 (arrocato), 2 pedoni in f2,g2: +30 cp (2×15)
        assert_eq!(
            white_safety, 30,
            "Re arrocato con 2 pedoni scudo dovrebbe avere +30 cp (2×15)"
        );
    }

    #[test]
    fn test_king_safety_queenside_castled() {
        // Re arrocato lungo (c1) con pedoni scudo
        let mut board = Board::new();
        board
            .set_from_fen("4k3/8/8/8/8/8/1PPP4/2K5 w - - 0 1")
            .unwrap();

        let white_safety = king_safety(&board, Color::White);

        // Re in c1 (arrocato lungo), 3 pedoni in b2,c2,d2: +45 cp
        assert_eq!(
            white_safety, 45,
            "Re arrocato lungo con 3 pedoni scudo dovrebbe avere +45 cp"
        );
    }
}
