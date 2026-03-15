# Changelog

All notable changes to Scacchista chess engine.

## [v0.5.3] - 2026-03-10

### Added
- **Razoring**: Conservative pruning at depth 1 (margin 50cp)
- **Test EPD Suite**: Positional validation tests
- **Optimized Draw Detection**: Skip threefold check in non-PV nodes

### Fixed
- Tactical tests: Increased depth from 3 to 5 for accurate material evaluation
- SEE Cache: Separate cache indices for White/Black attackers

### Test Results
- All tests passing: 30/30 ✅
- Self-consistency: Verified
- Regression tests: Passing

---

## [v0.5.2] - 2026-03-10

### Added
- **SEE Cache Array**: [i16; 128] instead of HashMap for O(1) access
- **Regression Tests**: Automated test suite for search consistency

### Fixed
- SEE cache bug: Cache now considers attacker color (critical fix)

---

## [v0.5.1] - 2026-03-09

### Fixed (Critical)
- **TT Race Condition**: Lock-free TT replaced with Mutex-based
  - Fixes -147 ELO regression vs v0.4
  - Tournament result: +50 ELO (7-3 win vs v0.4)

### Changed
- Parameter tuning: futility_margin 150, lmr_base_reduction 1

### Added
- **Tapered Evaluation**: MG/EG interpolation with PeSTO tables
- **Pawn Structure**: Doubled/isolated pawn penalties
- **Passed Pawns**: Progressive bonuses by rank
- **Bishop Pair**: +50cp bonus when both bishops present
- **Countermove Heuristic**: +19% NPS improvement

---

## [v0.4.1] - 2025-01

### Search
- Alpha-beta with PVS
- Aspiration windows
- Null-move pruning
- LMR (Late Move Reductions)
- Futility pruning
- Quiescence search

### Evaluation
- Material + PSQT
- King safety
- Center control

### Infrastructure
- UCI protocol
- Lazy-SMP threading
- Transposition table
- Zobrist hashing

---

## [Unreleased] - v0.6.0 Roadmap

### Planned
- Lazy-SMP diversity layer
- Pawn hash table
- Magic bitboards
- Endgame recognition
- Advanced pruning (singular extensions, probcut)

### Target
- ELO: 2000+
- Multi-threading: 50%+ scaling on 4 cores

---

## Legend

- **P0**: Critical fix or quick win
- **P1**: Medium effort, high impact
- **P2**: Long term investment

*Version format: MAJOR.MINOR.PATCH*
- MAJOR: Breaking changes
- MINOR: New features
- PATCH: Bug fixes
