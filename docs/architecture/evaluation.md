# Evaluation Function

This document describes the Hand-Crafted Evaluation (HCE) function used by Scacchista.

## Overview

The evaluation function estimates the value of a chess position from the perspective of the side to move. Positive scores favor the side to move, negative scores favor the opponent.

**Location:** `src/eval.rs`

**Convention:** Centipawns (cp), where 100cp = 1 pawn value.

## Evaluation Components

```
Total Score = Material + PSQT + King Safety + Development + Center Control
```

| Component | Weight | Description |
|-----------|--------|-------------|
| Material | Base | Piece values |
| PSQT | Variable | Position-dependent bonuses |
| King Safety | -50 to +95 | Castling, pawn shield |
| Development | -40 to 0 | Penalty for undeveloped pieces |
| Center Control | Variable | Bonus for central squares |

## Material Values

Standard piece values used:

| Piece | Value (cp) |
|-------|-----------|
| Pawn | 100 |
| Knight | 320 |
| Bishop | 330 |
| Rook | 500 |
| Queen | 900 |
| King | - |

```rust
const PIECE_VALUES: [i16; 6] = [100, 320, 330, 500, 900, 0];
```

## Piece-Square Tables (PSQT)

Each piece has a 64-entry table giving positional bonuses/penalties.

### Pawn PSQT

Encourages central pawns and advancement toward promotion.

```
Rank 8:  0   0   0   0   0   0   0   0   (impossible)
Rank 7: 50  50  50  50  50  50  50  50   (near promotion)
Rank 6: 20  20  20  25  25  20  20  20
Rank 5: 10  10  15  20  20  15  10  10
Rank 4:  5   5  10  15  15  10   5   5   (central control)
Rank 3: 10  10  20  30  30  20  10  10   (push encouraged)
Rank 2:  5   5   5   5   5   5   5   5
Rank 1:  0   0   0   0   0   0   0   0   (impossible)
        a   b   c   d   e   f   g   h
```

### Knight PSQT

Encourages centralization, penalizes rim knights.

```
Rank 8: -50 -40 -30 -30 -30 -30 -40 -50
Rank 7: -40 -20   0   0   0   0 -20 -40
Rank 6: -30   0  10  15  15  10   0 -30
Rank 5: -30   5  15  20  20  15   5 -30  (central squares best)
Rank 4: -30   0  15  20  20  15   0 -30
Rank 3: -30   5  10  15  15  10   5 -30
Rank 2: -40 -20   0   5   5   0 -20 -40
Rank 1: -50 -40 -30 -30 -30 -30 -40 -50
         a   b   c   d   e   f   g   h
```

### King PSQT

Encourages castled position, penalizes exposed king.

```
Rank 8: -30 -40 -40 -50 -50 -40 -40 -30
Rank 7: -30 -40 -40 -50 -50 -40 -40 -30
...
Rank 2:  20  20   0   0   0   0  20  20
Rank 1:  20  30  10   0   0  10  30  20  (g1/c1 = castled position)
         a   b   c   d   e   f   g   h
```

### PSQT Symmetry

Black's PSQT is computed by flipping the square index:

```rust
fn psqt_score(board: &Board) -> i16 {
    let mut score = 0;

    for piece in 0..6 {
        let mut white_bb = board.piece_bb(piece, Color::White);
        while white_bb != 0 {
            let sq = white_bb.trailing_zeros() as usize;
            white_bb &= white_bb - 1;
            score += PSQT[piece][sq];
        }

        let mut black_bb = board.piece_bb(piece, Color::Black);
        while black_bb != 0 {
            let sq = black_bb.trailing_zeros() as usize;
            black_bb &= black_bb - 1;
            score -= PSQT[piece][sq ^ 56];  // Flip for black
        }
    }

    score
}
```

## King Safety

Evaluates the safety of both kings.

### Components

1. **Exposed King Penalty** (-50cp)
   - Applied when king is in columns d/e (center)
   - Not applied if king has castled

2. **Pawn Shield Bonus** (+15cp per pawn, max +45cp)
   - Counts pawns in the 3 squares in front of the king
   - Maximum 3 pawns = 45cp bonus

3. **Castled Detection**
   - White: g1 (kingside) or c1 (queenside)
   - Black: g8 (kingside) or c8 (queenside)

```rust
fn king_safety(board: &Board, color: Color) -> i16 {
    let king_sq = board.king_sq(color);
    let mut safety = 0;

    // Exposed king in center penalty
    let file = king_sq % 8;
    if (file == 3 || file == 4) && !has_castled(board, king_sq, color) {
        safety -= 50;
    }

    // Pawn shield bonus
    let shield_count = count_pawn_shield(board, king_sq, color);
    safety += shield_count * 15;

    safety
}
```

### Total King Safety Range

- Worst case: -50cp (exposed, no shield)
- Best case: +45cp (castled with full shield)
- Delta: 95cp

## Development Penalty

Penalizes undeveloped minor pieces (knights/bishops) after move 10.

```rust
fn development_penalty(board: &Board) -> i16 {
    if board.fullmove <= 10 {
        return 0;  // No penalty in opening
    }

    const PENALTY_PER_PIECE: i16 = 10;

    let mut penalty = 0;

    // White pieces on rank 1
    let white_knights = board.piece_bb(Knight, White) & RANK_1_MASK;
    let white_bishops = board.piece_bb(Bishop, White) & RANK_1_MASK;
    penalty += (white_knights.count_ones() + white_bishops.count_ones()) as i16 * PENALTY_PER_PIECE;

    // Black pieces on rank 8
    let black_knights = board.piece_bb(Knight, Black) & RANK_8_MASK;
    let black_bishops = board.piece_bb(Bishop, Black) & RANK_8_MASK;
    penalty -= (black_knights.count_ones() + black_bishops.count_ones()) as i16 * PENALTY_PER_PIECE;

    penalty
}
```

**Range:** -40cp to +40cp (4 undeveloped pieces max per side)

## Center Control

Bonus for attacking central squares.

```rust
fn center_control(board: &Board) -> i16 {
    let mut score = 0;

    // Main center: d4, e4, d5, e5 (+10cp each)
    for sq in [27, 28, 35, 36] {  // d4, e4, d5, e5
        if board.is_square_attacked(sq, White) { score += 10; }
        if board.is_square_attacked(sq, Black) { score -= 10; }
    }

    // Extended center (+3cp each)
    for sq in EXTENDED_CENTER {
        if board.is_square_attacked(sq, White) { score += 3; }
        if board.is_square_attacked(sq, Black) { score -= 3; }
    }

    score
}
```

**Note:** This is computationally expensive (32 `is_square_attacked` calls). Consider caching or limiting to opening positions.

## Fast Evaluation

For quiescence search, use simplified evaluation (material + PSQT only):

```rust
pub fn evaluate_fast(board: &Board) -> i16 {
    let material = material_score(board);
    let psqt = psqt_score(board);

    // Adjust for side to move
    let score = material + psqt;
    if board.side == Color::Black { -score } else { score }
}
```

**Performance:** ~1us vs ~3us for full evaluation.

## Evaluation Function Entry Point

```rust
pub fn evaluate(board: &Board, ply: u8) -> i16 {
    let material = material_score(board);
    let psqt = psqt_score(board);
    let king_safety_w = king_safety(board, Color::White);
    let king_safety_b = king_safety(board, Color::Black);
    let development = development_penalty(board);
    let center = center_control(board);

    let score = material + psqt
        + (king_safety_w - king_safety_b)
        + development
        + center;

    // Return from side-to-move perspective
    if board.side == Color::Black { -score } else { score }
}
```

## Mate Scores

Special scores for checkmate:

```rust
const MATE: i16 = 30000;
const MATED: i16 = -30000;

// Mate in N plies from root
fn mate_score(ply: u8) -> i16 {
    MATE - ply as i16
}

// Being mated in N plies
fn mated_score(ply: u8) -> i16 {
    MATED + ply as i16
}
```

## Future Improvements

Potential evaluation enhancements:

1. **Passed Pawns** (+30-50 ELO)
   - Detect with bitboard masks
   - Progressive bonus by rank

2. **Bishop Pair** (+10-15 ELO)
   - +30-50cp when having both bishops

3. **Tapered Evaluation** (+20-30 ELO)
   - Separate middlegame/endgame scores
   - Interpolate based on game phase

4. **Pawn Structure**
   - Doubled pawns penalty
   - Isolated pawns penalty
   - Backward pawns penalty

5. **Mobility**
   - Bonus for piece mobility
   - Penalty for trapped pieces

6. **NNUE** (Long-term)
   - Neural network evaluation
   - +200-400 ELO potential

---

**Related Documents:**
- [Architecture Overview](./overview.md)
- [Search Engine](./search-engine.md)
- [Performance Reference](../reference/performance.md)
