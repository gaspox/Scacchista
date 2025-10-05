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
pub fn iter_bits(bb: u64) -> BitIter { BitIter { bb } }

// Precomputed attack tables (lazy_static / once_cell logic will initialize once)
pub static mut KNIGHT_ATTACKS: [u64; 64] = [0; 64];
pub static mut KING_ATTACKS: [u64; 64] = [0; 64];
#[inline(always)]
pub fn init_attack_tables() {
    unsafe {
        // Knight attacks
        const KNIGHT_DELTAS: [i8; 8] = [-17, -15, -10, -6, 6, 10, 15, 17];
        for sq in 0..64 {
            let mut attacks = 0u64;
            for d in &KNIGHT_DELTAS {
                let file = sq % 8;
                let rank = sq / 8;
                let new_file_i8 = file as i8 + *d % 8;
                let new_rank = rank as i8 + *d / 8;
                if (0..8).contains(&new_file_i8) && (0..8).contains(&new_rank) {
                    attacks |= 1u64 << ((new_rank * 8 + new_file_i8) as usize);
                }
            }
            KNIGHT_ATTACKS[sq as usize] = attacks;
        }
        // King attacks
        const KING_DELTAS: [i8; 8] = [-9, -8, -7, -1, 1, 7, 8, 9];
        for sq in 0..64 {
            let mut attacks = 0u64;
            for d in &KING_DELTAS {
                let file = sq % 8;
                let rank = sq / 8;
                let new_file = (file as i8 + *d) % 8;
                let new_rank = rank as i8 + *d / 8;
                if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                    attacks |= 1u64 << ((new_rank * 8 + new_file) as usize);
                }
            }
            KING_ATTACKS[sq as usize] = attacks;
        }
    }
}
#[inline]
pub fn knight_attacks(sq: usize) -> u64 { unsafe { KNIGHT_ATTACKS[sq] } }
#[inline]
pub fn king_attacks(sq: usize) -> u64 { unsafe { KING_ATTACKS[sq] } }