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

## [v0.6.0-alpha.2] - 2026-04-30

### Search Professionalization (Fase 2)
- **PV Tracking**: Triangular PV table popolata durante la ricerca
- **UCI Info Output Professionale**: `info depth seldepth score cp|mate nodes nps time hashfull pv`
- **Lazy-SMP Diversity**: worker helper con depth offset (`N % 3`) e aspiration window allargata (`+N*10`)
- **Main thread authority**: risultato preso dal primo worker che completa
- **MoveOverhead**: nuova opzione UCI (default 10ms) per compensare lag GUI/rete
- **Emergency Time Management**: allocazione conservativa se `< 5s`, emergency se `< 1s`
- **Ponderhit**: timer thread che ferma la ricerca dopo il tempo allocato reale

### Evaluation Overhaul (Fase 3)
- **Tapered Evaluation**: PSQT MG/EG separate con interpolazione PeSTO (`phase = 0..24`)
- **Pawn Structure**: doubled (−20), isolated (−15) penalties
- **Passed Pawns**: bonus progressivi per rank (20→40→80→150 cp)
- **Bishop Pair**: +30 cp bonus
- **Mobility**: cavallo +4, alfiere +3, torre +2, donna +1 per casella libera
- **Endgame Recognition Estesa**: KBB, KQvKR, KQvKP, KRNvKR, KRBvKR, KRPvKR (avanzato), KBPvK

### Pulizia
- Rimosso `evaluate_lazy()` e `quick_material_count()` (codice morto)
- Rimossi tutti `static mut` (Zobrist su `OnceLock`)
- Zero `unsafe` residui in board.rs
- Clippy pulito (`-D warnings` passa)

### Performance
- Perft: ~29 Mnps
- Search startpos d8: ~361 kNPS
- Tactical d8: ~1.55 MNPS

---

## [Unreleased] - v0.6.0 Final

### Planned
- Pawn hash table
- Syzygy tablebase integration
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
