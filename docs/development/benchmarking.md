# Benchmarking Guide

This document explains how to measure and analyze Scacchista's performance.

## Quick Start

```bash
# Build release (required for accurate benchmarks)
cargo build --release

# Benchmark search at depth 8
time printf 'uci\nposition startpos\ngo depth 8\nquit\n' | ./target/release/scacchista
```

## Benchmark Types

### 1. Search Performance

Measures nodes per second (NPS) during search.

```bash
# Depth 6 benchmark
time printf 'uci\nposition startpos\ngo depth 6\nquit\n' | ./target/release/scacchista

# Depth 8 benchmark
time printf 'uci\nposition startpos\ngo depth 8\nquit\n' | ./target/release/scacchista
```

**Expected Results (single thread):**

| Depth | Time | Notes |
|-------|------|-------|
| 6 | ~568ms | Quick test |
| 7 | ~2.2s | Standard |
| 8 | ~15s | Deep test |

### 2. Perft Performance

Measures move generation speed.

```bash
# Perft depth 5 (4.8M nodes)
time cargo run --release --bin perft -- --depth 5

# Perft depth 6 (119M nodes)
time cargo run --release --bin perft -- --depth 6
```

**Expected Results:**

| Depth | Nodes | Time | NPS |
|-------|-------|------|-----|
| 5 | 4,865,609 | ~1.1s | ~4.3M |
| 6 | 119,060,324 | ~27s | ~4.4M |

### 3. Function Microbenchmarks

Measure individual function performance:

```bash
# Using criterion (if available)
cargo bench

# Manual timing in tests
cargo test bench_function -- --nocapture
```

## Benchmark Positions

### Starting Position

```bash
position startpos
```

Standard benchmark baseline.

### Kiwipete (Complex)

```bash
position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1
```

Complex position with castling, pins, checks.

### Italian Game

```bash
position startpos moves e2e4 e7e5 g1f3 b8c6 f1c4 f8c5
```

Common opening position.

### Middlegame Position

```bash
position fen r1bq1rk1/ppp2ppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQ1RK1 w - - 0 1
```

Tactical middlegame position.

## Profiling

### Using flamegraph

```bash
# Install
cargo install flamegraph

# Profile perft
sudo cargo flamegraph --bin perft -- --depth 5

# Open flamegraph.svg in browser
firefox flamegraph.svg
```

### Using perf (Linux)

```bash
# Build with debug symbols
cargo build --release

# Record
perf record --call-graph=dwarf ./target/release/perft --depth 5

# Analyze
perf report
```

### Manual Instrumentation

```rust
use std::time::Instant;

let start = Instant::now();
// ... code to measure ...
println!("Time: {:?}", start.elapsed());
```

## Profiling Results Reference

From profiling session (December 2025):

### Time Distribution per Node (Search)

```
Total per node: ~66,667 ns (at 15k NPS)

Move generation:           4.8%
Make/unmake (x40 moves):   4.6%
Legality check (x40):      1.9%
────────────────────────────────
Subtotal move gen:        11.3%

Evaluation:               ~40%
TT operations:            ~20%
Move ordering:            ~12%
Search logic:             ~17%
────────────────────────────────
Subtotal search:          88.7%
```

### Key Findings

1. **Move generation is fast** (4.3M NPS in perft)
2. **Search overhead dominates** (88.7% of time)
3. **Evaluation is main bottleneck** (~40%)
4. **make/unmake already optimized** (76ns pair)

## Multi-threaded Benchmarks

### Thread Scaling

```bash
# 1 thread
printf 'uci\nsetoption name Threads value 1\nposition startpos\ngo depth 7\nquit\n' | ./target/release/scacchista

# 2 threads
printf 'uci\nsetoption name Threads value 2\nposition startpos\ngo depth 7\nquit\n' | ./target/release/scacchista

# 4 threads
printf 'uci\nsetoption name Threads value 4\nposition startpos\ngo depth 7\nquit\n' | ./target/release/scacchista
```

**Current Scaling (needs diversity improvement):**

| Threads | Depth 7 | Speedup |
|---------|---------|---------|
| 1 | 2.39s | 1.0x |
| 2 | 2.28s | 1.04x |

## Performance History

### Optimization Timeline

| Date | Change | Depth 6 | Speedup |
|------|--------|---------|---------|
| Baseline | Initial | 1,763ms | 1.0x |
| Dec 2025 | evaluate_fast() | 568ms | **3.1x** |

### Cumulative Improvements

| Optimization | Individual | Cumulative |
|--------------|------------|------------|
| Cache is_in_check | 1.03x | 1.03x |
| evaluate_fast in qsearch | 3.0x | **3.1x** |

## Benchmark Best Practices

### Do

- Always use release builds (`--release`)
- Run multiple iterations and average
- Warm up the cache before measuring
- Use consistent hardware/conditions
- Document system specs

### Don't

- Benchmark debug builds
- Run alongside other heavy processes
- Compare results across different machines
- Ignore outliers without investigation

### System Specs to Document

```
CPU: [model, cores, frequency]
RAM: [amount, speed]
OS: [name, version]
Rust: [version]
Date: [benchmark date]
```

## Interpreting Results

### Good Indicators

- Consistent results across runs (<5% variance)
- Perft matches reference values exactly
- NPS scales with depth (roughly constant)
- Multi-thread shows some improvement

### Warning Signs

- High variance between runs
- Perft node count mismatch
- NPS drops significantly at higher depth
- Multi-thread slower than single

## Comparing with Other Engines

### Reference Engines

| Engine | Approximate NPS |
|--------|-----------------|
| Stockfish | ~10M+ |
| Lc0 (CPU) | ~100k |
| Rustic | ~1-2M |
| Scacchista | ~46k |

Note: Comparison depends heavily on hardware and search features.

### Fair Comparison

- Same position
- Same depth
- Same time control
- Document hash size, threads

## Automated Benchmarking

### Benchmark Script

```bash
#!/bin/bash
# benchmark.sh

echo "Building release..."
cargo build --release

echo "Perft depth 5..."
time cargo run --release --bin perft -- --depth 5

echo "Search depth 6..."
time printf 'uci\nposition startpos\ngo depth 6\nquit\n' | ./target/release/scacchista

echo "Search depth 8..."
time printf 'uci\nposition startpos\ngo depth 8\nquit\n' | ./target/release/scacchista
```

---

**Related Documents:**
- [Development Setup](./setup.md)
- [Performance Reference](../reference/performance.md)
- [Architecture Overview](../architecture/overview.md)
