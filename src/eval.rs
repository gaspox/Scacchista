//! Modulo di valutazione - Piece-Square Tables (PSQT)
//!
//! Le PSQT funzionano come una "mappa di calore" della scacchiera:
//! ogni casella ha un valore bonus/malus che incentiva posizioni strategicamente
//! migliori (es: pedoni centrali, cavalieri sviluppati, re protetto dopo arrocco).

use crate::board::{Board, Color, PieceKind};
use crate::utils::{
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
};

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

/// PSQT per il re (Middlegame)
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
// ENDGAME PSQT (EG) - dal punto di vista del BIANCO
// ============================================================================
// I valori EG differiscono dal MG perché la strategia cambia in finale:
// - Re diventa aggressivo (bonus centro)
// - Pedoni avanzati sono decisivi (bonus molto più alto)
// - Pezzi minori ai bordi sono meno penalizzati

const PAWN_PSQT_EG: [i16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    10, 10, 10, 10, 10, 10, 10, 10,
    15, 15, 20, 25, 25, 20, 15, 15,
    25, 25, 35, 45, 45, 35, 25, 25,
    60, 60, 80, 100, 100, 80, 60, 60,
    150, 150, 180, 220, 220, 180, 150, 150,
    300, 300, 350, 400, 400, 350, 300, 300,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_PSQT_EG: [i16; 64] = [
    -40, -30, -20, -20, -20, -20, -30, -40,
    -30, -10, 10, 15, 15, 10, -10, -30,
    -20, 10, 20, 25, 25, 20, 10, -20,
    -20, 15, 25, 30, 30, 25, 15, -20,
    -20, 10, 25, 30, 30, 25, 10, -20,
    -20, 10, 20, 25, 25, 20, 10, -20,
    -30, -10, 10, 15, 15, 10, -10, -30,
    -40, -30, -20, -20, -20, -20, -30, -40,
];

const BISHOP_PSQT_EG: [i16; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10, 10, 5, 5, 5, 5, 10, -10,
    -10, 15, 15, 15, 15, 15, 15, -10,
    -10, 10, 15, 20, 20, 15, 10, -10,
    -10, 10, 15, 20, 20, 15, 10, -10,
    -10, 10, 15, 15, 15, 15, 10, -10,
    -10, 10, 5, 5, 5, 5, 10, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_PSQT_EG: [i16; 64] = [
    0, 0, 0, 5, 5, 0, 0, 0,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    10, 15, 15, 15, 15, 15, 15, 10,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const QUEEN_PSQT_EG: [i16; 64] = [
    -10, -5, -5, 0, 0, -5, -5, -10,
    -5, 5, 10, 5, 5, 5, 5, -5,
    -5, 10, 10, 10, 10, 10, 5, -5,
    0, 5, 10, 10, 10, 10, 5, 0,
    0, 5, 10, 10, 10, 10, 5, 0,
    -5, 5, 10, 10, 10, 10, 5, -5,
    -5, 5, 5, 5, 5, 5, 5, -5,
    -10, -5, -5, 0, 0, -5, -5, -10,
];

const KING_PSQT_EG: [i16; 64] = [
    -50, -30, -20, -10, -10, -20, -30, -50,
    -30, -10, 0, 10, 10, 0, -10, -30,
    -20, 0, 10, 20, 20, 10, 0, -20,
    -10, 10, 20, 30, 30, 20, 10, -10,
    -10, 10, 20, 30, 30, 20, 10, -10,
    -20, 0, 10, 20, 20, 10, 0, -20,
    -30, -10, 0, 10, 10, 0, -10, -30,
    -50, -30, -20, -10, -10, -20, -30, -50,
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
        safety -= base_penalty * multiplier / 100;
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

fn endgame_score(board: &Board) -> Option<i16> {
    let wc = material_counts(board, Color::White);
    let bc = material_counts(board, Color::Black);

    // Existing simple signatures (KQ/KR/KNB vs K)
    for signature in &SIMPLE_ENDGAME_SIGNATURES {
        if wc == *signature && bc.is_empty() {
            return Some(simple_endgame_score(Color::White, board.side));
        }
        if bc == *signature && wc.is_empty() {
            return Some(simple_endgame_score(Color::Black, board.side));
        }
    }

    let k = MaterialCounts {
        pawns: 0,
        knights: 0,
        bishops: 0,
        rooks: 0,
        queens: 0,
    };
    let kq = MaterialCounts {
        pawns: 0,
        knights: 0,
        bishops: 0,
        rooks: 0,
        queens: 1,
    };
    let kr = MaterialCounts {
        pawns: 0,
        knights: 0,
        bishops: 0,
        rooks: 1,
        queens: 0,
    };
    let kbb = MaterialCounts {
        pawns: 0,
        knights: 0,
        bishops: 2,
        rooks: 0,
        queens: 0,
    };
    let kbp = MaterialCounts {
        pawns: 1,
        knights: 0,
        bishops: 1,
        rooks: 0,
        queens: 0,
    };
    let krp = MaterialCounts {
        pawns: 1,
        knights: 0,
        bishops: 0,
        rooks: 1,
        queens: 0,
    };
    let krn = MaterialCounts {
        pawns: 0,
        knights: 1,
        bishops: 0,
        rooks: 1,
        queens: 0,
    };
    let krb = MaterialCounts {
        pawns: 0,
        knights: 0,
        bishops: 1,
        rooks: 1,
        queens: 0,
    };
    let kp = MaterialCounts {
        pawns: 1,
        knights: 0,
        bishops: 0,
        rooks: 0,
        queens: 0,
    };

    // KBB vs K (theoretical mate)
    if (wc == kbb && bc == k) || (bc == kbb && wc == k) {
        let attacker = if wc == kbb { Color::White } else { Color::Black };
        return Some(simple_endgame_score(attacker, board.side));
    }

    // KQ vs KR (easy win)
    if (wc == kq && bc == kr) || (bc == kq && wc == kr) {
        let attacker = if wc == kq { Color::White } else { Color::Black };
        return Some(simple_endgame_score(attacker, board.side));
    }

    // KQ vs KP (easy win)
    if (wc == kq && bc == kp) || (bc == kq && wc == kp) {
        let attacker = if wc == kq { Color::White } else { Color::Black };
        return Some(simple_endgame_score(attacker, board.side));
    }

    // KRN vs KR, KRB vs KR (usually winning)
    if (wc == krn && bc == kr) || (bc == krn && wc == kr) {
        let attacker = if wc == krn { Color::White } else { Color::Black };
        return Some(simple_endgame_score(attacker, board.side));
    }
    if (wc == krb && bc == kr) || (bc == krb && wc == kr) {
        let attacker = if wc == krb { Color::White } else { Color::Black };
        return Some(simple_endgame_score(attacker, board.side));
    }

    // KRP vs KR: win if pawn is advanced (rank >= 5 for white, rank <= 4 for black)
    if (wc == krp && bc == kr) || (bc == krp && wc == kr) {
        let attacker = if wc == krp { Color::White } else { Color::Black };
        let pawns = board.piece_bb(PieceKind::Pawn, attacker);
        let advanced = if attacker == Color::White {
            pawns & (crate::utils::RANK_5 | crate::utils::RANK_6 | crate::utils::RANK_7) != 0
        } else {
            pawns & (crate::utils::RANK_2 | crate::utils::RANK_3 | crate::utils::RANK_4) != 0
        };
        if advanced {
            return Some(simple_endgame_score(attacker, board.side));
        }
    }

    // KBP vs K: generally winning (simplified - ignores wrong-color-rook-pawn corners)
    if (wc == kbp && bc == k) || (bc == kbp && wc == k) {
        let attacker = if wc == kbp { Color::White } else { Color::Black };
        return Some(simple_endgame_score(attacker, board.side));
    }

    None
}



/// Fast evaluation: only material + PSQT (no king safety, development, etc.)
/// Used in quiescence search where speed is critical
///
/// Uses bitboard bit-scan (trailing_zeros) to iterate only over occupied squares.
/// This is ~30-40x fewer iterations than the naive 64-square loop.
///
/// NOTE: Include CRITICAL king safety penalties (castling rights loss in opening)
/// to avoid catastrophic blunders in tactical lines
pub fn evaluate_fast(board: &Board) -> i16 {
    if let Some(bonus) = endgame_score(board) {
        return bonus;
    }

    let mut white_score: i32 = 0;
    let mut black_score: i32 = 0;

    // Helper: iterate over set bits in a bitboard, accumulating material + PSQT
    // For White: PSQT index = sq (a1=0 is bottom-left)
    // For Black: PSQT index = sq ^ 56 (mirror vertically)

    // --- White pieces ---
    let pieces_and_tables: [(PieceKind, i16, &[i16; 64]); 6] = [
        (PieceKind::Pawn, PAWN_VALUE, &PAWN_PSQT),
        (PieceKind::Knight, KNIGHT_VALUE, &KNIGHT_PSQT),
        (PieceKind::Bishop, BISHOP_VALUE, &BISHOP_PSQT),
        (PieceKind::Rook, ROOK_VALUE, &ROOK_PSQT),
        (PieceKind::Queen, QUEEN_VALUE, &QUEEN_PSQT),
        (PieceKind::King, KING_VALUE, &KING_PSQT),
    ];

    for &(kind, material_value, psqt) in &pieces_and_tables {
        let mut bb = board.piece_bb(kind, Color::White);
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            white_score += material_value as i32 + psqt[sq] as i32;
            bb &= bb - 1; // Clear lowest set bit
        }
    }

    // --- Black pieces ---
    for &(kind, material_value, psqt) in &pieces_and_tables {
        let mut bb = board.piece_bb(kind, Color::Black);
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            black_score += material_value as i32 + psqt[sq ^ 56] as i32; // Mirror for Black
            bb &= bb - 1; // Clear lowest set bit
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
    let _king_sq = board.king_sq(color);
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

// ============================================================================
// PAWN STRUCTURE & BISHOP PAIR
// ============================================================================

const FILE_MASKS: [u64; 8] = [
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
];

/// Bonus per la coppia di alfieri (+30 cp)
fn bishop_pair(board: &Board, color: Color) -> i16 {
    if board.piece_bb(PieceKind::Bishop, color).count_ones() >= 2 {
        30
    } else {
        0
    }
}

/// Penalità per pedoni doppi (20 cp per ogni pedone extra sullo stesso file)
fn doubled_pawns(board: &Board, color: Color) -> i16 {
    let pawns = board.piece_bb(PieceKind::Pawn, color);
    let mut penalty = 0i16;
    for &mask in &FILE_MASKS {
        let count = (pawns & mask).count_ones() as i16;
        if count > 1 {
            penalty += (count - 1) * 20;
        }
    }
    penalty
}

/// Penalità per pedoni isolati (15 cp ciascuno)
fn isolated_pawns(board: &Board, color: Color) -> i16 {
    let pawns = board.piece_bb(PieceKind::Pawn, color);
    let mut penalty = 0i16;
    let mut bb = pawns;
    while bb != 0 {
        let sq = bb.trailing_zeros() as usize;
        bb &= bb - 1;
        let file = sq % 8;
        let neighbor_mask = if file == 0 {
            FILE_MASKS[1]
        } else if file == 7 {
            FILE_MASKS[6]
        } else {
            FILE_MASKS[file - 1] | FILE_MASKS[file + 1]
        };
        if pawns & neighbor_mask == 0 {
            penalty += 15;
        }
    }
    penalty
}

/// Bonus per pedoni passati (progressivo per rank)
fn passed_pawns(board: &Board, color: Color) -> i16 {
    let my_pawns = board.piece_bb(PieceKind::Pawn, color);
    let their_pawns = board.piece_bb(
        PieceKind::Pawn,
        match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        },
    );
    let mut bonus = 0i16;
    let mut bb = my_pawns;
    while bb != 0 {
        let sq = bb.trailing_zeros() as usize;
        bb &= bb - 1;
        let file = sq % 8;
        let rank = sq / 8;

        // Build mask of squares in front (same + adjacent files) that could block/capture
        let mut front_mask = 0u64;
        if color == Color::White {
            for r in (rank + 1)..8 {
                front_mask |= FILE_MASKS[file] << (r * 8);
                if file > 0 {
                    front_mask |= FILE_MASKS[file - 1] << (r * 8);
                }
                if file < 7 {
                    front_mask |= FILE_MASKS[file + 1] << (r * 8);
                }
            }
        } else {
            for r in 0..rank {
                front_mask |= FILE_MASKS[file] << (r * 8);
                if file > 0 {
                    front_mask |= FILE_MASKS[file - 1] << (r * 8);
                }
                if file < 7 {
                    front_mask |= FILE_MASKS[file + 1] << (r * 8);
                }
            }
        }

        if their_pawns & front_mask == 0 {
            let rank_bonus = if color == Color::White {
                match rank {
                    3 => 20,
                    4 => 40,
                    5 => 80,
                    6 => 150,
                    _ => 0,
                }
            } else {
                match rank {
                    1 => 150,
                    2 => 80,
                    3 => 40,
                    4 => 20,
                    _ => 0,
                }
            };
            bonus += rank_bonus;
        }
    }
    bonus
}

/// Bonus per mobilità dei pezzi (cavallo +4, alfiere +3, torre +2, donna +1 per casella)
fn mobility(board: &Board, color: Color) -> i16 {
    let own_occ = match color {
        Color::White => board.white_occ,
        Color::Black => board.black_occ,
    };
    let mut bonus = 0i16;

    let mut knights = board.piece_bb(PieceKind::Knight, color);
    while knights != 0 {
        let sq = knights.trailing_zeros() as usize;
        knights &= knights - 1;
        let attacks = crate::utils::knight_attacks(sq) & !own_occ;
        bonus += attacks.count_ones() as i16 * 4;
    }

    let mut bishops = board.piece_bb(PieceKind::Bishop, color);
    while bishops != 0 {
        let sq = bishops.trailing_zeros() as usize;
        bishops &= bishops - 1;
        let attacks = crate::magic::bishop_attacks(sq, board.occ) & !own_occ;
        bonus += attacks.count_ones() as i16 * 3;
    }

    let mut rooks = board.piece_bb(PieceKind::Rook, color);
    while rooks != 0 {
        let sq = rooks.trailing_zeros() as usize;
        rooks &= rooks - 1;
        let attacks = crate::magic::rook_attacks(sq, board.occ) & !own_occ;
        bonus += attacks.count_ones() as i16 * 2;
    }

    let mut queens = board.piece_bb(PieceKind::Queen, color);
    while queens != 0 {
        let sq = queens.trailing_zeros() as usize;
        queens &= queens - 1;
        let attacks = crate::magic::queen_attacks(sq, board.occ) & !own_occ;
        bonus += attacks.count_ones() as i16;
    }

    bonus
}

/// Calcola la fase di gioco (24 = apertura iniziale, 0 = finale puro)
///
/// Basato sul numero di pezzi rimasti: ogni pezzo contribuisce al "peso" della fase.
/// Formula PeSTO: phase = sum(piece_phase_weights), max 24.
fn game_phase(board: &Board) -> u8 {
    let mut phase = 0u8;
    phase += board.piece_bb(PieceKind::Queen, Color::White).count_ones() as u8 * 4;
    phase += board.piece_bb(PieceKind::Queen, Color::Black).count_ones() as u8 * 4;
    phase += board.piece_bb(PieceKind::Rook, Color::White).count_ones() as u8 * 2;
    phase += board.piece_bb(PieceKind::Rook, Color::Black).count_ones() as u8 * 2;
    phase += board.piece_bb(PieceKind::Bishop, Color::White).count_ones() as u8;
    phase += board.piece_bb(PieceKind::Bishop, Color::Black).count_ones() as u8;
    phase += board.piece_bb(PieceKind::Knight, Color::White).count_ones() as u8;
    phase += board.piece_bb(PieceKind::Knight, Color::Black).count_ones() as u8;
    phase.min(24)
}

/// Interpola tra score middlegame e endgame in base alla fase di gioco.
///
/// Formula: `(mg * phase + eg * (24 - phase)) / 24`
fn taper(mg: i32, eg: i32, phase: u8) -> i32 {
    let p = phase as i32;
    (mg * p + eg * (24 - p)) / 24
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
    if let Some(bonus) = endgame_score(board) {
        return bonus;
    }

    let mut white_mg: i32 = 0;
    let mut white_eg: i32 = 0;
    let mut black_mg: i32 = 0;
    let mut black_eg: i32 = 0;

    // Iterate directly on bitboards (much faster than piece_on() for each square)
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

        let psqt_mg = match kind {
            PieceKind::Pawn => &PAWN_PSQT,
            PieceKind::Knight => &KNIGHT_PSQT,
            PieceKind::Bishop => &BISHOP_PSQT,
            PieceKind::Rook => &ROOK_PSQT,
            PieceKind::Queen => &QUEEN_PSQT,
            PieceKind::King => &KING_PSQT,
        };
        let psqt_eg = match kind {
            PieceKind::Pawn => &PAWN_PSQT_EG,
            PieceKind::Knight => &KNIGHT_PSQT_EG,
            PieceKind::Bishop => &BISHOP_PSQT_EG,
            PieceKind::Rook => &ROOK_PSQT_EG,
            PieceKind::Queen => &QUEEN_PSQT_EG,
            PieceKind::King => &KING_PSQT_EG,
        };

        // White pieces
        let mut white_bb = board.piece_bb(kind, Color::White);
        while white_bb != 0 {
            let sq = white_bb.trailing_zeros() as usize;
            white_bb &= white_bb - 1; // Clear LSB
            white_mg += material_value as i32 + psqt_mg[sq] as i32;
            white_eg += material_value as i32 + psqt_eg[sq] as i32;
        }

        // Black pieces (flip vertically for PSQT index)
        let mut black_bb = board.piece_bb(kind, Color::Black);
        while black_bb != 0 {
            let sq = black_bb.trailing_zeros() as usize;
            black_bb &= black_bb - 1; // Clear LSB
            let psqt_index = sq ^ 56; // Flip verticale
            black_mg += material_value as i32 + psqt_mg[psqt_index] as i32;
            black_eg += material_value as i32 + psqt_eg[psqt_index] as i32;
        }
    }

    // Positional components are applied to MG only for now
    white_mg -= development_penalty(board, Color::White) as i32;
    black_mg -= development_penalty(board, Color::Black) as i32;

    white_mg += king_safety(board, Color::White) as i32;
    black_mg += king_safety(board, Color::Black) as i32;

    // Taper material + PSQT from MG to EG based on game phase
    let phase = game_phase(board);
    let mut white_score = taper(white_mg, white_eg, phase);
    let mut black_score = taper(black_mg, black_eg, phase);

    // Bishop pair bonus
    white_score += bishop_pair(board, Color::White) as i32;
    black_score += bishop_pair(board, Color::Black) as i32;

    // Pawn structure: doubled / isolated / passed
    white_score -= doubled_pawns(board, Color::White) as i32;
    black_score -= doubled_pawns(board, Color::Black) as i32;
    white_score -= isolated_pawns(board, Color::White) as i32;
    black_score -= isolated_pawns(board, Color::Black) as i32;
    white_score += passed_pawns(board, Color::White) as i32;
    black_score += passed_pawns(board, Color::Black) as i32;

    // Mobility bonus
    white_score += mobility(board, Color::White) as i32;
    black_score += mobility(board, Color::Black) as i32;

    // Center Control: valuta controllo delle caselle centrali
    let center = center_control(board);

    // Calcola lo score relativo (Bianco - Nero)
    let relative_score = (white_score - black_score) as i16 + center;

    // CRITICAL: Convenzione negamax - ritorna dal punto di vista del side-to-move
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

    #[test]
    fn test_evaluate_fast_bitboard_vs_naive() {
        // Create a board with various pieces
        // FEN: r3k2r/pppb1ppp/2n1pn2/3q4/3P4/2B1PN2/PP3PPP/R2QKB1R w KQkq - 1 9
        let mut board = Board::new();
        board
            .set_from_fen("r3k2r/pppb1ppp/2n1pn2/3q4/3P4/2B1PN2/PP3PPP/R2QKB1R w KQkq - 1 9")
            .unwrap();

        let fast_eval = evaluate_fast(&board);

        // Naive implementation for comparison
        let mut naive_white_score: i32 = 0;
        let mut naive_black_score: i32 = 0;

        for sq in 0..64 {
            if let Some((kind, color)) = board.piece_on(sq) {
                let val = match kind {
                    PieceKind::Pawn => PAWN_VALUE,
                    PieceKind::Knight => KNIGHT_VALUE,
                    PieceKind::Bishop => BISHOP_VALUE,
                    PieceKind::Rook => ROOK_VALUE,
                    PieceKind::Queen => QUEEN_VALUE,
                    PieceKind::King => KING_VALUE,
                };

                let psqt_idx = if color == Color::White { sq } else { sq ^ 56 };
                let psqt = match kind {
                    PieceKind::Pawn => PAWN_PSQT[psqt_idx],
                    PieceKind::Knight => KNIGHT_PSQT[psqt_idx],
                    PieceKind::Bishop => BISHOP_PSQT[psqt_idx],
                    PieceKind::Rook => ROOK_PSQT[psqt_idx],
                    PieceKind::Queen => QUEEN_PSQT[psqt_idx],
                    PieceKind::King => KING_PSQT[psqt_idx],
                };

                let term = val as i32 + psqt as i32;
                match color {
                    Color::White => naive_white_score += term,
                    Color::Black => naive_black_score += term,
                }
            }
        }

        // Add king safety critical (which is also called in evaluate_fast)
        naive_white_score += king_safety_critical_only(&board, Color::White) as i32;
        naive_black_score += king_safety_critical_only(&board, Color::Black) as i32;

        let relative = (naive_white_score - naive_black_score) as i16;
        let expected = if board.side == Color::Black {
            -relative
        } else {
            relative
        };

        assert_eq!(
            fast_eval, expected,
            "Bitboard evaluate_fast mismatch with naive iteration"
        );
    }

    #[test]
    fn test_bishop_pair_bonus() {
        let mut board_pair = Board::new();
        board_pair
            .set_from_fen("4k3/8/8/8/3B4/2B5/8/4K3 w - - 0 1")
            .unwrap();
        let mut board_single = Board::new();
        board_single
            .set_from_fen("4k3/8/8/8/3B4/8/8/4K3 w - - 0 1")
            .unwrap();
        assert!(
            evaluate(&board_pair) > evaluate(&board_single),
            "Bishop pair should be rewarded"
        );
    }

    #[test]
    fn test_doubled_pawns_penalty() {
        let mut board_doubled = Board::new();
        board_doubled
            .set_from_fen("4k3/8/8/8/4P3/4P3/8/4K3 w - - 0 1")
            .unwrap();
        let mut board_normal = Board::new();
        board_normal
            .set_from_fen("4k3/8/8/8/3P4/4P3/8/4K3 w - - 0 1")
            .unwrap();
        assert!(
            evaluate(&board_normal) > evaluate(&board_doubled),
            "Doubled pawns should be penalized"
        );
    }

    #[test]
    fn test_isolated_pawns_penalty() {
        let mut board_isolated = Board::new();
        board_isolated
            .set_from_fen("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1")
            .unwrap();
        let mut board_supported = Board::new();
        board_supported
            .set_from_fen("4k3/8/8/8/3P4/4P3/8/4K3 w - - 0 1")
            .unwrap();
        assert!(
            evaluate(&board_supported) > evaluate(&board_isolated),
            "Isolated pawn should be penalized"
        );
    }

    #[test]
    fn test_passed_pawns_bonus() {
        let mut board_passed = Board::new();
        board_passed
            .set_from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1")
            .unwrap();
        let mut board_blocked = Board::new();
        board_blocked
            .set_from_fen("4k3/8/8/8/8/4p3/4P3/4K3 w - - 0 1")
            .unwrap();
        assert!(
            evaluate(&board_passed) > evaluate(&board_blocked),
            "Passed pawn should be rewarded"
        );
    }

    #[test]
    fn test_mobility_bonus() {
        let mut board_center = Board::new();
        board_center
            .set_from_fen("4k3/8/8/8/3N4/8/8/4K3 w - - 0 1")
            .unwrap();
        let mut board_corner = Board::new();
        board_corner
            .set_from_fen("4k3/8/8/8/8/8/8/3NK3 w - - 0 1")
            .unwrap();
        assert!(
            evaluate(&board_center) > evaluate(&board_corner),
            "Knight in center should have higher mobility bonus"
        );
    }

    #[test]
    fn test_endgame_kbb_vs_k() {
        let mut board = Board::new();
        board
            .set_from_fen("4k3/8/8/8/8/8/4B1B1/4K3 w - - 0 1")
            .unwrap();
        assert_eq!(evaluate(&board), SIMPLE_ENDGAME_BONUS);
    }
}
