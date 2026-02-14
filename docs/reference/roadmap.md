# Development Roadmap

This document outlines planned improvements and future development for Scacchista.

## Current Status

**Version:** 0.5.0-dev
**ELO Estimate:** ~1400-1600
**Test Coverage:** 90+ tests passing

## Completed Milestones

### Phase 5: Tactical & Architectural Polish (Complete)

- [x] Dedicated Capture Generation in QSearch
- [x] Delta Pruning
- [x] Lock-Free Transposition Table
- [x] Bitboard-based static evaluation (HCE)
- [x] Draw detection (Threefold, 50-move, Insufficient material)
- [x] PVS (Principal Variation Search) at Root [v0.5.0-dev]

### Phase 1: Core Engine (Complete)

- [x] Board representation (bitboards)
- [x] Move generation (pseudo-legal + legal filter)
- [x] Make/unmake moves
- [x] Zobrist hashing
- [x] Basic alpha-beta search
- [x] Transposition table
- [x] Move ordering (MVV-LVA, killers, history)

### Phase 2: Search Enhancements (Complete)

- [x] Aspiration windows
- [x] Quiescence search
- [x] Null-move pruning
- [x] Late move reductions (LMR)
- [x] Futility pruning
- [x] Check extensions

### Phase 3: Infrastructure (Complete)

- [x] UCI interface
- [x] Time management
- [x] Multi-threading (Lazy-SMP)
- [x] Perft validation

### Phase 4: Evaluation (Complete)

- [x] Material evaluation
- [x] Piece-Square Tables (PSQT)
- [x] King safety (exposed penalty, pawn shield)
- [x] Development penalty
- [x] Center control
- [x] evaluate_fast() for qsearch

## Short-Term Goals (1-2 Weeks)

### High Priority

#### 1. SEE Cache Array

**Impact:** ~5-10% speedup
**Effort:** 30 minutes
**Status:** Planned

Replace HashMap with fixed array:
```rust
// Current
let see_cache: HashMap<Move, i16>

// Proposed
let see_cache: [Option<i16>; 64]
```

#### 2. Razoring

**Impact:** ~2-3% speedup
**Effort:** 1 hour
**Status:** Planned

Quick pruning at low depths:
```rust
if depth <= 2 && !in_check && static_eval + margin < alpha {
    return static_eval;
}
```

#### 3. Lazy-SMP Diversity

**Impact:** +50-80% speedup on 2 threads
**Effort:** 1-2 days
**Status:** Infrastructure ready

Options:
- Per-worker history tables
- Different aspiration windows
- Random eval noise

### Medium Priority

#### 4. Endgame Recognition

**Impact:** +60-80 ELO
**Effort:** 1-2 hours
**Status:** Not started

Simple endgames:
- KQ vs K = winning
- KR vs K = winning
- KB+KN vs K = winning

## Medium-Term Goals (1-2 Months)

### Search Improvements

#### 6. Magic Bitboards

**Impact:** 3-5x speedup in move generation
**Effort:** 1-2 weeks
**Status:** Not started

Replace loop-based sliding piece generation with magic bitboard lookups.

#### 7. Passed Pawn Evaluation

**Impact:** +30-50 ELO
**Effort:** 2-3 days
**Status:** Not started

Features:
- Detection via bitboard masks
- Progressive bonus by rank
- Connected passed pawns

#### 8. Tapered Evaluation

**Impact:** +20-30 ELO
**Effort:** 2-3 days
**Status:** Not started

Separate middlegame/endgame scores with interpolation:
```rust
let phase = calculate_phase(board);  // 0-256
let score = (mg_score * phase + eg_score * (256 - phase)) / 256;
```

#### 9. Bishop Pair Bonus

**Impact:** +10-15 ELO
**Effort:** 1 hour
**Status:** Not started

Simple:
```rust
if bishops.count_ones() == 2 {
    score += 30;  // ~30cp bonus
}
```

### Infrastructure Improvements

#### 10. Pawn Hash Table

**Impact:** ~10-15% speedup
**Effort:** 1 week
**Status:** Not started

Cache pawn structure evaluation separately.

#### 11. Multi-PV Support

**Impact:** Better analysis
**Effort:** 3-5 days
**Status:** Not started

Track and output top-N moves.

## Long-Term Goals (3-6 Months)

### Major Features

#### 12. NNUE Integration

**Impact:** +200-400 ELO
**Effort:** 2-3 months
**Status:** Not started

Options:
- Use Stockfish NNUE via FFI
- Train custom NNUE (HalfKP architecture)

#### 13. Advanced Pruning

**Impact:** +50-100 ELO cumulative
**Effort:** 1-2 months
**Status:** Not started

Techniques:
- Singular Extensions
- Multi-Cut
- Probcut

#### 14. Syzygy Full Integration

**Impact:** Perfect endgame play
**Effort:** 1 week
**Status:** Partially ready (crate in Cargo.toml)

Features:
- WDL/DTZ probing
- Root move filtering
- Search termination

#### 15. Experience Book Improvements

**Impact:** Better learning
**Effort:** 2-3 weeks
**Status:** Basic implementation exists

Improvements:
- Q-learning tuning
- Forgetting mechanism
- Move confidence

## ELO Projection

| Milestone | Estimated ELO | Notes |
|-----------|---------------|-------|
| Current | ~1500-1800 | Baseline |
| + Draw detection | +100 | ~1600-1900 |
| + Passed pawns | +50 | ~1650-1950 |
| + Tapered eval | +30 | ~1680-1980 |
| + Magic bitboards | +50 (via depth) | ~1730-2030 |
| + NNUE | +300 | ~2000-2300 |

**Conservative Target:** 2000 ELO
**Optimistic Target:** 2400 ELO (with NNUE)

## Priority Matrix

| Priority | Item | Impact | Effort |
|----------|------|--------|--------|
| P0 | Draw detection | High | Low |
| P0 | Endgame recognition | High | Low |
| P1 | Lazy-SMP diversity | High | Medium |
| P1 | Passed pawns | Medium | Medium |
| P2 | Magic bitboards | High | High |
| P2 | Tapered eval | Medium | Medium |
| P3 | NNUE | Very High | Very High |

## Known Issues to Fix

### Bug: King Legality Check (Depth 3+)

**Severity:** Medium
**Impact:** +179 nodes in perft depth 3
**Status:** Under investigation

Some king moves into check are not being filtered correctly.

### Bug: Limited Multi-Thread Scaling

**Severity:** Low
**Impact:** Only 4% speedup on 2 threads
**Status:** Needs diversity layer

## How to Contribute

See [Contributing Guidelines](../development/contributing.md) for how to help with these items.

**High-value contributions:**
- Draw detection implementation
- Endgame recognition
- Passed pawn evaluation
- Bug fixes

---

**Related Documents:**
- [Architecture Overview](../architecture/overview.md)
- [Performance Reference](./performance.md)
- [Contributing Guidelines](../development/contributing.md)
