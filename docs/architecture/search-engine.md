# Search Engine

This document details the search algorithms implemented in Scacchista.

## Overview

The search engine uses iterative deepening alpha-beta with Principal Variation Search (PVS) and various pruning techniques.

**Location:** `src/search/search.rs`

## Algorithm Stack

```
Iterative Deepening
    │
    ▼
Aspiration Windows
    │
    ▼
Principal Variation Search (PVS)
    │
    ├─▶ Null-Move Pruning
    ├─▶ Late Move Reductions (LMR)
    ├─▶ Futility Pruning
    ├─▶ Check Extensions
    │
    ▼
Quiescence Search
    │
    ▼
Static Evaluation
```

## Iterative Deepening

Search proceeds depth by depth, allowing time management and move ordering improvements.

```rust
fn iterative_deepening(&mut self, max_depth: u8) -> (Move, i16) {
    let mut best_move = None;
    let mut best_score = -INFINITY;

    for depth in 1..=max_depth {
        let score = self.search_root(depth);
        if self.should_stop() { break; }

        best_move = self.pv[0];
        best_score = score;

        // Report info to UCI
        self.report_info(depth, score, &self.pv);
    }

    (best_move.unwrap(), best_score)
}
```

## Aspiration Windows

Narrow search windows around expected score for faster cutoffs.

```rust
fn search_with_aspiration(&mut self, depth: u8, prev_score: i16) -> i16 {
    let mut delta = 50;  // Initial window width
    let mut alpha = prev_score - delta;
    let mut beta = prev_score + delta;

    loop {
        let score = self.negamax_pv(depth, alpha, beta, 0);

        if score <= alpha {
            alpha = score - delta;
            delta *= 2;
        } else if score >= beta {
            beta = score + delta;
            delta *= 2;
        } else {
            return score;  // Score within window
        }
    }
}
```

## Principal Variation Search (PVS)

Assumes first move (from previous iteration) is best, searches others with null window.

```rust
fn negamax_pv(&mut self, depth: u8, alpha: i16, beta: i16, ply: u8) -> i16 {
    // Base cases
    if depth == 0 { return self.quiescence(alpha, beta, ply); }

    // TT probe
    if let Some(entry) = self.tt.probe(self.board.zobrist) {
        if entry.depth >= depth && entry.is_exact() {
            return entry.score;
        }
    }

    let moves = self.generate_and_order_moves();
    if moves.is_empty() {
        return if self.is_in_check() { -MATE + ply } else { 0 };
    }

    let mut best_score = -INFINITY;
    for (i, mv) in moves.iter().enumerate() {
        let undo = self.board.make_move(*mv);

        let score = if i == 0 {
            // First move: full window
            -self.negamax_pv(depth - 1, -beta, -alpha, ply + 1)
        } else {
            // Other moves: null window first
            let score = -self.negamax_pv(depth - 1, -alpha - 1, -alpha, ply + 1);
            if score > alpha && score < beta {
                // Re-search with full window
                -self.negamax_pv(depth - 1, -beta, -alpha, ply + 1)
            } else {
                score
            }
        };

        self.board.unmake_move(*mv, &undo);

        if score > best_score {
            best_score = score;
            if score > alpha {
                alpha = score;
                if score >= beta {
                    // Beta cutoff
                    self.update_killers(*mv, ply);
                    self.update_history(*mv, depth);
                    break;
                }
            }
        }
    }

    // TT store
    self.tt.store(self.board.zobrist, best_score, depth, ...);

    best_score
}
```

## Pruning Techniques

### Null-Move Pruning

If passing (doing nothing) still causes a beta cutoff, the position is likely winning.

```rust
// Null-move pruning
if !is_pv && !in_check && depth >= 3 && has_non_pawn_material {
    let R = 2;  // Reduction
    self.board.make_null_move();
    let score = -self.negamax_pv(depth - 1 - R, -beta, -beta + 1, ply + 1);
    self.board.unmake_null_move();

    if score >= beta {
        return beta;  // Null-move cutoff
    }
}
```

**Conditions:**
- Not in PV node
- Not in check
- Depth >= 3
- Has non-pawn material (avoid zugzwang)

### Late Move Reductions (LMR)

Later moves in the move list are searched with reduced depth.

```rust
fn lmr_reduction(depth: u8, move_index: usize, is_quiet: bool) -> u8 {
    if depth < 3 || move_index < 4 || !is_quiet {
        return 0;
    }

    // Logarithmic reduction formula
    let reduction = (0.75 + (depth as f32).ln() * (move_index as f32).ln() / 2.25) as u8;
    reduction.min(depth - 1)
}
```

**Conditions:**
- Depth >= 3
- Move index >= 4 (not first few moves)
- Quiet move (not capture/promotion)
- Re-search if reduced search returns > alpha

### Futility Pruning

Skip moves that cannot improve alpha at low depths.

```rust
// Futility pruning at depth 1-2
if !in_check && depth <= 2 && !is_capture {
    let futility_margin = [0, 200, 300][depth];
    let static_eval = self.evaluate();

    if static_eval + futility_margin <= alpha {
        continue;  // Skip this move
    }
}
```

### Check Extensions

Extend search when in check to find forced mates.

```rust
let in_check = self.is_in_check();
let mut search_depth = depth - 1;

if in_check && ply < 10 {
    search_depth += 1;  // Extend by 1 ply
}
```

**Limit:** Only when ply < 10 to prevent search explosion.

## Quiescence Search

Searches captures/promotions until position is "quiet".

```rust
fn quiescence(&mut self, mut alpha: i16, beta: i16, ply: u8) -> i16 {
    // Stand-pat: can we do nothing and still be good?
    let stand_pat = self.evaluate();
    if stand_pat >= beta { return beta; }
    if stand_pat > alpha { alpha = stand_pat; }

    let moves = self.generate_moves();

    // Check for mate/stalemate
    if moves.is_empty() {
        return if self.is_in_check() { -MATE + ply } else { 0 };
    }

    // In check: search all evasions
    // Not in check: search only captures/promotions
    let moves_to_search = if self.is_in_check() {
        moves
    } else {
        moves.into_iter().filter(|m| is_capture(*m) || is_promotion(*m)).collect()
    };

    for mv in moves_to_search {
        let undo = self.board.make_move(mv);
        let score = -self.quiescence(-beta, -alpha, ply + 1);
        self.board.unmake_move(mv, &undo);

        if score >= beta { return beta; }
        if score > alpha { alpha = score; }
    }

    alpha
}
```

**Optimization:** Uses `evaluate_fast()` (material + PSQT only) for speed.

## Transposition Table

Hash table storing previously searched positions.

**Entry structure:**

```rust
struct TTEntry {
    key: u64,           // Zobrist hash (verification)
    best_move: Move,    // Best move found
    score: i16,         // Evaluation score
    depth: u8,          // Search depth
    flag: NodeType,     // Exact/LowerBound/UpperBound
    age: u8,            // For replacement
}
```

**Operations:**
- `probe(zobrist)` - Look up position
- `store(zobrist, score, depth, flag, best_move)` - Store result

**Replacement scheme:**
- Always replace if same position
- Prefer deeper entries
- Age-based replacement for old entries

See `src/search/tt.rs` for implementation.

## Move Ordering

Good move ordering is critical for alpha-beta efficiency.

**Order (highest priority first):**
1. TT move (from previous search)
2. Captures ordered by MVV-LVA + SEE
3. Promotions
4. Killer moves (2 per ply)
5. History heuristic score
6. Remaining moves

### MVV-LVA (Most Valuable Victim - Least Valuable Attacker)

```rust
fn mvv_lva_score(victim: PieceKind, attacker: PieceKind) -> i16 {
    PIECE_VALUE[victim] * 10 - PIECE_VALUE[attacker]
}
// QxP scores higher than PxQ (capturing queen with pawn is great)
```

### Killer Moves

Quiet moves that caused beta cutoffs at the same ply.

```rust
struct Killers {
    moves: [[Option<Move>; 2]; MAX_PLY],
}
```

### History Heuristic

Tracks which quiet moves have been successful historically.

```rust
struct History {
    table: [[u32; 64]; 12],  // [piece][to_square]
}

fn update_history(&mut self, mv: Move, depth: u8) {
    let piece = move_piece(mv);
    let to = move_to_sq(mv);
    self.history.table[piece][to] += (depth * depth) as u32;
}
```

## Performance Tuning

### Depth 6 Benchmark

| Component | Before | After | Improvement |
|-----------|--------|-------|-------------|
| Full search | 1763ms | 568ms | **3.1x** |
| Evaluation | 40% | 20% | 2x faster |

### Key Optimizations Applied

1. **Cache is_in_check()** - Avoid duplicate expensive calls
2. **evaluate_fast() in qsearch** - Skip king safety, development in leaves
3. **Incremental Zobrist** - Update hash incrementally on make/unmake

## Search Statistics

Track via `SearchStats`:

```rust
struct SearchStats {
    nodes: u64,           // Total nodes searched
    tt_hits: u64,         // TT probe successes
    tt_cutoffs: u64,      // TT caused immediate return
    beta_cutoffs: u64,    // Beta cutoffs
    null_cutoffs: u64,    // Null-move cutoffs
    lmr_researches: u64,  // LMR re-searches needed
}
```

---

**Related Documents:**
- [Architecture Overview](./overview.md)
- [Evaluation](./evaluation.md)
- [Threading](./threading.md)
- [Performance Reference](../reference/performance.md)
