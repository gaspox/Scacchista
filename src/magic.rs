//! Magic Bitboards for O(1) sliding piece attack generation
//!
//! This module implements "magic bitboards" - a technique that uses
//! precomputed lookup tables to generate sliding piece attacks in O(1).
//!
//! The key insight is that for any square and occupancy pattern,
//! there's a unique attack pattern. We use "magic numbers" to hash
//! the relevant occupancy bits into a table index.

use std::sync::OnceLock;

// ============================================================================
// MAGIC NUMBERS (from Chess Programming Wiki / Stockfish)
// ============================================================================

/// Rook magic numbers - one per square
/// These are carefully chosen constants that produce perfect hashing
const ROOK_MAGICS: [u64; 64] = [
    0x0080001020400080, 0x0040001000200040, 0x0080081000200080, 0x0080040800100080,
    0x0080020400080080, 0x0080010200040080, 0x0080008001000200, 0x0080002040800100,
    0x0000800020400080, 0x0000400020005000, 0x0000801000200080, 0x0000800800100080,
    0x0000800400080080, 0x0000800200040080, 0x0000800100020080, 0x0000800040800100,
    0x0000208000400080, 0x0000404000201000, 0x0000808010002000, 0x0000808008001000,
    0x0000808004000800, 0x0000808002000400, 0x0000010100020004, 0x0000020000408104,
    0x0000208080004000, 0x0000200040005000, 0x0000100080200080, 0x0000080080100080,
    0x0000040080080080, 0x0000020080040080, 0x0000010080800200, 0x0000800080004100,
    0x0000204000800080, 0x0000200040401000, 0x0000100080802000, 0x0000080080801000,
    0x0000040080800800, 0x0000020080800400, 0x0000020001010004, 0x0000800040800100,
    0x0000204000808000, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
    0x0000040008008080, 0x0000020004008080, 0x0000010002008080, 0x0000004081020004,
    0x0000204000800080, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
    0x0000040008008080, 0x0000020004008080, 0x0000800100020080, 0x0000800041000080,
    0x00FFFCDDFCED714A, 0x007FFCDDFCED714A, 0x003FFFCDFFD88096, 0x0000040810002101,
    0x0001000204080011, 0x0001000204000801, 0x0001000082000401, 0x0001FFFAABFAD1A2,
];

/// Bishop magic numbers - one per square
const BISHOP_MAGICS: [u64; 64] = [
    0x0002020202020200, 0x0002020202020000, 0x0004010202000000, 0x0004040080000000,
    0x0001104000000000, 0x0000821040000000, 0x0000410410400000, 0x0000104104104000,
    0x0000040404040400, 0x0000020202020200, 0x0000040102020000, 0x0000040400800000,
    0x0000011040000000, 0x0000008210400000, 0x0000004104104000, 0x0000002082082000,
    0x0004000808080800, 0x0002000404040400, 0x0001000202020200, 0x0000800802004000,
    0x0000800400A00000, 0x0000200100884000, 0x0000400082082000, 0x0000200041041000,
    0x0002080010101000, 0x0001040008080800, 0x0000208004010400, 0x0000404004010200,
    0x0000840000802000, 0x0000404002011000, 0x0000808001041000, 0x0000404000820800,
    0x0001041000202000, 0x0000820800101000, 0x0000104400080800, 0x0000020080080080,
    0x0000404040040100, 0x0000808100020100, 0x0001010100020800, 0x0000808080010400,
    0x0000820820004000, 0x0000410410002000, 0x0000082088001000, 0x0000002011000800,
    0x0000080100400400, 0x0001010101000200, 0x0002020202000400, 0x0001010101000200,
    0x0000410410400000, 0x0000208208200000, 0x0000002084100000, 0x0000000020880000,
    0x0000001002020000, 0x0000040408020000, 0x0004040404040000, 0x0002020202020000,
    0x0000104104104000, 0x0000002082082000, 0x0000000020841000, 0x0000000000208800,
    0x0000000010020200, 0x0000000404080200, 0x0000040404040400, 0x0002020202020200,
];

/// Rook shift amounts (64 - number of relevant bits)
const ROOK_SHIFTS: [u8; 64] = [
    52, 53, 53, 53, 53, 53, 53, 52,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    52, 53, 53, 53, 53, 53, 53, 52,
];

/// Bishop shift amounts
const BISHOP_SHIFTS: [u8; 64] = [
    58, 59, 59, 59, 59, 59, 59, 58,
    59, 59, 59, 59, 59, 59, 59, 59,
    59, 59, 57, 57, 57, 57, 59, 59,
    59, 59, 57, 55, 55, 57, 59, 59,
    59, 59, 57, 55, 55, 57, 59, 59,
    59, 59, 57, 57, 57, 57, 59, 59,
    59, 59, 59, 59, 59, 59, 59, 59,
    58, 59, 59, 59, 59, 59, 59, 58,
];

// ============================================================================
// ATTACK TABLES
// ============================================================================

/// Total size needed for rook attack table entries
const ROOK_TABLE_SIZE: usize = 102400; // Sum of 2^(64-shift) for all squares

/// Total size needed for bishop attack table entries  
const BISHOP_TABLE_SIZE: usize = 5248;

/// Magic entry for a single square
#[derive(Clone, Copy)]
struct MagicEntry {
    mask: u64,      // Relevant occupancy mask (excludes edges)
    magic: u64,     // Magic number
    shift: u8,      // Shift amount (64 - bits)
    offset: usize,  // Offset into attack table
}

/// Global magic tables
struct MagicTables {
    rook_entries: [MagicEntry; 64],
    bishop_entries: [MagicEntry; 64],
    rook_attacks: Vec<u64>,
    bishop_attacks: Vec<u64>,
}

static MAGIC_TABLES: OnceLock<MagicTables> = OnceLock::new();

// ============================================================================
// MASK GENERATION (relevant occupancy squares, excluding edges)
// ============================================================================

/// Generate rook mask for a square (relevant blockers, excluding edge squares)
fn rook_mask(sq: usize) -> u64 {
    let mut mask = 0u64;
    let rank = sq / 8;
    let file = sq % 8;
    
    // North (exclude rank 7)
    for r in (rank + 1)..7 {
        mask |= 1u64 << (r * 8 + file);
    }
    // South (exclude rank 0)
    for r in 1..rank {
        mask |= 1u64 << (r * 8 + file);
    }
    // East (exclude file H)
    for f in (file + 1)..7 {
        mask |= 1u64 << (rank * 8 + f);
    }
    // West (exclude file A)
    for f in 1..file {
        mask |= 1u64 << (rank * 8 + f);
    }
    
    mask
}

/// Generate bishop mask for a square
fn bishop_mask(sq: usize) -> u64 {
    let mut mask = 0u64;
    let rank = sq / 8;
    let file = sq % 8;
    
    // NE
    let (mut r, mut f) = (rank + 1, file + 1);
    while r < 7 && f < 7 {
        mask |= 1u64 << (r * 8 + f);
        r += 1;
        f += 1;
    }
    // NW
    let (mut r, mut f) = (rank + 1, file.wrapping_sub(1));
    while r < 7 && f > 0 && f < 8 {
        mask |= 1u64 << (r * 8 + f);
        r += 1;
        f = f.wrapping_sub(1);
    }
    // SE
    let (mut r, mut f) = (rank.wrapping_sub(1), file + 1);
    while r > 0 && r < 8 && f < 7 {
        mask |= 1u64 << (r * 8 + f);
        r = r.wrapping_sub(1);
        f += 1;
    }
    // SW
    let (mut r, mut f) = (rank.wrapping_sub(1), file.wrapping_sub(1));
    while r > 0 && r < 8 && f > 0 && f < 8 {
        mask |= 1u64 << (r * 8 + f);
        r = r.wrapping_sub(1);
        f = f.wrapping_sub(1);
    }
    
    mask
}

// ============================================================================
// ATTACK GENERATION (used to build tables)
// ============================================================================

/// Generate rook attacks for a given square and occupancy (slow, for table building)
fn rook_attacks_slow(sq: usize, occ: u64) -> u64 {
    let mut attacks = 0u64;
    let rank = sq / 8;
    let file = sq % 8;
    
    // North
    for r in (rank + 1)..8 {
        let bit = 1u64 << (r * 8 + file);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
    // South
    for r in (0..rank).rev() {
        let bit = 1u64 << (r * 8 + file);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
    // East
    for f in (file + 1)..8 {
        let bit = 1u64 << (rank * 8 + f);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
    // West
    for f in (0..file).rev() {
        let bit = 1u64 << (rank * 8 + f);
        attacks |= bit;
        if occ & bit != 0 { break; }
    }
    
    attacks
}

/// Generate bishop attacks for a given square and occupancy (slow, for table building)
fn bishop_attacks_slow(sq: usize, occ: u64) -> u64 {
    let mut attacks = 0u64;
    let rank = sq / 8;
    let file = sq % 8;
    
    // NE
    let (mut r, mut f) = (rank + 1, file + 1);
    while r < 8 && f < 8 {
        let bit = 1u64 << (r * 8 + f);
        attacks |= bit;
        if occ & bit != 0 { break; }
        r += 1;
        f += 1;
    }
    // NW
    let (mut r, mut f) = (rank + 1, file.wrapping_sub(1));
    while r < 8 && f < 8 {
        let bit = 1u64 << (r * 8 + f);
        attacks |= bit;
        if occ & bit != 0 { break; }
        r += 1;
        f = f.wrapping_sub(1);
    }
    // SE
    let (mut r, mut f) = (rank.wrapping_sub(1), file + 1);
    while r < 8 && f < 8 {
        let bit = 1u64 << (r * 8 + f);
        attacks |= bit;
        if occ & bit != 0 { break; }
        r = r.wrapping_sub(1);
        f += 1;
    }
    // SW
    let (mut r, mut f) = (rank.wrapping_sub(1), file.wrapping_sub(1));
    while r < 8 && f < 8 {
        let bit = 1u64 << (r * 8 + f);
        attacks |= bit;
        if occ & bit != 0 { break; }
        r = r.wrapping_sub(1);
        f = f.wrapping_sub(1);
    }
    
    attacks
}

// ============================================================================
// TABLE INITIALIZATION
// ============================================================================

/// Generate all blocker subsets for a given mask
fn enumerate_subsets(mask: u64) -> Vec<u64> {
    let mut subsets = Vec::new();
    let mut subset = 0u64;
    loop {
        subsets.push(subset);
        subset = subset.wrapping_sub(mask) & mask;
        if subset == 0 { break; }
    }
    subsets
}

/// Initialize all magic tables
fn init_magic_tables() -> MagicTables {
    let mut rook_entries = [MagicEntry { mask: 0, magic: 0, shift: 0, offset: 0 }; 64];
    let mut bishop_entries = [MagicEntry { mask: 0, magic: 0, shift: 0, offset: 0 }; 64];
    let mut rook_attacks = vec![0u64; ROOK_TABLE_SIZE];
    let mut bishop_attacks = vec![0u64; BISHOP_TABLE_SIZE];
    
    let mut rook_offset = 0usize;
    let mut bishop_offset = 0usize;
    
    for sq in 0..64 {
        // Rook
        let mask = rook_mask(sq);
        let magic = ROOK_MAGICS[sq];
        let shift = ROOK_SHIFTS[sq];
        let table_size = 1 << (64 - shift);
        
        rook_entries[sq] = MagicEntry { mask, magic, shift, offset: rook_offset };
        
        for occ in enumerate_subsets(mask) {
            let attacks = rook_attacks_slow(sq, occ);
            let index = ((occ.wrapping_mul(magic)) >> shift) as usize;
            rook_attacks[rook_offset + index] = attacks;
        }
        
        rook_offset += table_size;
        
        // Bishop
        let mask = bishop_mask(sq);
        let magic = BISHOP_MAGICS[sq];
        let shift = BISHOP_SHIFTS[sq];
        let table_size = 1 << (64 - shift);
        
        bishop_entries[sq] = MagicEntry { mask, magic, shift, offset: bishop_offset };
        
        for occ in enumerate_subsets(mask) {
            let attacks = bishop_attacks_slow(sq, occ);
            let index = ((occ.wrapping_mul(magic)) >> shift) as usize;
            bishop_attacks[bishop_offset + index] = attacks;
        }
        
        bishop_offset += table_size;
    }
    
    MagicTables {
        rook_entries,
        bishop_entries,
        rook_attacks,
        bishop_attacks,
    }
}

/// Initialize magic tables (thread-safe, called once)
#[inline(always)]
pub fn init() {
    MAGIC_TABLES.get_or_init(init_magic_tables);
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Get rook attacks for a square given board occupancy
#[inline]
pub fn rook_attacks(sq: usize, occ: u64) -> u64 {
    let tables = MAGIC_TABLES.get().expect("Magic tables not initialized. Call magic::init() first.");
    let entry = &tables.rook_entries[sq];
    let masked = occ & entry.mask;
    let index = ((masked.wrapping_mul(entry.magic)) >> entry.shift) as usize;
    tables.rook_attacks[entry.offset + index]
}

/// Get bishop attacks for a square given board occupancy
#[inline]
pub fn bishop_attacks(sq: usize, occ: u64) -> u64 {
    let tables = MAGIC_TABLES.get().expect("Magic tables not initialized. Call magic::init() first.");
    let entry = &tables.bishop_entries[sq];
    let masked = occ & entry.mask;
    let index = ((masked.wrapping_mul(entry.magic)) >> entry.shift) as usize;
    tables.bishop_attacks[entry.offset + index]
}

/// Get queen attacks (bishop + rook)
#[inline]
pub fn queen_attacks(sq: usize, occ: u64) -> u64 {
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rook_attacks_corner() {
        init();
        // Rook on a1 (sq=0), empty board
        let attacks = rook_attacks(0, 0);
        // Should attack a2-a8 (7 squares) and b1-h1 (7 squares) = 14 squares
        assert_eq!(attacks.count_ones(), 14);
    }

    #[test]
    fn test_rook_attacks_with_blocker() {
        init();
        // Rook on a1, blocker on a4
        let blocker = 1u64 << 24; // a4
        let attacks = rook_attacks(0, blocker);
        // North: a2, a3, a4 (stopped), East: b1-h1
        // Total: 3 + 7 = 10
        assert_eq!(attacks.count_ones(), 10);
    }

    #[test]
    fn test_bishop_attacks_center() {
        init();
        // Bishop on d4 (sq=27), empty board
        let attacks = bishop_attacks(27, 0);
        // d4 has diagonals reaching corners
        assert!(attacks.count_ones() >= 13);
    }

    #[test]
    fn test_queen_attacks() {
        init();
        // Queen = rook + bishop
        let occ = 0u64;
        let rook_atk = rook_attacks(27, occ);
        let bishop_atk = bishop_attacks(27, occ);
        let queen_atk = queen_attacks(27, occ);
        assert_eq!(queen_atk, rook_atk | bishop_atk);
    }
}
