# Performance Reference

This document contains performance benchmarks and metrics for Scacchista.

## Current Performance (December 2025)

### Search Performance

| Depth | Time | NPS | Notes |
|-------|------|-----|-------|
| 6 | ~568ms | ~46k | Quick test |
| 7 | ~2.2s | ~46k | Standard |
| 8 | ~15s | ~27k | Deep test |

**Test Configuration:**
- CPU: (system dependent)
- Build: `cargo build --release`
- Position: startpos
- Threads: 1

### Perft Performance

| Depth | Nodes | Time | NPS |
|-------|-------|------|-----|
| 5 | 4,865,609 | ~1.1s | ~4.3M |
| 6 | 119,060,324 | ~27s | ~4.4M |

### Function-Level Performance

| Function | Time | Notes |
|----------|------|-------|
| `make_move + unmake_move` | ~76ns | Per move pair |
| `is_square_attacked` | ~31ns | Attack detection |
| `generate_moves` | ~3,235ns | All legal moves |
| `evaluate` | ~3us | Full HCE |
| `evaluate_fast` | ~1us | Material + PSQT |

## Performance History

### Optimization Timeline

| Date | Change | Depth 6 | Speedup |
|------|--------|---------|---------|
| Initial | Baseline | 1,763ms | 1.0x |
| Dec 2025 | TT incremental zobrist | 1,730ms | 1.02x |
| Dec 2025 | evaluate_fast() in qsearch | **568ms** | **3.1x** |

### Cumulative Improvement

| Phase | Improvement | Cumulative |
|-------|-------------|------------|
| Phase 1 (baseline) | 1.0x | 1.0x |
| Phase 2 (TT optimization) | 1.02x | 1.02x |
| Phase 3 (eval optimization) | 3.0x | **3.1x** |

## Time Breakdown

### Per-Node Analysis (Search)

```
Total time per node: ~66,667ns (at 15k NPS baseline)

After optimization: ~21,739ns (at 46k NPS)

Breakdown:
├─ Move generation:     4.8%  (~3,235ns)
├─ Make/unmake:         4.6%  (~3,040ns for ~40 moves)
├─ Legality check:      1.9%  (~1,240ns)
├─ Evaluation:         ~35%   (reduced from ~40%)
├─ TT operations:      ~20%
├─ Move ordering:      ~12%
└─ Search logic:       ~22%
```

### Profiling Insights

1. **Move generation is NOT the bottleneck**
   - Perft achieves 4.3M NPS
   - make/unmake already at 76ns

2. **Evaluation was the bottleneck**
   - Reduced from ~40% to ~35% with evaluate_fast()
   - 3.1x total speedup achieved

3. **TT is significant but acceptable**
   - ~20% of time
   - Future optimization opportunity

## Multi-Threading Performance

### Current Scaling (Lazy-SMP)

| Threads | Depth 7 | Speedup |
|---------|---------|---------|
| 1 | 2.386s | 1.0x |
| 2 | 2.283s | 1.04x |

**Note:** Limited scaling due to lack of search diversity. All workers search identical paths.

### Expected After Diversity Implementation

| Threads | Expected Speedup |
|---------|------------------|
| 2 | 1.5-1.8x |
| 4 | 2.0-2.8x |
| 8 | 3.0-4.5x |

## Comparison with Other Engines

### Approximate NPS Ranges

| Engine | NPS | Notes |
|--------|-----|-------|
| Stockfish | 10M+ | NNUE, highly optimized |
| Lc0 (CPU) | ~100k | Neural network |
| Rustic | 1-2M | Similar architecture |
| **Scacchista** | **~46k** | HCE, Rust |

### Performance Gap Analysis

Scacchista vs Stockfish:
- ~200x slower in raw NPS
- Acceptable for hobby engine
- Focus on correctness over speed

## Benchmark Positions

### Standard Benchmark Suite

| Position | FEN | Use |
|----------|-----|-----|
| Startpos | `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1` | Baseline |
| Kiwipete | `r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1` | Complex |
| Position 3 | `8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1` | En passant |
| Position 4 | `r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1` | Castling |

### Running Benchmarks

```bash
# Quick benchmark (depth 6)
time printf 'uci\nposition startpos\ngo depth 6\nquit\n' | ./target/release/scacchista

# Standard benchmark (depth 8)
time printf 'uci\nposition startpos\ngo depth 8\nquit\n' | ./target/release/scacchista

# Perft benchmark
time cargo run --release --bin perft -- --depth 5
```

## Performance Targets

### Achieved

- [x] Depth 6 in < 1 second
- [x] Depth 8 in < 20 seconds
- [x] 3x speedup from baseline
- [x] 4M+ NPS in perft

### Future Goals

- [ ] Multi-thread scaling (1.5x on 2 threads)
- [ ] Depth 8 in < 10 seconds
- [ ] 100k NPS in search

## Hardware Recommendations

### Minimum

- Any modern CPU (x86_64)
- 256 MB RAM
- Rust toolchain

### Recommended

- Multi-core CPU (4+ cores)
- 1 GB+ RAM for larger hash
- SSD for tablebase access

### For Development

- Fast CPU for quick test cycles
- Profiling tools (perf, flamegraph)

## Known Performance Issues

### 1. Limited Multi-Thread Scaling

**Issue:** Lazy-SMP shows minimal speedup.
**Cause:** No search diversity.
**Impact:** Cannot utilize multi-core effectively.
**Status:** Infrastructure ready, needs diversity layer.

### 2. Evaluation Overhead

**Issue:** Full eval expensive in qsearch.
**Mitigation:** evaluate_fast() for quiescence.
**Remaining:** ~35% of time still in eval.

### 3. TT Contention (Multi-thread)

**Issue:** Mutex on TT causes contention.
**Impact:** Reduces multi-thread efficiency.
**Future:** Consider lock-free TT.

---

**Related Documents:**
- [Benchmarking Guide](../development/benchmarking.md)
- [Architecture Overview](../architecture/overview.md)
- [Threading](../architecture/threading.md)
