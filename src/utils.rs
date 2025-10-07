// Bitboard masks, iterators and helpers for move generation and search

// File masks (A is column 0, H column 7)
pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_C: u64 = 0x0404040404040404;
pub const FILE_D: u64 = 0x0808080808080808;
pub const FILE_E: u64 = 0x1010101010101010;
pub const FILE_F: u64 = 0x2020202020202020;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

pub const NOT_FILE_A: u64 = !FILE_A;
pub const NOT_FILE_H: u64 = !FILE_H;

// Rank masks (A1 is square 0)
pub const RANK_1: u64 = 0x00000000000000FF;
pub const RANK_2: u64 = 0x000000000000FF00;
pub const RANK_3: u64 = 0x0000000000FF0000;
pub const RANK_4: u64 = 0x00000000FF000000;
pub const RANK_5: u64 = 0x000000FF00000000;
pub const RANK_6: u64 = 0x0000FF0000000000;
pub const RANK_7: u64 = 0x00FF000000000000;
pub const RANK_8: u64 = 0xFF00000000000000;

// Direction deltas for sliding moves
pub const NORTH: i8 = 8;
pub const SOUTH: i8 = -8;
pub const EAST: i8 = 1;
pub const WEST: i8 = -1;
pub const NORTH_EAST: i8 = 9;
pub const NORTH_WEST: i8 = 7;
pub const SOUTH_EAST: i8 = -7;
pub const SOUTH_WEST: i8 = -9;

// Bit operations
#[inline]
pub fn pop_lsb(bb: &mut u64) -> Option<usize> {
    if *bb == 0 {
        return None;
    }
    let lsb = bb.trailing_zeros() as usize;
    *bb &= *bb - 1;
    Some(lsb)
}
#[inline]
pub fn lsb_index(bb: u64) -> Option<usize> {
    if bb == 0 {
        None
    } else {
        Some(bb.trailing_zeros() as usize)
    }
}
#[inline]
pub fn count_bits(bb: u64) -> u32 {
    bb.count_ones()
}
pub struct BitIter {
    bb: u64,
}
impl Iterator for BitIter {
    type Item = usize;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        pop_lsb(&mut self.bb)
    }
}
#[inline]
pub fn iter_bits(bb: u64) -> BitIter {
    BitIter { bb }
}

// Precomputed attack tables using OnceLock for thread safety
use std::sync::OnceLock;

static KNIGHT_ATTACKS: OnceLock<[u64; 64]> = OnceLock::new();
static KING_ATTACKS: OnceLock<[u64; 64]> = OnceLock::new();

fn init_knight_attacks() -> [u64; 64] {
    const KNIGHT_OFFSETS: [(i8, i8); 8] = [
        (-2, -1),
        (-2, 1),
        (-1, -2),
        (-1, 2),
        (1, -2),
        (1, 2),
        (2, -1),
        (2, 1),
    ];
    let mut attacks = [0u64; 64];

    for sq in 0..64 {
        let file = sq % 8;
        let rank = sq / 8;
        let mut attack_mask = 0u64;

        for (dx, dy) in &KNIGHT_OFFSETS {
            let new_file = file as i8 + dx;
            let new_rank = rank as i8 + dy;
            if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                let target_sq = (new_rank as usize) * 8 + (new_file as usize);
                attack_mask |= 1u64 << target_sq;
            }
        }
        attacks[sq] = attack_mask;
    }
    attacks
}

fn init_king_attacks() -> [u64; 64] {
    const KING_OFFSETS: [(i8, i8); 8] = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];
    let mut attacks = [0u64; 64];

    for sq in 0..64 {
        let file = sq % 8;
        let rank = sq / 8;
        let mut attack_mask = 0u64;

        for (dx, dy) in &KING_OFFSETS {
            let new_file = file as i8 + dx;
            let new_rank = rank as i8 + dy;
            if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                let target_sq = (new_rank as usize) * 8 + (new_file as usize);
                attack_mask |= 1u64 << target_sq;
            }
        }
        attacks[sq] = attack_mask;
    }
    attacks
}

#[inline(always)]
pub fn init_attack_tables() {
    // Initialize both tables if not already done
    KNIGHT_ATTACKS.get_or_init(|| init_knight_attacks());
    KING_ATTACKS.get_or_init(|| init_king_attacks());
}

#[inline]
pub fn knight_attacks(sq: usize) -> u64 {
    let table = KNIGHT_ATTACKS.get().unwrap_or_else(|| {
        init_attack_tables();
        KNIGHT_ATTACKS.get().unwrap()
    });
    table[sq]
}

#[inline]
pub fn king_attacks(sq: usize) -> u64 {
    let table = KING_ATTACKS.get().unwrap_or_else(|| {
        init_attack_tables();
        KING_ATTACKS.get().unwrap()
    });
    table[sq]
}
