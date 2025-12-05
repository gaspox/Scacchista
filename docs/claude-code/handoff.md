# Technical Handoff Document

**Date:** December 2025
**Version:** 0.2.1-beta
**Branch:** master
**Status:** Active Development

---

## Project Overview

**Scacchista** is a UCI chess engine written in Rust, designed with focus on performance and correctness.

### Key Features

- **UCI Protocol**: Complete with Hash, Threads, Style, Experience Book
- **Search Engine**: Alpha-beta with PVS, aspiration windows, quiescence
- **Pruning**: Null-move, LMR, futility
- **Move Ordering**: TT moves, MVV-LVA+SEE, killers, history
- **Evaluation**: Material + PSQT + king safety + development + center control
- **Parallel Search**: Lazy-SMP broadcast architecture
- **Time Management**: Intelligent allocation with fallback strategies
- **Opening Book**: Polyglot format support
- **Endgame**: Syzygy tablebase probing

### Tech Stack

- **Language**: Rust (Edition 2021)
- **Core Crates**: shakmaty, shakmaty-uci, shakmaty-syzygy
- **Serialization**: serde, bincode

---

## Current Session Summary

### Recent Work Completed

1. **Performance Optimization** (3.1x speedup)
   - evaluate_fast() for quiescence search
   - Cached is_in_check() calls
   - Depth 6: 1763ms -> 568ms

2. **Lazy-SMP Infrastructure**
   - Fixed TT sharing bug (workers were creating local TT)
   - Implemented broadcast model
   - Note: Needs diversity layer for full scaling

3. **Documentation Reorganization**
   - Created structured docs/ directory
   - Migrated all documentation
   - Added cross-references

### Performance Metrics

| Depth | Before | After | Speedup |
|-------|--------|-------|---------|
| 6 | 1,763ms | 568ms | **3.1x** |
| 8 | ~26s | 15.2s | **1.7x** |

### Test Status

- Unit tests: 57/57 passing
- Integration tests: 23/23 passing
- Total: **80/80 tests passing**
- Perft validation: Exact match

---

## Architecture Overview

```
src/
├── main.rs              # Entry point
├── lib.rs               # Module exports
├── board.rs             # Bitboard representation
├── eval.rs              # HCE evaluation
├── zobrist.rs           # Hashing
├── utils.rs             # Attack tables
├── uci/                 # UCI protocol
│   ├── loop.rs         # Main loop
│   ├── parser.rs       # Command parsing
│   └── options.rs      # UCI options
├── search/              # Search engine
│   ├── search.rs       # Alpha-beta core
│   ├── tt.rs           # Transposition table
│   ├── thread_mgr.rs   # Lazy-SMP
│   ├── stats.rs        # Statistics
│   └── params.rs       # Parameters
└── time/
    └── mod.rs           # Time management
```

### Key Files Modified Recently

1. **src/search/search.rs**
   - TT now `Arc<Mutex<TranspositionTable>>`
   - Added `with_shared_tt()` method
   - 9 TT accesses updated with lock pattern

2. **src/search/thread_mgr.rs**
   - Broadcast lazy-SMP architecture
   - job_available, workers_done, results vector

3. **src/eval.rs**
   - Bitboard iteration optimization
   - O(768) -> O(32) complexity

---

## Known Issues

### 1. Lazy-SMP Limited Scaling

**Problem**: Workers execute identical searches, no speedup.
**Status**: Infrastructure ready, needs diversity layer.
**Fix Options**:
- Per-worker history tables
- Different aspiration windows
- Random eval perturbations

### 2. SEE Cache Overhead

**Problem**: HashMap allocation/lookup for SEE caching.
**Fix**: Use `[Option<i16>; 64]` array.
**Effort**: 30 minutes
**Impact**: ~5-10% speedup

### 3. Magic Bitboards Missing

**Problem**: Sliding piece generation uses loops O(7n).
**Fix**: Implement magic bitboards.
**Effort**: 1-2 weeks
**Impact**: 3-5x speedup in move gen

---

## Development Roadmap

### Immediate (1-3 days)

1. SEE Cache Array (30 min, ~5-10% speedup)
2. Razoring (1 hour, ~2-3% speedup)
3. Lazy-SMP Diversity (1-2 days, +50-80% on 2 threads)

### Short-Term (1-2 weeks)

4. Magic Bitboards (1-2 weeks, 3-5x move gen)
5. Passed Pawn Evaluation (2-3 days, +30-50 ELO)
6. Tapered Evaluation (2-3 days, +20-30 ELO)
7. Bishop Pair Bonus (1 hour, +10-15 ELO)

### Medium-Term (1-2 months)

8. Pawn Hash Table (1 week, ~10-15% speedup)
9. King Safety Improvements (1 week, +20-30 ELO)
10. Multi-PV Support (3-5 days)

### Long-Term (3-6 months)

11. NNUE Integration (2-3 months, +200-400 ELO)
12. Advanced Pruning (1-2 months, +50-100 ELO)
13. Syzygy Full Integration (1 week)

---

## Quick Start for New Session

### Build & Test

```bash
git pull origin master
cargo build --release
cargo test
```

### Benchmark

```bash
time printf 'uci\nposition startpos\ngo depth 7\nquit\n' | ./target/release/scacchista
# Expected: ~2.2s
```

### Pick Next Task

**Quick wins**:
1. SEE cache array (30 min, easy)
2. Razoring (1 hour, easy)

**Medium effort**:
3. Lazy-SMP diversity (1-2 days)
4. Draw detection (2-3 hours)

### Workflow

1. Create feature branch
2. Implement + test
3. Benchmark before/after
4. Document in commit message
5. PR to master

---

## Lessons Learned

### 1. Data-Driven Optimization

- Profile before optimizing
- Lazy eval showed -10% regression -> REVERTED
- Bitboard iteration: 8-9% actual vs 30-40% estimated

### 2. Architecture Over Hacks

- Broadcast lazy-SMP: correct infrastructure, deferred diversity
- TT sharing fix: zero regression due to clean design

### 3. Test Coverage Essential

- 80 tests caught regressions multiple times
- Perft validates move gen correctness
- Tactical tests ensure no search bugs

### 4. Compiler Surprises

- `piece_on()` already optimized by compiler
- Branch prediction mitigates some anti-patterns
- Debug assertions have zero overhead in release

---

## Contact & Ownership

**Project**: Scacchista UCI Chess Engine
**Owner**: Gaspare (@gaspox)
**Repository**: github.com:gaspox/Scacchista.git
**Status**: Active Development

---

## Pre-Handoff Checklist

- [x] All tests passing (80/80)
- [x] Build release working
- [x] Performance benchmarked
- [x] Commit messages descriptive
- [x] Known issues documented
- [x] Roadmap prioritized
- [x] Documentation complete
- [x] Git history clean

---

**Related Documents:**
- [Project Guide](./project-guide.md)
- [Agents Configuration](./agents.md)
- [Architecture Overview](../architecture/overview.md)
- [Roadmap](../reference/roadmap.md)
