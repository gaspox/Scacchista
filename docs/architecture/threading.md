# Threading and Parallel Search

This document describes the Lazy-SMP parallel search implementation in Scacchista.

## Overview

Scacchista implements Lazy-SMP (Symmetric Multi-Processing) for parallel search, where multiple threads search the same position independently while sharing a transposition table.

**Location:** `src/search/thread_mgr.rs`

## Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                      ThreadManager                              │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Shared Transposition Table                   │  │
│  │                Arc<Mutex<TranspositionTable>>            │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│         ┌────────────────────┼────────────────────┐            │
│         │                    │                    │            │
│         ▼                    ▼                    ▼            │
│    ┌─────────┐          ┌─────────┐          ┌─────────┐      │
│    │ Worker  │          │ Worker  │          │ Worker  │      │
│    │   #0    │          │   #1    │          │   #N    │      │
│    └─────────┘          └─────────┘          └─────────┘      │
│         │                    │                    │            │
│         └────────────────────┼────────────────────┘            │
│                              │                                  │
│                              ▼                                  │
│                   Best Result Selection                         │
└────────────────────────────────────────────────────────────────┘
```

## Key Components

### ThreadManager

Coordinates worker threads and aggregates results.

```rust
pub struct ThreadManager {
    num_threads: usize,
    workers: Vec<JoinHandle<()>>,
    shared_tt: Arc<Mutex<TranspositionTable>>,
    job_available: Arc<AtomicBool>,
    workers_done: Arc<AtomicUsize>,
    stop_flag: Arc<AtomicBool>,
    results: Arc<Mutex<Vec<SearchResult>>>,
}
```

### Search with Shared TT

Workers use `with_shared_tt()` builder to share the transposition table:

```rust
impl Search {
    pub fn with_shared_tt(mut self, tt: Arc<Mutex<TranspositionTable>>) -> Self {
        self.tt = tt;
        self
    }
}
```

### TT Access Pattern

All TT accesses use lock pattern:

```rust
// Probe
if let Some(entry) = self.tt.lock().unwrap().probe(zobrist) {
    // Use entry...
}

// Store
self.tt.lock().unwrap().store(zobrist, score, depth, flag, best_move);
```

## Broadcast Model

All N workers search the same position simultaneously:

```rust
fn search_parallel(&mut self, board: Board, depth: u8) -> SearchResult {
    // Signal all workers
    self.job_available.store(true, Ordering::SeqCst);

    // Wait for all workers to complete
    while self.workers_done.load(Ordering::SeqCst) < self.num_threads {
        if self.should_stop() { break; }
        thread::sleep(Duration::from_millis(1));
    }

    // Collect best result
    let results = self.results.lock().unwrap();
    results.iter()
        .max_by_key(|r| r.score)
        .cloned()
        .unwrap()
}
```

## Synchronization

### Job Signaling

```rust
// Main thread: signal new job
self.job_available.store(true, Ordering::SeqCst);

// Worker: wait for job
while !self.job_available.load(Ordering::SeqCst) {
    if self.should_stop() { return; }
    thread::sleep(Duration::from_micros(100));
}
```

### Completion Tracking

```rust
// Worker: signal completion
self.workers_done.fetch_add(1, Ordering::SeqCst);

// Main: wait for all
while self.workers_done.load(Ordering::SeqCst) < self.num_threads {
    thread::sleep(Duration::from_millis(1));
}
```

### Stop Flag

Cooperative stopping across all threads:

```rust
// Main thread: signal stop
self.stop_flag.store(true, Ordering::SeqCst);

// Workers: check periodically
fn should_stop(&self) -> bool {
    self.stop_flag.load(Ordering::SeqCst)
}
```

## Performance Characteristics

### Current Status

| Threads | Depth 7 | Speedup | Notes |
|---------|---------|---------|-------|
| 1 | 2.386s | 1.0x | Baseline |
| 2 | 2.283s | 1.04x | Minimal improvement |

### Root Cause: Missing Diversity

Current implementation has all workers executing identical searches. Benefits from shared TT are minimal because:

1. All workers explore same PV
2. First worker to finish "wins", others wasted
3. No diversity in search paths

### Required: Artificial Diversity

To achieve real speedup, workers need to explore different parts of the tree:

**Option 1: Per-Worker History Tables**
```rust
struct Worker {
    id: usize,
    history: HistoryTable,  // Private, not shared
    shared_tt: Arc<Mutex<TT>>,
}
```

**Option 2: Different Aspiration Windows**
```rust
fn worker_aspiration_delta(worker_id: usize) -> i16 {
    let base = 50;
    let variation = (worker_id as i16 * 10) % 30;
    base + variation  // Workers use different window widths
}
```

**Option 3: Random Evaluation Noise**
```rust
fn evaluate_with_noise(&self, worker_id: usize) -> i16 {
    let base = self.evaluate();
    let noise = (worker_id as i16 * 3) % 10 - 5;  // [-5, +5]
    base + noise
}
```

**Option 4: Depth Offsets**
```rust
fn worker_start_depth(worker_id: usize, target_depth: u8) -> u8 {
    if worker_id == 0 { 1 }
    else { 1 + (worker_id as u8 % 3) }
}
```

### Expected Gains with Diversity

| Threads | Current | With Diversity |
|---------|---------|----------------|
| 2 | 1.04x | 1.5-1.8x |
| 4 | ~1.1x | 2.0-2.8x |
| 8 | ~1.2x | 3.0-4.5x |

## Implementation Notes

### Thread-Safe TT

Transposition table uses `Arc<Mutex<...>>`:

```rust
// Pros: Simple, correct
// Cons: Lock contention under heavy load

// Future optimization: Lock-free TT
// - Use atomic operations for entry updates
// - Accept occasional data races (benign in chess)
```

### Worker Lifecycle

```rust
fn worker_loop(&mut self) {
    loop {
        // Wait for job
        while !self.job_available.load(Ordering::SeqCst) {
            if self.should_stop() { return; }
            thread::park_timeout(Duration::from_micros(100));
        }

        // Execute search
        let result = self.search(...);

        // Store result
        self.results.lock().unwrap().push(result);

        // Signal completion
        self.workers_done.fetch_add(1, Ordering::SeqCst);
    }
}
```

### Best Result Selection

```rust
fn select_best_result(results: &[SearchResult]) -> SearchResult {
    results.iter()
        .filter(|r| r.is_valid())
        .max_by(|a, b| {
            // Prefer deeper searches
            if a.depth != b.depth {
                a.depth.cmp(&b.depth)
            } else {
                // Then higher scores
                a.score.cmp(&b.score)
            }
        })
        .cloned()
        .unwrap()
}
```

## UCI Integration

### Thread Option

```
option name Threads type spin default 1 min 1 max 256
```

### Setting Threads

```rust
fn set_threads(&mut self, n: usize) {
    self.thread_manager.set_thread_count(n);
}
```

## Future Work

1. **Implement Artificial Diversity** (Priority: High)
   - Per-worker history tables
   - Different aspiration windows
   - Expected: +50-80% speedup on 2 threads

2. **Lock-Free TT** (Priority: Medium)
   - Atomic updates instead of mutex
   - Reduce contention

3. **NUMA Awareness** (Priority: Low)
   - Thread affinity
   - Memory locality

4. **Young Brothers Wait Concept (YBWC)** (Priority: Low)
   - More sophisticated work distribution
   - Search first child serially, parallelize rest

---

**Related Documents:**
- [Architecture Overview](./overview.md)
- [Search Engine](./search-engine.md)
- [Performance Reference](../reference/performance.md)
