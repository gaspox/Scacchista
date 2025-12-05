# Optimization Plan - Scacchista Performance Enhancement

**Date**: 2025-12-03
**Current Performance**: ~15k nodes/sec
**Target Performance**: 50k-150k nodes/sec (3-10x improvement)
**Performance Gap**: 273x slower than slow64 reference engine

---

## 🔍 Identified Bottlenecks

### Critical Bottleneck #1: Legal Move Generation (HIGHEST PRIORITY)

**Location**: `src/board.rs:857-871` (`generate_moves()`)

**Problem**:
```rust
for mv in pseudo {
    let undo = self.make_move(mv);  // ← EXPENSIVE!
    // ... check if king is attacked ...
    self.unmake_move(undo);        // ← EXPENSIVE!
}
```

**Impact**:
- `make_move()` does full Zobrist hash update, castling rights, en passant handling
- Called ~40 times per position (average branching factor)
- At 15k nodes/sec, this means ~600k make/unmake calls per second
- Estimated cost: **50-70% of total CPU time**

**Solution**: Implement fast legality check without make/unmake
- Option A: Pin-aware move generation (generate only legal moves)
- Option B: Lightweight `is_legal_move()` function
- **Recommended**: Option B (easier to implement, less risky)

**Expected Impact**: **3-5x speedup** (reduce 600k calls to near-zero)

---

### Critical Bottleneck #2: Heap Allocations per Node

**Location**: `src/board.rs:854,856`

**Problem**:
```rust
let mut pseudo: Vec<Move> = Vec::with_capacity(256);  // Heap allocation!
// ...
let mut legal = Vec::with_capacity(pseudo.len());     // Heap allocation!
```

**Impact**:
- Two heap allocations per node
- At depth 8, ~400k nodes → **800k heap allocations**
- Each allocation involves syscall overhead

**Solution**: Stack-allocated move buffers
```rust
const MAX_MOVES: usize = 256;
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    count: usize,
}
```

**Expected Impact**: **20-40% speedup** (eliminate allocation overhead)

---

### Medium Bottleneck #3: piece_on() Calls

**Location**: `src/board.rs:937,957` (in `generate_pawn_pseudos`)

**Problem**:
```rust
let captured_kind = self.piece_on(to).unwrap().0;  // Called for every capture
```

**Impact**:
- Called for every capture move during generation
- Likely scans multiple bitboards

**Solution**: Inline piece lookup or cache piece board
```rust
#[inline(always)]
fn piece_kind_at(&self, sq: usize) -> Option<PieceKind> {
    // Direct bitboard check, no iteration
}
```

**Expected Impact**: **10-15% speedup**

---

### Medium Bottleneck #4: Missing Inline Annotations

**Location**: Various hot path functions

**Problem**:
- `move_from_sq()`, `move_to_sq()`, `move_piece()` not marked `#[inline(always)]`
- Small functions called millions of times

**Solution**: Add inline annotations
```rust
#[inline(always)]
pub fn move_from_sq(m: Move) -> usize {
    (m & 0x3F) as usize
}
```

**Expected Impact**: **5-10% speedup**

---

### Low Bottleneck #5: Zobrist init_zobrist() Call

**Location**: `src/board.rs:393`

**Problem**:
```rust
crate::zobrist::init_zobrist();  // Called on EVERY make_move!
```

**Impact**: Unnecessary if tables are already initialized

**Solution**: Lazy static initialization
```rust
use once_cell::sync::Lazy;
static ZOBRIST: Lazy<ZobristTables> = Lazy::new(|| ZobristTables::new());
```

**Expected Impact**: **2-5% speedup**

---

## 📊 Optimization Priority Matrix

| # | Optimization | Difficulty | Risk | Impact | Priority Score |
|---|--------------|------------|------|--------|----------------|
| 1 | Fast legality check | Medium | Medium | 3-5x | ⭐⭐⭐⭐⭐ (10/10) |
| 2 | Stack move buffers | Easy | Low | 1.2-1.4x | ⭐⭐⭐⭐ (8/10) |
| 3 | Inline annotations | Easy | Low | 1.05-1.1x | ⭐⭐⭐ (7/10) |
| 4 | Fast piece_on() | Easy | Low | 1.1-1.15x | ⭐⭐⭐ (7/10) |
| 5 | Zobrist lazy init | Easy | Low | 1.02-1.05x | ⭐⭐ (5/10) |

**Priority Score Formula**: `(Impact * 10) / (Difficulty * Risk)`

---

## 🚀 Implementation Roadmap

### Phase 1: Quick Wins (Day 1-2)
**Goal**: 2x speedup with low-risk changes

1. **Add inline annotations** (#3)
   - Files: `src/board.rs`
   - Functions: `move_from_sq`, `move_to_sq`, `move_piece`, `move_flag`, etc.
   - Test: cargo test + perft
   - Benchmark: depth 8

2. **Optimize piece_on()** (#4)
   - Implement fast piece lookup
   - Test: cargo test + perft
   - Benchmark: depth 8

3. **Zobrist lazy init** (#5)
   - Use `once_cell` or `lazy_static`
   - Test: cargo test + perft
   - Benchmark: depth 8

**Expected Result**: 15k → 30k nodes/sec

---

### Phase 2: Stack Allocations (Day 3-4)
**Goal**: 3x speedup cumulative

4. **Implement MoveList struct** (#2)
   - Create stack-allocated move buffer
   - Replace Vec<Move> with MoveList in:
     - `generate_moves()`
     - `generate_pseudo_moves()`
     - All generate_*_pseudos() functions
   - Test: cargo test + perft (CRITICAL: ensure no buffer overflows)
   - Benchmark: depth 8

**Expected Result**: 30k → 45k nodes/sec

---

### Phase 3: Fast Legality (Day 5-7)
**Goal**: 10x speedup cumulative (150k nodes/sec)

5. **Implement is_legal_move() without make/unmake** (#1)

   **Approach**:
   ```rust
   fn is_legal_move_fast(&self, mv: Move) -> bool {
       let from = move_from_sq(mv);
       let to = move_to_sq(mv);
       let piece = move_piece(mv);
       let our_king_sq = self.king_sq(self.side);

       // Case 1: King move - check destination not attacked
       if piece == PieceKind::King {
           return !self.is_square_attacked_by(to, opponent);
       }

       // Case 2: Check if moving piece is pinned
       if self.is_pinned(from, our_king_sq) {
           // If pinned, move must be along pin ray
           return self.is_move_along_ray(from, to, our_king_sq);
       }

       // Case 3: In check - move must block or capture checker
       if self.in_check() {
           return self.move_resolves_check(mv);
       }

       // Case 4: Normal move - legal
       true
   }
   ```

   **Required helpers**:
   - `is_pinned(sq, king_sq) -> bool`
   - `is_move_along_ray(from, to, king) -> bool`
   - `in_check() -> bool`
   - `move_resolves_check(mv) -> bool`

   **Implementation Steps**:
   1. Implement helper functions
   2. Add unit tests for each case (pinned pieces, checks, normal)
   3. Replace make/unmake loop in `generate_moves()`
   4. Run full perft suite (MUST match exact node counts!)
   5. Benchmark depth 8

   **Risk Mitigation**:
   - Implement alongside existing code (feature flag)
   - Extensive unit testing
   - Perft validation before replacing old code

   **Expected Result**: 45k → 150k+ nodes/sec

---

## ✅ Success Criteria

### Mandatory (MUST PASS):
- ✅ All unit tests pass (`cargo test`)
- ✅ Perft tests exact match (`cargo run --bin perft -- --depth 6`)
- ✅ No clippy warnings
- ✅ Code formatted (`cargo fmt`)

### Performance Targets:
- 🎯 **Minimum Goal**: 50k nodes/sec (3x improvement)
- 🎯 **Target Goal**: 100k nodes/sec (6-7x improvement)
- 🎯 **Stretch Goal**: 150k nodes/sec (10x improvement)

### Benchmark Command:
```bash
time printf 'uci\nposition startpos\ngo depth 8\nquit\n' | ./target/release/scacchista
```

**Baseline**: 26.2 seconds (408k nodes, 15.6k nodes/sec)
**Target**: 4-8 seconds (50k-100k nodes/sec)

---

## 🔬 Validation Strategy

After each optimization:

1. **Correctness**:
   ```bash
   cargo test
   cargo run --release --bin perft -- --depth 6
   ```

2. **Performance**:
   ```bash
   cargo build --release
   time printf 'uci\nposition startpos\ngo depth 8\nquit\n' | ./target/release/scacchista
   ```

3. **Quality**:
   ```bash
   cargo test tactical
   ```

4. **Code Quality**:
   ```bash
   cargo clippy --all-targets -- -D warnings
   cargo fmt --check
   ```

**Decision Rule**:
- If speedup <5%: REVERT
- If any test fails: FIX before continuing
- If perft mismatch: CRITICAL BUG, stop all work

---

## 📈 Expected Performance Progression

| Phase | Optimization | Nodes/sec | Speedup | Cumulative | Time@depth8 |
|-------|--------------|-----------|---------|------------|-------------|
| Baseline | - | 15,600 | 1.0x | 1.0x | 26.2s |
| 1.1 | Inline annotations | 17,000 | 1.09x | 1.09x | 24.0s |
| 1.2 | Fast piece_on() | 19,000 | 1.12x | 1.22x | 21.5s |
| 1.3 | Zobrist lazy init | 20,000 | 1.05x | 1.28x | 20.4s |
| 2.1 | Stack move buffers | 30,000 | 1.50x | 1.92x | 13.6s |
| 3.1 | Fast legality check | 150,000 | 5.00x | 9.6x | 2.7s |

**Final Target**: 150k nodes/sec in 2.7 seconds (vs current 26.2s)

---

## ⚠️ Risk Assessment

### High-Risk Items:
1. **Fast legality check (#1)**:
   - Risk: Correctness bugs (missed pins, illegal moves allowed)
   - Mitigation: Extensive testing, perft validation, parallel implementation

### Medium-Risk Items:
2. **Stack move buffers (#2)**:
   - Risk: Buffer overflow if >256 moves
   - Mitigation: Assert + panic in debug, capacity check in release

### Low-Risk Items:
3-5. All others: Low risk, easily revertible

---

## 🔧 Tools & Dependencies

No additional dependencies required! All optimizations use:
- Standard library
- Existing bitboard utilities
- Rust optimization flags (already set in Cargo.toml)

Optional (for future profiling):
- `cargo install flamegraph` (for visual profiling)
- `perf` (Linux profiling tool)

---

## 📝 Notes

- All optimizations preserve correctness (pass perft tests)
- Optimizations are independent (can be applied separately)
- Each phase builds on previous phase
- If Phase 3 proves too risky, Phases 1-2 alone give 2-3x speedup
- Fast legality check is the "big bet" - high reward, medium risk

---

**Next Step**: Implement Phase 1 optimizations (quick wins, low risk)
