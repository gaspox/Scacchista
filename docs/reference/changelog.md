# Changelog

All notable changes to Scacchista are documented in this file.

## [0.5.0-dev] - February 2026

### Major Features

#### Principal Variation Search (PVS) at Root
- Implemented PVS logic specifically for the iteration deepening root search.
- Use of null-window search for non-PV moves to increase pruning efficiency.

### Bug Fixes

#### Futility Pruning MATE Score Fix
- **Critical**: Restored correct evaluation when Futility Pruning skips all moves in a node.
- Previously, it erroneously returned `-MATE`, leading to evaluation corruption at the root.
- Added `legal_moves` tracking to safely return `alpha` (fail-low) if all moves are pruned.

## [0.4.1] - January 2026

### Performance & Optimization

#### Dedicated Capture Generation in QSearch
- Implemented specialized capture-only generator for quiescence search.
- Reduces moves to search from ~40-50 down to ~5-10 per node.
- NPS boost: +40-60%.

#### Delta Pruning
- Implemented delta pruning in qsearch to skip futile captures.
- NPS boost: +15-20%.

#### Lock-Free Transposition Table
- Replaced lock-based TT with a multi-thread friendly, concurrent access pattern.
- Significant scaling improvement for multi-core search.

#### Bitboard-based Static Evaluation
- Replaced iterative square-by-square piece evaluation with high-performance bitboard operations.
- NPS boost for `evaluate_fast()`: 2-3x speedup per call.

### Bug Fixes
- Fixed pawn promotion move generation bugs.
- Fixed UCI time management (movetime vs increment priority).
- Fixed thread-safety issues in Lazy-SMP.

## [0.2.1-beta] - December 2025

### Performance Improvements

#### 3.1x Search Speedup
- **evaluate_fast() in quiescence search**: Simplified evaluation (material + PSQT only) for qsearch nodes
  - Depth 6: 1763ms -> 568ms
  - Depth 8: ~26s -> 15.2s
- **Cache is_in_check()**: Avoid duplicate expensive calls in negamax
- **TT incremental zobrist**: Optimized hash updates

#### Lazy-SMP Broadcast Architecture
- Implemented true TT sharing between worker threads
- Changed `Search::tt` from owned to `Arc<Mutex<TranspositionTable>>`
- Added `with_shared_tt()` builder method
- Note: Scaling limited (needs diversity layer for full benefit)

### Bug Fixes

#### TT Sharing Fix
- **Critical**: Workers were creating local 16MB TT instead of sharing global
- `_tt_clone` was cloned but never used
- Fix: Pass shared TT to workers via `with_shared_tt()`

#### Eval Loop Optimization
- Changed from O(768) to O(~32) complexity
- Iterate bitboards directly instead of checking all 64 squares

### Documentation

- Added comprehensive HANDOFF.md
- Created docs/ directory structure
- Migrated all documentation to organized format

## [0.2.0] - October-November 2025

### Major Features

#### Piece-Square Tables (PSQT)
- Added 6 PSQT tables (one per piece type)
- Proper symmetry for black pieces
- Improved opening play significantly
- Impact: +80-120 ELO

#### King Safety Evaluation
- Exposed king penalty (-50cp)
- Pawn shield bonus (+15cp per pawn, max +45cp)
- Castled position detection
- Impact: +60-100 ELO

#### Development Penalty
- -10cp per undeveloped minor piece after move 10
- Encourages piece development
- Impact: +40-60 ELO

#### Center Control
- +10cp per central square controlled (d4, e4, d5, e5)
- +3cp per extended center square
- Impact: +30-50 ELO

### Bug Fixes

#### Mate Detection Fix
- Fixed qsearch not detecting checkmate
- Added check for empty move list in quiescence
- Search all evasions when in check (not just captures)
- Impact: Engine now finds mates correctly

#### Castling Rights Bug
- Fixed completely inverted bit mapping
- Was removing opponent's castling rights instead of own
- Impact: +14 nodes in perft depth 2

#### En Passant Generation
- En passant captures were never generated
- EP square is empty, not in enemy_occ
- Added separate EP handling
- Impact: Correct move generation

#### Promotion Unmake Bug
- Was removing pawn instead of promoted piece
- Added `promoted_piece` field to Undo struct
- Impact: Perft correctness

#### Castle Rook Movement
- Rook was not being moved during castling
- Only king was moved
- Impact: Castling now works correctly

### Search Improvements

#### Check Extensions
- +1 ply extension when in check
- Limited to ply < 10 to prevent explosion
- Impact: Finds deeper mates

### Test Suite

- Added tactical.rs (7 tests)
- Added test_development_penalty.rs (5 tests)
- Added test_king_safety.rs (5 tests)
- Total: 80+ tests passing

## [0.1.1] - October 2025

### Bug Fixes

#### Tactical Blindness Fix
- **Critical**: Missing negation in qsearch call from iddfs
- Engine showed +1120 after losing knight (should be negative)
- Engine showed +20800 after losing queen (should be ~-600)
- Impact: Engine now evaluates material correctly

### Test Suite

- Added tactical test suite
- Added material evaluation tests
- 50+ tests passing

## [0.1.0] - Initial Release

### Core Features

- Bitboard-based board representation
- Legal move generation via pseudo-legal + filter
- Incremental Zobrist hashing
- Alpha-beta search with iterative deepening
- Transposition table with replacement scheme
- Move ordering (TT move, MVV-LVA, killers, history)
- Quiescence search
- Null-move pruning
- Late move reductions (LMR)
- Futility pruning
- Time management
- UCI protocol support
- Multi-threading (Lazy-SMP)

### Dependencies

- shakmaty for rules validation
- shakmaty-uci for UCI parsing
- shakmaty-syzygy for tablebase support
- serde + bincode for serialization

---

## Version Numbering

- **0.x.y**: Development versions
  - x: Major feature additions
  - y: Bug fixes and minor improvements
- **1.0.0**: First stable release (planned)

## Contributing

See [Contributing Guidelines](../development/contributing.md) for how to report issues and submit changes.
