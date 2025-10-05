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
    pub from: usize,
    pub to: usize,
    pub moved_piece: PieceKind,
    pub flags: u32,
    pub captured_piece: Option<PieceKind>,
    pub captured_sq: Option<usize>,
    pub prev_ep: Option<u8>,
    pub prev_castling: u8,
    pub prev_halfmove: u16,
    pub prev_fullmove: u16,
    pub prev_side: Color,
    pub prev_zobrist: u64,
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
    // King squares for fast king safety check
    pub white_king_sq: u8,
    pub black_king_sq: u8,
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
            white_king_sq: 0,
            black_king_sq: 0,
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
        if kind == PieceKind::King {
            match color {
                Color::White => self.white_king_sq = sq as u8,
                Color::Black => self.black_king_sq = sq as u8,
            }
        }
    }

    // Rimuovi pezzo; se re, memorizzo che è momentaneamente non sul board (make/unmake tracking)
    pub fn remove_piece(&mut self, sq: usize, kind: PieceKind, color: Color) {
        let i = piece_index(kind, color);
        self.piece_bb[i] &= !(1u64 << sq);
        if kind == PieceKind::King {
            // In make/unmake tracking, la rimozione del re potrà avvenire per un attimo durante arrocco, ma lo riposizioniamo subito.
            // Non aggiorniamo king squares qui; lo farà make_move con le logiche ordinate.
        }
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

    // King square per side
    pub fn king_sq(&self, side: Color) -> usize {
        match side {
            Color::White => self.white_king_sq as usize,
            Color::Black => self.black_king_sq as usize,
        }
    }

    pub fn make_move(&mut self, mv: Move) -> Undo {
        let from = move_from_sq(mv);
        let to = move_to_sq(mv);
        let piece = move_piece(mv);
        let flags = mv & 0xFF000000u32;
        let captured = move_captured(mv);
        let ep_target = self.ep;

        let color = if self.white_occ & (1u64 << from) != 0 { Color::White } else { Color::Black };
        let captured_sq = if move_flag(mv, FLAG_EN_PASSANT) {
            Some(if color == Color::White { (ep_target.unwrap() as i32) - 8 } else { (ep_target.unwrap() as i32) + 8 } as usize)
        } else if captured.is_some() {
            Some(to)
        } else {
            None
        };
        let undo = Undo {
            from,
            to,
            moved_piece: piece,
            flags,
            captured_piece: captured,
            captured_sq,
            prev_ep: self.ep,
            prev_castling: self.castling,
            prev_halfmove: self.halfmove,
            prev_fullmove: self.fullmove,
            prev_side: self.side,
            prev_zobrist: self.zobrist,
        };
        // Update Zobrist incrementally (undo still holds previous hash)
        crate::zobrist::init_zobrist();
        unsafe {
            // Remove piece from old square
            self.zobrist ^= crate::zobrist::ZOB_PIECE[piece_index(piece, color)][from];
            // Add piece/moved piece or promoted piece
            let moved = if move_flag(mv, FLAG_PROMOTION) { move_promotion(mv).unwrap() } else { piece };
            self.zobrist ^= crate::zobrist::ZOB_PIECE[piece_index(moved, color)][to];
            // Remove captured or e-p captured
            if let Some(capt) = captured {
                let cap_color = if color == Color::White { Color::Black } else { Color::White };
                let cap_sq = if move_flag(mv, FLAG_EN_PASSANT) { captured_sq.unwrap() } else { to };
                self.zobrist ^= crate::zobrist::ZOB_PIECE[piece_index(capt, cap_color)][cap_sq];
            }
            // Side toggle
            self.zobrist ^= crate::zobrist::ZOB_SIDE;
            // Castling rights changes
            let old_r = self.castling as usize;
            self.update_castling_after_move(color, piece, from);
            let new_r = self.castling as usize;
            if old_r != new_r {
                self.zobrist ^= crate::zobrist::ZOB_CASTLING[old_r];
                self.zobrist ^= crate::zobrist::ZOB_CASTLING[new_r];
            }
            // En-passant file toggle
            {
                if let Some(old_ep_sq) = undo.prev_ep {
                    let old_file = (old_ep_sq % 8) as usize;
                    self.zobrist ^= crate::zobrist::ZOB_EP_FILE[old_file];
                }
                let new_ep_sq = if piece == PieceKind::Pawn && to.abs_diff(from) == 16 {
                Some(((from + to) / 2) as u8)
            } else {
                None
            };
                if let Some(ep_sq) = new_ep_sq {
                    let file = (ep_sq % 8) as usize;
                    self.zobrist ^= crate::zobrist::ZOB_EP_FILE[file];
                }
            }
        }
        // Update piece/occupancy fields
        if piece == PieceKind::King {
            if color == Color::White { self.white_king_sq = to as u8; } else { self.black_king_sq = to as u8; }
        }
        self.remove_piece(from, piece, color);
        if let Some(capt) = captured {
            if move_flag(mv, FLAG_EN_PASSANT) {
                self.remove_piece(captured_sq.unwrap(), capt, if color == Color::White { Color::Black } else { Color::White });
            } else {
                self.remove_piece(to, capt, if color == Color::White { Color::Black } else { Color::White });
            }
        }
        let moved_piece = if move_flag(mv, FLAG_PROMOTION) { move_promotion(mv).unwrap() } else { piece };
        self.set_piece(to, moved_piece, color);
        self.refresh_occupancy();
        // Update en-passant flag
        self.ep = if piece == PieceKind::Pawn && to.abs_diff(from) == 16 { Some(((from + to) / 2) as u8) } else { None };
        // Update move counters
        self.halfmove += 1;
        if piece == PieceKind::Pawn || captured.is_some() { self.halfmove = 0; }
        self.side = if self.side == Color::White { Color::Black } else { Color::White };
        if self.side == Color::White { self.fullmove += 1; }
        undo
    }

    pub fn unmake_move(&mut self, undo: Undo) {
        // Restore move counters/halfmove/fullmove/side first
        self.side = undo.prev_side;
        self.halfmove = undo.prev_halfmove;
        self.fullmove = undo.prev_fullmove;
        self.ep = undo.prev_ep;
        self.castling = undo.prev_castling;
        // Restore piece bitboards and king square
        let moved_piece = undo.moved_piece; // promotion piece already stored in undo.moved_piece
        let mover_color = self.side; // self.side was restored to the mover's color above

        // Remove moved piece from destination and put it back on origin
        self.remove_piece(undo.to, moved_piece, mover_color);
        self.set_piece(undo.from, moved_piece, mover_color);

        if moved_piece == PieceKind::King {
            if mover_color == Color::White {
                self.white_king_sq = undo.from as u8;
            } else {
                self.black_king_sq = undo.from as u8;
            }
        }

        // Restore captured if any (captured piece belongs to the opponent)
        if let Some(capt) = undo.captured_piece {
            let cap_color = if mover_color == Color::White { Color::Black } else { Color::White };
            self.set_piece(undo.captured_sq.unwrap(), capt, cap_color);
        }

        self.refresh_occupancy();
        // Restore hash
        self.zobrist = undo.prev_zobrist;
    }

    // Aggiorna castling rights dopo che il pezzo/pioniere si è mosso da from
    fn update_castling_after_move(&mut self, side: Color, piece: PieceKind, from: usize) {
        const KING_SQ: [usize; 2] = [4, 60]; // white king e1, black king e8
        const ROOK_KS: [usize; 2] = [7, 63]; // white rook h1, black rook h8
        const ROOK_QS: [usize; 2] = [0, 56]; // rooks a1,a8

        let s = side as usize;
        if piece == PieceKind::King && from == KING_SQ[s] {
            self.castling &= !(0b11 << (2 * s as u8));
        }
        if piece == PieceKind::Rook {
            if from == ROOK_KS[s] {
                self.castling &= !(1 << (2 * s as u8));
            } else if from == ROOK_QS[s] {
                self.castling &= !(1 << (2 * s as u8 + 1));
            }
        }
    }

    // Debug method to see what's attacking a square
    fn debug_attacks(&self, sq: usize, by: Color) {
        // Pawn attacks
        if by == Color::White {
            let white_pawns = self.piece_bb(PieceKind::Pawn, Color::White);
            if ((white_pawns & crate::utils::NOT_FILE_A) << 7) & (1u64 << sq) != 0 {
                eprintln!("  Attacked by white pawn from file-1");
            }
            if ((white_pawns & crate::utils::NOT_FILE_H) << 9) & (1u64 << sq) != 0 {
                eprintln!("  Attacked by white pawn from file+1");
            }
        } else {
            let black_pawns = self.piece_bb(PieceKind::Pawn, Color::Black);
            let left_attacks = (black_pawns & crate::utils::NOT_FILE_A) >> 9;
            let right_attacks = (black_pawns & crate::utils::NOT_FILE_H) >> 7;
            eprintln!("  Black pawns bb: {:x}", black_pawns);
            eprintln!("  Black left attacks on sq {}: {:x} & {:x} = {:x}", sq, left_attacks, 1u64 << sq, left_attacks & (1u64 << sq));
            eprintln!("  Black right attacks on sq {}: {:x} & {:x} = {:x}", sq, right_attacks, 1u64 << sq, right_attacks & (1u64 << sq));
            if left_attacks & (1u64 << sq) != 0 {
                eprintln!("  Attacked by black pawn from file+1");
            }
            if right_attacks & (1u64 << sq) != 0 {
                eprintln!("  Attacked by black pawn from file-1");
            }
        }
        // Knight attacks
        if crate::utils::knight_attacks(sq) & self.piece_bb(PieceKind::Knight, by) != 0 {
            eprintln!("  Attacked by knight");
        }
        // King attacks
        if crate::utils::king_attacks(sq) & self.piece_bb(PieceKind::King, by) != 0 {
            eprintln!("  Attacked by king");
        }
        // Diagonal attacks
        let diagonal_attackers = self.piece_bb(PieceKind::Bishop, by) | self.piece_bb(PieceKind::Queen, by);
        if diagonal_attackers != 0 {
            eprintln!("  Checking diagonal attacks with attackers at {:x}", diagonal_attackers);
        }
        // Orthogonal attacks
        let orthogonal_attackers = self.piece_bb(PieceKind::Rook, by) | self.piece_bb(PieceKind::Queen, by);
        if orthogonal_attackers != 0 {
            eprintln!("  Checking orthogonal attacks with attackers at {:x}", orthogonal_attackers);
        }
    }

    // Public method to force recalc Zobrist
    pub fn recalc_zobrist(&self) -> u64 {
        crate::zobrist::recalc_zobrist_full(self)
    }

    // Legality helpers -------------------------------------------
    pub fn is_square_attacked(&self, sq: usize, by: Color) -> bool {
        // Pawn attacks
        if by == Color::White {
            let white_pawns = self.piece_bb(PieceKind::Pawn, Color::White);
            if ((white_pawns & crate::utils::NOT_FILE_A) << 7) & (1u64 << sq) != 0 { return true; }
            if ((white_pawns & crate::utils::NOT_FILE_H) << 9) & (1u64 << sq) != 0 { return true; }
        } else {
            let black_pawns = self.piece_bb(PieceKind::Pawn, Color::Black);
            if ((black_pawns & crate::utils::NOT_FILE_A) >> 9) & (1u64 << sq) != 0 { return true; }
            if ((black_pawns & crate::utils::NOT_FILE_H) >> 7) & (1u64 << sq) != 0 { return true; }
        }
        // Knight attacks
        if crate::utils::knight_attacks(sq) & self.piece_bb(PieceKind::Knight, by) != 0 { return true; }
        // King attacks
        if crate::utils::king_attacks(sq) & self.piece_bb(PieceKind::King, by) != 0 { return true; }
        // Bishop/Queen (diagonal sliding)
        let diagonal_attackers = self.piece_bb(PieceKind::Bishop, by) | self.piece_bb(PieceKind::Queen, by);
        if diagonal_attackers != 0 {
            // northwest
            let mut s = sq as i8 - 9;
            while s >= 0 && (s % 8) != 7 {
                if (1u64 << s) & self.occ != 0 {
                    if (1u64 << s) & diagonal_attackers != 0 { return true; }
                    break;
                }
                s -= 9;
            }
            // northeast
            let mut s = sq as i8 - 7;
            while s >= 0 && (s % 8) != 0 {
                if (1u64 << s) & self.occ != 0 {
                    if (1u64 << s) & diagonal_attackers != 0 { return true; }
                    break;
                }
                s -= 7;
            }
            // southwest
            let mut s = sq as i8 + 7;
            while s < 64 && (s % 8) != 0 {
                if (1u64 << s) & self.occ != 0 {
                    if (1u64 << s) & diagonal_attackers != 0 { return true; }
                    break;
                }
                s += 7;
            }
            // southeast
            let mut s = sq as i8 + 9;
            while s < 64 && (s % 8) != 7 {
                if (1u64 << s) & self.occ != 0 {
                    if (1u64 << s) & diagonal_attackers != 0 { return true; }
                    break;
                }
                s += 9;
            }
        }
        // Rook/Queen (orthogonal sliding)
        let orthogonal_attackers = self.piece_bb(PieceKind::Rook, by) | self.piece_bb(PieceKind::Queen, by);
        if orthogonal_attackers != 0 {
            // north
            let mut s = (sq as i8) + 8;
            while s < 64 {
                if (1u64 << s) & self.occ != 0 {
                    if (1u64 << s) & orthogonal_attackers != 0 { return true; }
                    break;
                }
                s += 8;
            }
            // south
            let mut s = (sq as i8) - 8;
            while s >= 0 {
                if (1u64 << s) & self.occ != 0 {
                    if (1u64 << s) & orthogonal_attackers != 0 { return true; }
                    break;
                }
                s -= 8;
            }
            // east
            if sq % 8 != 7 {
                let mut s = sq as i8 + 1;
                while s % 8 != 0 {
                    if 1u64 << s & self.occ != 0 {
                        if 1u64 << s & orthogonal_attackers != 0 { return true; }
                        break;
                    }
                    s += 1;
                }
            }
            // west
            if sq % 8 != 0 {
                let mut s = sq as i8 - 1;
                while s >= 0 && s % 8 != 7 {
                    if (1u64 << s) & self.occ != 0 {
                        if (1u64 << s) & orthogonal_attackers != 0 { return true; }
                        break;
                    }
                    s -= 1;
                }
            }
        }
        false
    }

    // Generate moves APIs -----------------------------------------
    pub fn generate_moves(&mut self) -> Vec<Move> {
        let mut pseudo: Vec<Move> = Vec::with_capacity(256);
        self.generate_pseudo_moves(&mut pseudo);
        let mut legal = Vec::with_capacity(pseudo.len());
        for mv in pseudo {
            let undo = self.make_move(mv);
            // After make_move, self.side is now the opponent
            let side_to_move = self.side;
            let side_that_moved = if side_to_move == Color::White { Color::Black } else { Color::White };
            let own_king_sq = self.king_sq(side_that_moved);
            let is_attacked = self.is_square_attacked(own_king_sq, side_to_move);
            if !is_attacked {
                legal.push(mv);
            }
            self.unmake_move(undo);
        }
        legal
    }
    pub fn generate_pseudo_moves(&self, out: &mut Vec<Move>) {
        self.generate_pawn_pseudos(self.side, out);
        self.generate_knight_pseudos(self.side, out);
        self.generate_bishop_pseudos(self.side, out);
        self.generate_rook_pseudos(self.side, out);
        self.generate_queen_pseudos(self.side, out);
        self.generate_king_pseudos(self.side, out);
        // All piece types implemented
    }
    fn generate_pawn_pseudos(&self, side: Color, out: &mut Vec<Move>) {
        let pawns = self.piece_bb(PieceKind::Pawn, side);
        let empty = !self.occ;
        let (prom_rank, enemy_occ, _forward_shift, ep_target) = match side {
            Color::White => (crate::utils::RANK_8, self.black_occ, 8, self.ep),
            Color::Black => (crate::utils::RANK_1, self.white_occ, -8, self.ep),
        };
        // Single pushes
        let push_dest = match side {
            Color::White => (pawns << 8) & empty,
            Color::Black => (pawns >> 8) & empty,
        };
        let mut bb = push_dest & !prom_rank;
        while let Some(to) = crate::utils::pop_lsb(&mut bb) {
            let from = match side {
                Color::White => to - 8,
                Color::Black => to + 8,
            };
            out.push(new_move(from, to, PieceKind::Pawn, None, None, FLAG_NONE));
        }
        // Double pushes (only if on start rank)
        let start_rank = match side {
            Color::White => crate::utils::RANK_2,
            Color::Black => crate::utils::RANK_7,
        };
        let candidates = pawns & start_rank;
        let first_push = match side {
            Color::White => (candidates << 8) & empty,
            Color::Black => (candidates >> 8) & empty,
        };
        let double_dest = match side {
            Color::White => (first_push << 8) & empty,
            Color::Black => (first_push >> 8) & empty,
        };
        bb = double_dest;
        while let Some(to) = crate::utils::pop_lsb(&mut bb) {
            let from = match side {
                Color::White => to - 16,
                Color::Black => to + 16,
            };
            out.push(new_move(from, to, PieceKind::Pawn, None, None, FLAG_NONE));
        }
        // Captures (including ep target)
        let right_capture = match side {
            Color::White => ((pawns & crate::utils::NOT_FILE_H) << 9) & enemy_occ,
            Color::Black => ((pawns & crate::utils::NOT_FILE_H) >> 7) & enemy_occ,
        };
        bb = right_capture;
        while let Some(to) = crate::utils::pop_lsb(&mut bb) {
            let from = match side {
                Color::White => to - 9,
                Color::Black => to + 7,
            };
            let (_capsq, flags) = if ep_target.map_or(false, |ep| to == ep as usize) {
                let ep_sq = ep_target.unwrap() as usize;
                (match side {
                    Color::White => (ep_sq as i32 - 8) as usize,
                    Color::Black => (ep_sq as i32 + 8) as usize,
                }, FLAG_EN_PASSANT | FLAG_CAPTURE)
            } else {
                (to, FLAG_CAPTURE)
            };
            out.push(new_move(from, to, PieceKind::Pawn, Some(PieceKind::Pawn), None, flags));
        }
        let left_capture = match side {
            Color::White => ((pawns & crate::utils::NOT_FILE_A) << 7) & enemy_occ,
            Color::Black => ((pawns & crate::utils::NOT_FILE_A) >> 9) & enemy_occ,
        };
        bb = left_capture;
        while let Some(to) = crate::utils::pop_lsb(&mut bb) {
            let from = match side {
                Color::White => to - 7,
                Color::Black => to + 9,
            };
            let (_capsq, flags) = if ep_target.map_or(false, |ep| to == ep as usize) {
                let ep_sq = ep_target.unwrap() as usize;
                (match side {
                    Color::White => (ep_sq as i32 - 8) as usize,
                    Color::Black => (ep_sq as i32 + 8) as usize,
                }, FLAG_EN_PASSANT | FLAG_CAPTURE)
            } else {
                (to, FLAG_CAPTURE)
            };
            out.push(new_move(from, to, PieceKind::Pawn, Some(PieceKind::Pawn), None, flags));
        }
        // Promotions (push and capture onto promotion rank)
        let promo_push_dest = push_dest & prom_rank;
        bb = promo_push_dest;
        while let Some(to) = crate::utils::pop_lsb(&mut bb) {
            let from = match side {
                Color::White => to - 8,
                Color::Black => to + 8,
            };
            for kind in [PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
                out.push(new_move(from, to, PieceKind::Pawn, None, Some(kind), FLAG_PROMOTION));
            }
        }
        // Promo captures
        let promo_capture_right = match side {
            Color::White => ((pawns & crate::utils::NOT_FILE_H) << 9) & enemy_occ & prom_rank,
            Color::Black => ((pawns & crate::utils::NOT_FILE_H) >> 7) & enemy_occ & prom_rank,
        };
        bb = promo_capture_right;
        while let Some(to) = crate::utils::pop_lsb(&mut bb) {
            let from = match side {
                Color::White => to - 9,
                Color::Black => to + 7,
            };
            for kind in [PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
                out.push(new_move(from, to, PieceKind::Pawn, Some(PieceKind::Pawn), Some(kind), FLAG_PROMOTION | FLAG_CAPTURE));
            }
        }
        let promo_capture_left = match side {
            Color::White => ((pawns & crate::utils::NOT_FILE_A) << 7) & enemy_occ & prom_rank,
            Color::Black => ((pawns & crate::utils::NOT_FILE_A) >> 9) & enemy_occ & prom_rank,
        };
        bb = promo_capture_left;
        while let Some(to) = crate::utils::pop_lsb(&mut bb) {
            let from = match side {
                Color::White => to - 7,
                Color::Black => to + 9,
            };
            for kind in [PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
                out.push(new_move(from, to, PieceKind::Pawn, Some(PieceKind::Pawn), Some(kind), FLAG_PROMOTION | FLAG_CAPTURE));
            }
        }
    }

    fn generate_knight_pseudos(&self, side: Color, out: &mut Vec<Move>) {
        let knights = self.piece_bb(PieceKind::Knight, side);
        let mut bb = knights;
        while let Some(from) = crate::utils::pop_lsb(&mut bb) {
            let attacks = crate::utils::knight_attacks(from);
            let mut dest_bb = attacks & !self.occ; // quiet moves
            while let Some(to) = crate::utils::pop_lsb(&mut dest_bb) {
                out.push(new_move(from, to, PieceKind::Knight, None, None, FLAG_NONE));
            }
            let mut capture_bb = attacks & {
                if side == Color::White { self.black_occ } else { self.white_occ }
            };
            while let Some(to) = crate::utils::pop_lsb(&mut capture_bb) {
                let piece_on_to = self.piece_on(to).unwrap().0;
                out.push(new_move(from, to, PieceKind::Knight, Some(piece_on_to), None, FLAG_CAPTURE));
            }
        }
    }

    fn generate_bishop_pseudos(&self, side: Color, out: &mut Vec<Move>) {
        let bishops = self.piece_bb(PieceKind::Bishop, side);
        let mut bb = bishops;
        while let Some(from) = crate::utils::pop_lsb(&mut bb) {
            // Generate moves along all 4 diagonal directions
            let directions = [9i8, -9i8, 7i8, -7i8]; // NE, SW, NW, SE

            for &dir in &directions {
                let mut to = from as i8;
                loop {
                    to += dir;

                    // Check if we're still on board
                    if to < 0 || to >= 64 {
                        break;
                    }

                    // Check board edge transitions for diagonal moves
                    let from_sq = to - dir;
                    if from_sq < 0 || from_sq >= 64 {
                        break;
                    }
                    let from_file = from_sq % 8;

                    // Bishop moves shouldn't wrap around files
                    if (dir == 9 && from_file == 7) || (dir == -9 && from_file == 0) {
                        break; // Cannot wrap from H to A or vice versa
                    }
                    if (dir == 7 && from_file == 0) || (dir == -7 && from_file == 7) {
                        break; // Cannot wrap from A to H or vice versa
                    }

                    let to_usize = to as usize;

                    // Check if square is occupied
                    if self.is_occupied(to_usize) {
                        // If occupied by enemy, can capture
                        let enemy_occ = if side == Color::White { self.black_occ } else { self.white_occ };
                        if ((1u64 << to) & enemy_occ) != 0 {
                            if let Some((piece_kind, _)) = self.piece_on(to_usize) {
                                out.push(new_move(from, to_usize, PieceKind::Bishop, Some(piece_kind), None, FLAG_CAPTURE));
                            }
                        }
                        break; // Stop sliding when we hit any piece
                    }

                    // Empty square - can move here
                    out.push(new_move(from, to_usize, PieceKind::Bishop, None, None, FLAG_NONE));
                }
            }
        }
    }

    fn generate_rook_pseudos(&self, side: Color, out: &mut Vec<Move>) {
        let rooks = self.piece_bb(PieceKind::Rook, side);
        let mut bb = rooks;
        while let Some(from) = crate::utils::pop_lsb(&mut bb) {
            // Generate moves along all 4 orthogonal directions
            let directions = [8i8, -8i8, 1i8, -1i8]; // North, South, East, West

            for &dir in &directions {
                let mut to = from as i8;
                loop {
                    to += dir;

                    // Check if we're still on board
                    if to < 0 || to >= 64 {
                        break;
                    }

                    // Check board edge transitions for horizontal moves
                    if dir == 1 { // East
                        let from_sq = to - dir;
                        if from_sq >= 0 && (from_sq % 8) == 7 { break; } // Cannot wrap from H to A
                    } else if dir == -1 { // West
                        let from_sq = to - dir;
                        if from_sq >= 0 && (from_sq % 8) == 0 { break; } // Cannot wrap from A to H
                    }

                    let to_usize = to as usize;

                    // Check if square is occupied
                    if self.is_occupied(to_usize) {
                        // If occupied by enemy, can capture
                        let enemy_occ = if side == Color::White { self.black_occ } else { self.white_occ };
                        if ((1u64 << to) & enemy_occ) != 0 {
                            if let Some((piece_kind, _)) = self.piece_on(to_usize) {
                                out.push(new_move(from, to_usize, PieceKind::Rook, Some(piece_kind), None, FLAG_CAPTURE));
                            }
                        }
                        break; // Stop sliding when we hit any piece
                    }

                    // Empty square - can move here
                    out.push(new_move(from, to_usize, PieceKind::Rook, None, None, FLAG_NONE));
                }
            }
        }
    }

    fn generate_queen_pseudos(&self, side: Color, out: &mut Vec<Move>) {
        let queens = self.piece_bb(PieceKind::Queen, side);
        let mut bb = queens;
        while let Some(from) = crate::utils::pop_lsb(&mut bb) {
            // Generate moves along all 8 directions (diagonal + orthogonal)
            let directions = [8i8, -8i8, 1i8, -1i8, 9i8, -9i8, 7i8, -7i8]; // N,S,E,W,NE,SW,NW,SE

            for &dir in &directions {
                let mut to = from as i8;
                loop {
                    to += dir;

                    // Check if we're still on board
                    if to < 0 || to >= 64 {
                        break;
                    }

                    // Check board edge transitions for horizontal/diagonal moves
                    let from_sq = to - dir;
                    if from_sq < 0 || from_sq >= 64 {
                        break;
                    }
                    let from_file = from_sq % 8;

                    // Check wrapping issues
                    match dir {
                        1 => { if from_file == 7 { break; } }      // East: cannot wrap H->A
                        -1 => { if from_file == 0 { break; } }     // West: cannot wrap A->H
                        9 => { if from_file == 7 { break; } }      // NE: cannot wrap H->A
                        -9 => { if from_file == 0 { break; } }     // SW: cannot wrap A->H
                        7 => { if from_file == 0 { break; } }      // NW: cannot wrap A->H
                        -7 => { if from_file == 7 { break; } }     // SE: cannot wrap H->A
                        _ => {} // N,S moves don't have file wrapping issues
                    }

                    let to_usize = to as usize;

                    // Check if square is occupied
                    if self.is_occupied(to_usize) {
                        // If occupied by enemy, can capture
                        let enemy_occ = if side == Color::White { self.black_occ } else { self.white_occ };
                        if ((1u64 << to) & enemy_occ) != 0 {
                            if let Some((piece_kind, _)) = self.piece_on(to_usize) {
                                out.push(new_move(from, to_usize, PieceKind::Queen, Some(piece_kind), None, FLAG_CAPTURE));
                            }
                        }
                        break; // Stop sliding when we hit any piece
                    }

                    // Empty square - can move here
                    out.push(new_move(from, to_usize, PieceKind::Queen, None, None, FLAG_NONE));
                }
            }
        }
    }

    fn generate_king_pseudos(&self, side: Color, out: &mut Vec<Move>) {
        let kings = self.piece_bb(PieceKind::King, side);
        let mut bb = kings;
        while let Some(from) = crate::utils::pop_lsb(&mut bb) {
            // Generate king moves to all 8 adjacent squares
            let attacks = crate::utils::king_attacks(from);

            // Quiet moves (non-captures)
            let mut dest_bb = attacks & !self.occ;
            while let Some(to) = crate::utils::pop_lsb(&mut dest_bb) {
                out.push(new_move(from, to, PieceKind::King, None, None, FLAG_NONE));
            }

            // Captures
            let enemy_occ = if side == Color::White { self.black_occ } else { self.white_occ };
            let mut capture_bb = attacks & enemy_occ;
            while let Some(to) = crate::utils::pop_lsb(&mut capture_bb) {
                if let Some((piece_kind, _)) = self.piece_on(to) {
                    out.push(new_move(from, to, PieceKind::King, Some(piece_kind), None, FLAG_CAPTURE));
                }
            }

            // Castling moves
            self.generate_castling_moves(side, from, out);
        }
    }

    fn generate_castling_moves(&self, side: Color, king_from: usize, out: &mut Vec<Move>) {
        // Check if castling rights are available
        let castle_mask = if side == Color::White {
            0b1100u8 // White castling bits (4: K, 8: Q) but we use 8: K, 4: Q based on our mapping
        } else {
            0b0011u8  // Black castling bits (2: k, 1: q) but we use 2: k, 1: q based on our mapping
        };

        if (self.castling & castle_mask) == 0 {
            return; // No castling rights for this side
        }

        // King must be on the correct starting square
        let king_start_sq = if side == Color::White { 4 } else { 60 }; // e1 and e8
        if king_from != king_start_sq {
            return;
        }

        let enemy_occ = if side == Color::White { self.black_occ } else { self.white_occ };

        // Kingside castling
        let ks_mask = if side == Color::White { 0b1000u8 } else { 0b0010u8 };
        if (self.castling & ks_mask) != 0 {
            let (rook_start, king_to, _rook_to) = if side == Color::White {
                (7, 6, 5) // h1->f1, e1->g1, h1->f1
            } else {
                (63, 62, 61) // h8->f8, e8->g8, h8->f8
            };

            // Check if squares between king and rook are empty
            let squares_clear = match side {
                Color::White => {
                    !self.is_occupied(5) && !self.is_occupied(6) // f1, g1 empty
                }
                Color::Black => {
                    !self.is_occupied(61) && !self.is_occupied(62) // f8, g8 empty
                }
            };

            // Check if king and rook are on correct squares
            let rook_in_place = (self.piece_bb(PieceKind::Rook, side) & (1u64 << rook_start)) != 0;

            // Check if king path is safe (not under attack)
            let mut path_safe = true;
            if squares_clear && rook_in_place {
                let check_squares = match side {
                    Color::White => [4, 5, 6], // e1, f1, g1
                    Color::Black => [60, 61, 62], // e8, f8, g8
                };
                for &sq in &check_squares {
                    if self.is_square_attacked(sq, if side == Color::White { Color::Black } else { Color::White }) {
                        path_safe = false;
                        break;
                    }
                }
            } else {
                path_safe = false;
            }

            if path_safe {
                out.push(new_move(king_from, king_to, PieceKind::King, None, None, FLAG_CASTLE_KING));
            }
        }

        // Queenside castling
        let qs_mask = if side == Color::White { 0b0100u8 } else { 0b0001u8 };
        if (self.castling & qs_mask) != 0 {
            let (rook_start, king_to, _rook_to) = if side == Color::White {
                (0, 2, 3) // a1->d1, e1->c1, a1->d1
            } else {
                (56, 58, 59) // a8->d8, e8->c8, a8->d8
            };

            // Check if squares between king and rook are empty
            let squares_clear = match side {
                Color::White => {
                    !self.is_occupied(1) && !self.is_occupied(2) && !self.is_occupied(3) // b1, c1, d1 empty
                }
                Color::Black => {
                    !self.is_occupied(57) && !self.is_occupied(58) && !self.is_occupied(59) // b8, c8, d8 empty
                }
            };

            // Check if king and rook are on correct squares
            let rook_in_place = (self.piece_bb(PieceKind::Rook, side) & (1u64 << rook_start)) != 0;

            // Check if king path is safe
            let mut path_safe = true;
            if squares_clear && rook_in_place {
                let check_squares = match side {
                    Color::White => [4, 3, 2], // e1, d1, c1
                    Color::Black => [60, 59, 58], // e8, d8, c8
                };
                for &sq in &check_squares {
                    if self.is_square_attacked(sq, if side == Color::White { Color::Black } else { Color::White }) {
                        path_safe = false;
                        break;
                    }
                }
            } else {
                path_safe = false;
            }

            if path_safe {
                out.push(new_move(king_from, king_to, PieceKind::King, None, None, FLAG_CASTLE_QUEEN));
            }
        }
    }
}

// Helper conversioni FEN
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_make_unmake_zobrist_invariant() {
        crate::init();
        let fen = START_FEN; // use starting position to avoid promotions/complex cases for Zobrist invariant
        let mut board = Board::new();
        board.set_from_fen(fen).unwrap();
        let original_hash = board.zobrist;
        let pseudo_moves = board.generate_moves();
        for mv in pseudo_moves {
            let undo = board.make_move(mv);
            board.unmake_move(undo);
            assert_eq!(board.zobrist, original_hash, "Mismatched Zobrist after make/unmake for move {:?}", mv);
        }
    }
}

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
                    let sq = (7 - rank) * 8 + file;
                    self.set_piece(sq, kind, color);
                    file += 1;
                }
            }
            if rank == 0 { break; } else { rank -= 1; }
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