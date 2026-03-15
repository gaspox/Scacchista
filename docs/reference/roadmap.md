# Development Roadmap

**Current Version:** v0.5.3  
**Last Updated:** 2026-03-10

---

## Quick Links

- [Changelog](../../CHANGELOG.md)
- [v0.6.0 Roadmap](../../ROADMAP_v0.6.md)
- [Status Report](../../DOC_STATUS.md)

---

## Current Status

**Version:** 0.5.3-stable  
**ELO Estimate:** ~1700-1800  
**Test Coverage:** 30/30 tests passing ✅

### Recent Achievements (v0.5.3)

| Feature | Status | Impact |
|---------|--------|--------|
| SEE Cache Array | ✅ Complete | ~5-10% speed |
| Razoring | ✅ Complete | ~2-3% speed |
| Draw Detection Opt | ✅ Complete | Reduced overhead |
| Test Infrastructure | ✅ Complete | EPD + Regression |

---

## Completed Milestones

### Phase 1: Core Engine ✅
- Board representation (bitboards)
- Move generation (pseudo-legal + legal filter)
- Make/unmake moves
- Zobrist hashing
- Alpha-beta search
- Transposition table (Mutex-based)
- Move ordering (TT, MVV-LVA, killers, history, countermoves)

### Phase 2: Search Enhancements ✅
- Aspiration windows
- Quiescence search
- Null-move pruning
- Late move reductions (LMR)
- Futility pruning
- Razoring (v0.5.3)
- SEE pruning

### Phase 3: Evaluation ✅
- Material evaluation
- Tapered Evaluation (MG/EG interpolation)
- PeSTO-based Piece-Square Tables
- King safety (pawn shield, exposure)
- Pawn structure (doubled, isolated, passed)
- Bishop pair bonus

### Phase 4: Infrastructure ✅
- UCI interface
- Time management
- Multi-threading (Lazy-SMP)
- Test infrastructure (Unit, EPD, Regression)

---

## Short-Term Goals (v0.6.0)

See [ROADMAP_v0.6.md](../../ROADMAP_v0.6.md) for detailed planning.

### Priority 0
- **Lazy-SMP Diversity**: Per-worker tables, different windows
- **Pawn Hash Table**: Cache pawn structure evaluation

### Priority 1
- **Magic Bitboards**: 3-5x move generation speedup
- **Endgame Recognition**: KQK, KRK, etc.

### Priority 2
- **Advanced Pruning**: Singular extensions, probcut

---

## Long-Term Goals (v0.7.0+)

- **NNUE Integration**: +200-400 ELO
- **Syzygy Full Integration**: Perfect endgame
- **Advanced Time Management**: Think on opponent's time

---

## ELO Projection

| Version | ELO Estimate | Key Features |
|---------|--------------|--------------|
| v0.5.3 | 1700-1800 | Stable, all P0 done |
| v0.6.0 | 1900-2000 | Multi-threading optimized |
| v0.7.0 | 2100-2300 | NNUE integration |

---

## Contributing

See [Contributing Guidelines](../development/contributing.md)

High-value areas:
- Magic bitboards implementation
- Endgame recognition
- Performance optimization
- Test expansion

---

*Last updated: 2026-03-10 | v0.5.3 release*
