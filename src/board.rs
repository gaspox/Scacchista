// Mapping di quadrati: A1=0, B1=1, ..., H8=63
// Usiamo questo mapping coerente per tutte le operazioni pipeline

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White = 0,
    Black = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

// Indice nel piece_bb array: white piece = kind as usize; black piece = 6 + kind as usize
fn piece_index(kind: PieceKind, color: Color) -> usize {
    (color as usize) * 6 + (kind as usize)
}

pub type Move = u32;

// Codifica mosse: 32-bit layout
// Bits 0-5: from (0-63)
// Bits 6-11: to (0-63)
// Bits 12-15: piece (0-5)
// Bits 16-19: captured (0-5, 0xFF = none)
// Bits 20-23: promotion (0-5, 0xFF = none)
// Bits 24-31: flags
pub const FLAG_NONE: u32 = 0;
pub const FLAG_EN_PASSANT: u32 = 1 << 24;
pub const FLAG_CASTLE_KING: u32 = 1 << 25;
pub const FLAG_CASTLE_QUEEN: u32 = 1 << 26;
pub const FLAG_PROMOTION: u32 = 1 << 27;
pub const FLAG_CAPTURE: u32 = 1 << 28;

pub fn move_from_sq(m: Move) -> usize { (m & 0x3F) as usize }
pub fn move_to_sq(m: Move) -> usize { ((m >> 6) & 0x3F) as usize }
pub fn move_piece(m: Move) -> PieceKind { match (m >> 12) & 0xF {
    0 => PieceKind::Pawn,
    1 => PieceKind::Knight,
    2 => PieceKind::Bishop,
    3 => PieceKind::Rook,
    4 => PieceKind::Queen,
    5 => PieceKind::King,
    _ => panic!(),
}}
pub fn move_captured(m: Move) -> Option<PieceKind> {
    let v = (m >> 16) & 0xF;
    if v == 0xF { None }
    else { Some(match v {
        0 => PieceKind::Pawn,
        1 => PieceKind::Knight,
        2 => PieceKind::Bishop,
        3 => PieceKind::Rook,
        4 => PieceKind::Queen,
        5 => PieceKind::King,
        _ => panic!(),
    })}
}
pub fn move_promotion(m: Move) -> Option<PieceKind> {
    let v = (m >> 20) & 0xF;
    if v == 0xF { None }
    else { Some(match v {
        0 => PieceKind::Pawn,
        1 => PieceKind::Knight,
        2 => PieceKind::Bishop,
        3 => PieceKind::Rook,
        4 => PieceKind::Queen,
        5 => PieceKind::King,
        _ => panic!(),
    })}
}
pub fn move_flag(m: Move, flag: u32) -> bool { (m & flag) != 0 }

// Costruzione mossa
pub fn new_move(from: usize, to: usize, piece: PieceKind, captured: Option<PieceKind>, promotion: Option<PieceKind>, flags: u32) -> Move {
    let cap = captured.map(|p| p as u32).unwrap_or(0xF);
    let prom = promotion.map(|p| p as u32).unwrap_or(0xF);
    (from as u32 & 0x3F)
        | ((to as u32 & 0x3F) << 6)
        | ((piece as u32 & 0xF) << 12)
        | ((cap & 0xF) << 16)
        | ((prom & 0xF) << 20)
        | flags
}

// Undo entry per rollback
#[derive(Debug, Clone)]
pub struct Undo {
    pub move_played: Move,
    pub captured_piece: Option<PieceKind>,
    pub old_castling: u8,
    pub old_ep: Option<u8>,
    pub old_halfmove: u16,
    pub old_zobrist: u64,
}

pub struct Board {
    // 12 bitboard: 0-5 = white p,n,b,r,q,k; 6-11 = black p,n,b,r,q,k
    piece_bb: [u64; 12],
    pub white_occ: u64,
    pub black_occ: u64,
    pub occ: u64,
    pub side: Color,
    pub castling: u8, // 4 LSB: white kingside, white queenside, black ks, black qs
    pub ep: Option<u8>, // en-passant square index or None
    pub halfmove: u16,
    pub fullmove: u16,
    pub zobrist: u64,
    // Undo stack per unmake; capacità per centinaia di plies
    _undo_stack: Vec<Undo>,
}

impl Board {
    // Board vuota da popolare via hand FEN-SETUP
    pub fn new() -> Self {
        Self {
            piece_bb: [0; 12],
            white_occ: 0,
            black_occ: 0,
            occ: 0,
            side: Color::White,
            castling: 0,
            ep: None,
            halfmove: 0,
            fullmove: 1,
            zobrist: 0,
            _undo_stack: Vec::with_capacity(1024),
        }
    }

    // Restituisce piece_bb index
    pub fn piece_bb(&self, kind: PieceKind, color: Color) -> u64 {
        self.piece_bb[piece_index(kind, color)]
    }

    // Helper accesso raw bb per rendering/debug
    pub fn piece_bb_raw(&self, idx: usize) -> u64 { self.piece_bb[idx] }

    // Restituisce piece (kind,color) su square idx o None
    pub fn piece_on(&self, sq: usize) -> Option<(PieceKind, Color)> {
        let mask = 1u64 << sq;
        for i in 0..12 {
            if self.piece_bb[i] & mask != 0 {
                let kind = match i % 6 {
                    0 => PieceKind::Pawn,
                    1 => PieceKind::Knight,
                    2 => PieceKind::Bishop,
                    3 => PieceKind::Rook,
                    4 => PieceKind::Queen,
                    5 => PieceKind::King,
                    _ => unreachable!(),
                };
                let color = if i < 6 { Color::White } else { Color::Black };
                return Some((kind, color));
            }
        }
        None
    }

    // Posiziona un pezzo; helper per FEN; NON aggiorna occupancy o Zobrist internamente (via set_from_fen)
    pub fn set_piece(&mut self, sq: usize, kind: PieceKind, color: Color) {
        let i = piece_index(kind, color);
        self.piece_bb[i] |= 1u64 << sq;
    }

    // Rimuove pezzo; helper per make/unmake
    pub fn remove_piece(&mut self, sq: usize, kind: PieceKind, color: Color) {
        let i = piece_index(kind, color);
        self.piece_bb[i] &= !(1u64 << sq);
    }

    // Refresh occupancy after bulk placement (usata in FEN setup)
    pub fn refresh_occupancy(&mut self) {
        self.white_occ = 0;
        self.black_occ = 0;
        for i in 0..6 {
            self.white_occ |= self.piece_bb[i];
        }
        for i in 6..12 {
            self.black_occ |= self.piece_bb[i];
        }
        self.occ = self.white_occ | self.black_occ;
    }

    // Verifica se un quadrato è occupato
    pub fn is_occupied(&self, sq: usize) -> bool { (1u64 << sq & self.occ) != 0 }

    // Verifica se un quadrato è occupato dal colore opposto (old-style)
    pub fn is_attacked_by(&self, _sq: usize, _by: Color) -> bool {
        // TODO implementare dipendenza da magic tables o naive sliding
        false
    }

    // make_move restituisce Undo entry per unmake; dobbiamo chiamare update_occupancy e Zobrist dopo
    // Per ora implementiamo snapshot-based full copy (lento) ma in grado di fare perft correttamente
    // Optimization plan: implementare make/unmake incremental
    pub fn make_move(&mut self, mv: Move) -> Undo {
        let from = move_from_sq(mv);
        let to = move_to_sq(mv);
        let piece = move_piece(mv);
        let captured = move_captured(mv);
        let ep_target = self.ep;

        let undo = Undo {
            move_played: mv,
            captured_piece: captured,
            old_castling: self.castling,
            old_ep: self.ep,
            old_halfmove: self.halfmove,
            old_zobrist: self.zobrist,
        };
        // Rimuovi pezzo da from
        let color = if self.white_occ & (1u64 << from) != 0 { Color::White } else { Color::Black };
        self.remove_piece(from, piece, color);
        // Se cattura, rimuovi pezzo catturato (salvato captured)
        if let Some(capt) = captured {
            // Logica per rimuovere da ep square se cattura en-passant
            if move_flag(mv, FLAG_EN_PASSANT) {
                // remove captured pawn at ep square
                let ep_sq = ep_target.unwrap();
                self.remove_piece(ep_sq as usize, PieceKind::Pawn, if color == Color::White { Color::Black } else { Color::White });
            } else {
                self.remove_piece(to, capt, if color == Color::White { Color::Black } else { Color::White });
            }
        }
        // Promo: inserisci (cambia kind)
        if move_flag(mv, FLAG_PROMOTION) {
            self.set_piece(to, move_promotion(mv).unwrap(), color);
        } else {
            self.set_piece(to, piece, color);
        }

        // Gestione en-passant: se pedone di due step, imposta
        if piece == PieceKind::Pawn && to.abs_diff(from) == 16 {
            self.ep = Some(((from + to) / 2) as u8); // square halfway
        } else {
            self.ep = None;
        }
        // Increment halfmove: non per pedone o cattura
        self.halfmove += 1;
        if piece == PieceKind::Pawn || captured.is_some() {
            self.halfmove = 0;
        }
        // Castling: remove rook from castling squares, update flags
        if move_flag(mv, FLAG_CASTLE_KING) || move_flag(mv, FLAG_CASTLE_QUEEN) {
            let (rook_from, rook_to) = if color == Color::White {
                if move_flag(mv, FLAG_CASTLE_KING) { (7, 5) } else { (0, 3) }
            } else {
                if move_flag(mv, FLAG_CASTLE_KING) { (63, 61) } else { (56, 59) }
            };
            self.remove_piece(rook_from, PieceKind::Rook, color);
            self.set_piece(rook_to, PieceKind::Rook, color);
        }
        // Aggiorna castling rights: se re muove o torre muove da/quella posizione
        self.update_castling_after_move(color, piece, from);
        // Refresh occupancy and zobrist
        self.refresh_occupancy();
        // Switch side
        self.side = if self.side == Color::White { Color::Black } else { Color::White };
        // Incrementa fullmove per ogni white move
        if self.side == Color::White { self.fullmove += 1; }
        // Per ora reinizializzo Zobrist a zero; aggiorno con zobrist.rs dopo
        self.zobrist = self.recalc_zobrist();
        undo
    }

    // Unmake ripristina Board usando Undo e revertendo Zobrist
    pub fn unmake_move(&mut self, _undo: Undo) {
        // Ripristina TUTTO usando undo snapshot (temp)
        // Implementazione estremamente conservativa: rollback con copia vecchia?
        // A scelta: implementare ricorsivo, ma per ora ricalcoliamo
        // In perft testing higher depth è poco ottimizzato ma sufficente per validazione
        // Una volta passata validate, implementiamo make/unmake in-place incremental
        // Qui per ora facciamo un rollback ricostituendo da undo e salvando stato Board corrente su temporaneo
        // Riporto Board temporanea al vecchio stato; ma cosi' non fa rollback correttamente l'ep/castling/halfmove count. Da completare con test.
        // Semplificazione: in questa fase usiamo copy-on-make su perft per robustezza (lento ma corretto)
        // Implementazione corretta richiedera' make/unmake incremental, che aggiungeremo dopo Perft OK
    }

    // Aggiorna castling rights dopo che il pezzo/pioniere si è mosso da from
    fn update_castling_after_move(&mut self, side: Color, piece: PieceKind, from: usize) {
        const KING_SQ: [usize; 2] = [4, 60]; // white king e1, black king e8
        const ROOK_KS: [usize; 2] = [7, 63]; // white rook h1, black rook h8
        const ROOK_QS: [usize; 2] = [0, 56]; // rooks a1,a8

        let s = side as usize;
        if piece == PieceKind::King && from == KING_SQ[s] {
            self.castling &= !(0b11 << (2 * s as u8)); // toglie king e queen
        }
        if piece == PieceKind::Rook {
            if from == ROOK_KS[s] {
                self.castling &= !(1 << (2 * s as u8));
            } else if from == ROOK_QS[s] {
                self.castling &= !(1 << (2 * s as u8 + 1));
            }
        }
    }

    // Ricalc Zobrist serve temporaneamente (evita rattoppi con aggiornamenti incremental)
    fn recalc_zobrist(&self) -> u64 {
        0  // placeholder per ora, lo implemento subito dopo
    }

}

// Helper conversioni FEN
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// FEN parsing/setter su Board
impl Board {
    pub fn set_from_fen(&mut self, fen: &str) -> Result<(), &'static str> {
        let mut parts = fen.trim().split_whitespace();
        let piece_part = parts.next().ok_or("missing pieces")?;
        let side_part = parts.next().ok_or("missing side")?;
        let castle_part = parts.next().ok_or("missing castling")?;
        let ep_part = parts.next().ok_or("missing en-passant")?;
        let halfmove_part = parts.next().ok_or("missing halfmove")?;
        let fullmove_part = parts.next().ok_or("missing fullmove")?;

        // Reset board
        self.piece_bb = [0;12];
        self.white_occ = 0; self.black_occ = 0; self.occ = 0;

        // Parse pieces: rank8 .. rank1
        let mut rank = 7;
        for rank_part in piece_part.split('/') {
            let mut file = 0;
            for ch in rank_part.chars() {
                if ch.is_ascii_digit() {
                    file += ch.to_digit(10).unwrap() as usize;
                } else {
                    let (kind, color) = match ch {
                        'P' => (PieceKind::Pawn, Color::White),
                        'N' => (PieceKind::Knight, Color::White),
                        'B' => (PieceKind::Bishop, Color::White),
                        'R' => (PieceKind::Rook, Color::White),
                        'Q' => (PieceKind::Queen, Color::White),
                        'K' => (PieceKind::King, Color::White),
                        'p' => (PieceKind::Pawn, Color::Black),
                        'n' => (PieceKind::Knight, Color::Black),
                        'b' => (PieceKind::Bishop, Color::Black),
                        'r' => (PieceKind::Rook, Color::Black),
                        'q' => (PieceKind::Queen, Color::Black),
                        'k' => (PieceKind::King, Color::Black),
                        _ => return Err("invalid piece char"),
                    };
                    self.set_piece(rank*8 + file, kind, color);
                    file += 1;
                }
            }
            rank -= 1;
        }

        self.refresh_occupancy();

        // Side to move
        self.side = match side_part {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("invalid side char"),
        };

        // Castling rights: KQkq mapping bits wk wq bk bq
        self.castling = 0;
        for ch in castle_part.chars() {
            match ch {
                'K' => self.castling |= 0b1000u8,
                'Q' => self.castling |= 0b0100u8,
                'k' => self.castling |= 0b0010u8,
                'q' => self.castling |= 0b0001u8,
                '-' => {}
                _ => return Err("invalid castle char"),
            }
        }

        // En-passant
        self.ep = match ep_part {
            "-" => None,
            s => {
                // ep square string represent la square dietro all'avanzato di 2, convert to index
                if s.len() != 2 { return Err("invalid ep string"); }
                let file = s.chars().next().unwrap();
                let rank = s.chars().nth(1).unwrap();
                let f_idx = match file {
                    'a'..='h' => (file as u8 - b'a') as usize,
                    _ => return Err("invalid ep file"),
                };
                let r_idx = match rank {
                    '3' | '6' => (rank as u8 - b'1') as usize,
                    _ => return Err("invalid ep rank"),
                };
                Some((r_idx * 8 + f_idx) as u8)
            }
        };

        self.halfmove = halfmove_part.parse().map_err(|_| "invalid halfmove")?;
        self.fullmove = fullmove_part.parse().map_err(|_| "invalid fullmove")?;

        // Zobrist placeholder per ora
        self.zobrist = self.recalc_zobrist();

        Ok(())
    }
}

// Simple display (fen)
impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Rappresentazione solo del bitboard testuale (debug)
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = rank*8+file;
                if let Some((p,c)) = self.piece_on(sq) {
                    let ch = match (c,p) {
                        (Color::White, PieceKind::Pawn) => 'P',
                        (Color::White, PieceKind::Knight) => 'N',
                        (Color::White, PieceKind::Bishop) => 'B',
                        (Color::White, PieceKind::Rook) => 'R',
                        (Color::White, PieceKind::Queen) => 'Q',
                        (Color::White, PieceKind::King) => 'K',
                        (Color::Black, PieceKind::Pawn) => 'p',
                        (Color::Black, PieceKind::Knight) => 'n',
                        (Color::Black, PieceKind::Bishop) => 'b',
                        (Color::Black, PieceKind::Rook) => 'r',
                        (Color::Black, PieceKind::Queen) => 'q',
                        (Color::Black, PieceKind::King) => 'k',
                    };
                    write!(f, "{} ", ch)?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}