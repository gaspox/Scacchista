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

const SIMPLE_ENDGAME_BONUS: i16 = 10000;

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

// ============================================================================
// PIECE-SQUARE TABLES (dal punto di vista del BIANCO)
// ============================================================================
// Indici: rank 0 = prima traversa (A1..H1), rank 7 = ottava traversa (A8..H8)
// Per il Nero, specchiamo verticalmente: flip_sq = sq XOR 56

/// PSQT per i pedoni (MIGLIORATO - Fix GrandMaster #5)
/// Incentiva:
/// - Pedoni centrali (e4, d4, e5, d5): +20..+30 cp
/// - Avanzamento controllato
/// - **NUOVO**: Pedoni passati avanzati (rank 6/7) → +100/+200 cp (era +20/+35)
/// - Motivazione: L'engine non capiva il valore dei pedoni avanzati vicini alla promozione
const PAWN_PSQT: [i16; 64] = [
    // Rank 1 (impossibile per i pedoni, ma per simmetria)
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 2 (pedoni iniziali)
    5, 5, 5, 5, 5, 5, 5, 5, // Rank 3
    5, 5, 10, 15, 15, 10, 5, 5, // Rank 4
    10, 10, 20, 30, 30, 20, 10, 10, // Rank 5
    50, 50, 70, 90, 90, 70, 50,
    50, // Rank 6 (molto avanzati) - AUMENTATO: +50/+90 cp (era +20/+30)
    120, 120, 150, 180, 180, 150, 120,
    120, // Rank 7 (vicini alla promozione) - AUMENTATO: +120/+180 cp (era +25/+35)
    300, 300, 350, 400, 400, 350, 300,
    300, // Rank 8 (impossibile, ma teoricamente +300/+400 cp)
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

/// Conta pezzi minori e maggiori attivi (N, B, R, Q) per un colore
///
/// Usato per calcolare il pericolo per il Re avversario: più pezzi attaccanti
/// sono in gioco, più è pericoloso avere il Re esposto al centro.
///
/// # Argomenti
/// * `board` - La posizione da valutare
/// * `color` - Colore di cui contare i pezzi
///
/// # Returns
/// Numero di pezzi minori e maggiori (esclusi pedoni e Re)
fn count_active_pieces(board: &Board, color: Color) -> i16 {
    let mut count = 0;
    count += board.piece_bb(PieceKind::Knight, color).count_ones() as i16;
    count += board.piece_bb(PieceKind::Bishop, color).count_ones() as i16;
    count += board.piece_bb(PieceKind::Rook, color).count_ones() as i16;
    count += board.piece_bb(PieceKind::Queen, color).count_ones() as i16;
    count
}

/// Verifica se il Re ha ancora diritto di arrocco (corto o lungo)
///
/// Controlla i bit del campo board.castling:
/// - White: bit 3 (K) o bit 2 (Q) → 0b1100
/// - Black: bit 1 (k) o bit 0 (q) → 0b0011
///
/// # Argomenti
/// * `board` - La posizione da valutare
/// * `color` - Colore del Re da controllare
///
/// # Returns
/// `true` se il Re può ancora arroccare (corto o lungo), `false` altrimenti
fn has_castling_rights(board: &Board, color: Color) -> bool {
    match color {
        Color::White => (board.castling & 0b1100u8) != 0, // K o Q
        Color::Black => (board.castling & 0b0011u8) != 0, // k o q
    }
}

/// Verifica se il Re è sotto scacco
///
/// Un Re è sotto scacco se la sua casella è attaccata da pezzi avversari.
///
/// # Argomenti
/// * `board` - La posizione da valutare
/// * `color` - Colore del Re da controllare
///
/// # Returns
/// `true` se il Re è sotto scacco, `false` altrimenti
fn is_in_check(board: &Board, color: Color) -> bool {
    let king_sq = board.king_sq(color);
    let opponent_color = match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    board.is_square_attacked(king_sq, opponent_color)
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

/// Valuta la sicurezza del Re (MIGLIORATO - Fix GrandMaster #1)
///
/// Calcola uno score basato su:
/// 1. **NUOVO**: Penalità severa se ha perso diritto arrocco in apertura: -70 cp
/// 2. **MIGLIORATO**: Penalità dinamica per Re al centro, proporzionale a pezzi avversari
/// 3. Bonus per ogni pedone scudo davanti al Re: +15 cp
///
/// # Motivazione (da analisi prova_2.pgn)
/// - Mossa 13.Qxe7?? → Qxe7+ ha perso il diritto di arrocco in apertura
/// - L'engine non ha capito che è catastrofico (vecchia penalità: solo -50 cp fissi)
/// - Re esposto con tanti pezzi avversari attivi è molto più pericoloso
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
    let file = king_sq % 8;

    // 2. Verifica se Re ha arrocato
    let castled = has_castled(board, color);

    // 3. Verifica se Re ha ancora diritto di arrocco
    let has_rights = has_castling_rights(board, color);

    // 4. NUOVO: Penalità pesante per perdita diritto arrocco in apertura
    // Se siamo in apertura (fullmove 5-14), re non ha arrocato, e ha perso diritti → GRAVE
    // Nota: fullmove >= 5 evita falsi positivi su FEN personalizzate nelle prime mosse
    if board.fullmove >= 5 && board.fullmove < 15 && !castled && !has_rights {
        // Re ha perso diritto arrocco in apertura senza aver arrocato
        // Es: 13.Qxe7?? in prova_2.pgn → Qxe7+ e re bloccato in e1
        // FIX v0.4.1: Aumentata da -120 a -400 cp (approssimazione Desperado mode)
        // La perdita dell'arrocco in apertura è CATASTROFICA e deve dominare
        // il vantaggio materiale di +330 cp (Donna vs Alfiere)
        safety -= 400; // Penalità catastrofica (> valore di un pezzo minore)
    }

    // 4.5 NUOVO (Fix Issue #1): Threat evaluation - Re sotto scacco che perderà arrocco
    // Questa è una "penalità preventiva" che si applica PRIMA che il re perda i diritti
    // Se il re è sotto scacco E ha ancora diritti di arrocco → molto pericoloso
    // Esempio: Dopo 13.Qxe7 Qxe7+, il re bianco è sotto scacco e dovrà muoversi
    // perdendo i diritti di arrocco. Questa penalità previene Qxe7.
    if is_in_check(board, color) && has_rights {
        // Penalità preventiva: il re è sotto scacco e probabilmente perderà arrocco
        // AUMENTATA a -200 cp (circa 2/3 di pezzo minore) dopo test pratico
        // -80 cp non era sufficiente per prevenire Qxe7
        safety -= 200;
    }

    // 5. MIGLIORATO: Penalità per Re al centro, proporzionale a pezzi avversari
    if (file == 3 || file == 4) && !castled {
        // Conta pezzi avversari attivi (più pezzi = re più in pericolo)
        let opponent_color = match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
        let active_pieces = count_active_pieces(board, opponent_color);

        // Penalità base -50, aumentata in base a pezzi avversari
        // Formula: -50 * (1 + active_pieces / 6)
        // Es: 4 pezzi avversari → -50 * (1 + 4/6) ≈ -83 cp
        //     8 pezzi avversari → -50 * (1 + 8/6) ≈ -117 cp
        let base_penalty = 50;
        let multiplier = 100 + (active_pieces * 16); // 100 = 1.0, 16 ≈ 1/6 scaled to percentage
        safety -= (base_penalty * multiplier / 100) as i16;
    }

    // 6. Bonus pedoni scudo (invariato)
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
    if let Some(bonus) = simple_endgame_bonus(board) {
        return bonus;
    }

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
///
/// NOTE: Include CRITICAL king safety penalties (castling rights loss in opening)
/// to avoid catastrophic blunders in tactical lines
pub fn evaluate_fast(board: &Board) -> i16 {
    if let Some(bonus) = simple_endgame_bonus(board) {
        return bonus;
    }

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

    // CRITICAL: Apply catastrophic penalty for losing castling rights in opening
    // This is essential to prevent blunders like Qxe7?? (Issue #1)
    // We only add this penalty, not full king_safety(), to keep evaluate_fast fast
    let white_king_penalty = king_safety_critical_only(board, Color::White);
    let black_king_penalty = king_safety_critical_only(board, Color::Black);

    white_score += white_king_penalty as i32;
    black_score += black_king_penalty as i32;

    let relative_score = (white_score - black_score) as i16;

    if board.side == Color::Black {
        -relative_score
    } else {
        relative_score
    }
}

/// Fast version of king_safety: only catastrophic penalties (no pawn shield, no dynamic penalties)
/// Used in evaluate_fast() to keep quiescence search fast but avoid critical blunders
fn king_safety_critical_only(board: &Board, color: Color) -> i16 {
    let mut safety = 0;

    // Check if king has castled
    let king_sq = board.king_sq(color);
    let castled = has_castled(board, color);

    // Check if king has castling rights
    let has_rights = has_castling_rights(board, color);

    // CRITICAL PENALTY: Lost castling rights in opening without castling
    if board.fullmove >= 5 && board.fullmove < 15 && !castled && !has_rights {
        safety -= 400; // Catastrophic penalty
    }

    // CRITICAL PENALTY: King in check with castling rights (threat evaluation)
    if is_in_check(board, color) && has_rights {
        safety -= 200; // Preventive penalty
    }

    safety
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
    if let Some(bonus) = simple_endgame_bonus(board) {
        return bonus;
    }

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
            .set_from_fen("rnbq1rk1/pppp1ppp/4pn2/3P4/4P3/5N2/PPP2PPP/RNBQ1RK1 w - - 0 1")
            .unwrap();
        let score_castled = evaluate(&board_castled);

        let mut board_uncastled = Board::new();
        board_uncastled
            .set_from_fen("rnbq1rk1/pppp1ppp/4pn2/3P4/4P3/5N2/PPP2PPP/RNBQK2R w - - 0 1")
            .unwrap();
        let score_uncastled = evaluate(&board_uncastled);

        assert!(score_castled > score_uncastled);
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

    #[test]
    fn test_king_safety_lost_castling_rights_in_opening() {
        // Test basato su prova_2.pgn dopo 13.Qxe7 Qxe7+ 14.Kf1
        // Re bianco ha perso diritto arrocco in apertura (mossa 14) → penalità severa
        let mut board = Board::new();
        board
            .set_from_fen("r5k1/pp2qppp/1n1p4/2pPb3/2P1P3/2N2N2/PP2BPPP/R1B2K1R b - - 1 14")
            .unwrap();

        let white_safety = king_safety(&board, Color::White);

        // Re in f1 (file 5, non centro), fullmove=14, no castling rights, not castled
        // - Penalità -120 per perdita diritto arrocco in apertura (FIX v0.4.0: era -70)
        // - NO penalità dinamica (re in f1, non d1/e1)
        // - Bonus pedoni scudo: probabilmente 2-3 pedoni → +30/+45 cp
        // Totale atteso: circa -120 + 30/45 = -90/-75 cp
        assert!(
            white_safety < -300,
            "Re che ha perso diritto arrocco in apertura (mossa 14) dovrebbe avere penalità severa: white_safety={white_safety}"
        );

        // Confronto: se il re avesse ancora diritti di arrocco, la penalità sarebbe minore
        let mut board_with_rights = Board::new();
        board_with_rights
            .set_from_fen("r5k1/pp2qppp/1n1p4/2pPb3/2P1P3/2N2N2/PP2BPPP/R1B2K1R b KQ - 1 14")
            .unwrap();

        let white_safety_with_rights = king_safety(&board_with_rights, Color::White);

        // Con diritti di arrocco, non si applica la penalità -70
        assert!(
            white_safety_with_rights > white_safety,
            "Re con diritti di arrocco dovrebbe essere più sicuro: with_rights={white_safety_with_rights}, without={white_safety}"
        );
    }

    #[test]
    fn test_king_safety_center_with_active_pieces() {
        // Re al centro (e1) con molti pezzi avversari attivi → penalità aumentata
        let mut board = Board::new();
        board
            .set_from_fen("rnbq1rk1/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQ - 0 10")
            .unwrap();

        let white_safety = king_safety(&board, Color::White);

        // Re in e1 (centro), non arrocato, fullmove=10, ha diritti di arrocco (KQ)
        // Pezzi neri attivi dalla FEN: 1N + 1B + 2R + 1Q = 5 pezzi
        // Penalità dinamica: -50 * (1 + 5/6) ≈ -50 * 1.83 ≈ -92 cp
        // Bonus pedoni scudo: 3 pedoni (d2,e2,f2) → +45 cp
        // Totale atteso: -92 + 45 = -47 cp (circa -45)
        // NO penalità -70 (ha ancora diritti di arrocco KQ)
        assert!(
            white_safety < -30 && white_safety > -60,
            "Re al centro con 5 pezzi avversari attivi: white_safety={white_safety}"
        );

        // Confronto: re al centro con pochi pezzi avversari ma stessi pedoni scudo
        let mut board_few_pieces = Board::new();
        board_few_pieces
            .set_from_fen("4k3/8/8/8/8/8/PPPPPPPP/RNBQK2R w KQ - 0 10")
            .unwrap();

        let white_safety_few = king_safety(&board_few_pieces, Color::White);

        // Con 0 pezzi avversari: penalità -50 * (1 + 0/6) = -50, bonus +45 pedoni = -5 cp
        assert!(
            white_safety_few > -20 && white_safety_few < 10,
            "Re al centro con 0 pezzi avversari: white_safety_few={white_safety_few}"
        );

        // Re al centro con molti pezzi deve essere MENO sicuro (più negativo)
        assert!(
            white_safety < white_safety_few,
            "Re con molti pezzi avversari più in pericolo: many={white_safety}, few={white_safety_few}"
        );
    }
}
