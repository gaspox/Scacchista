# Testing Guide

This document describes the testing strategy and how to run tests for Scacchista.

## Test Suite Overview

| Category | Count | Location | Description |
|----------|-------|----------|-------------|
| Unit Tests | ~57 | `src/*.rs` | Module-level tests |
| Integration Tests | ~23 | `tests/*.rs` | Cross-module tests |
| Perft Tests | 2 | `tests/` | Move generation validation |
| Tactical Tests | 7 | `tests/tactical.rs` | Search correctness |
| **Total** | **~80+** | - | - |

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Categories

```bash
# Unit tests only (in src/)
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific test file
cargo test --test tactical
cargo test --test test_king_safety
cargo test --test test_development_penalty

# Specific test function
cargo test test_material_after_knight_loss

# With output
cargo test -- --nocapture

# Release mode (faster execution)
cargo test --release
```

## Test Categories

### 1. Perft Tests (Move Generation)

Perft (performance test) validates move generation correctness by counting nodes.

```bash
# Binary perft
cargo run --release --bin perft -- --depth 5

# Test perft
cargo test perft
```

**Reference Values (Starting Position):**

| Depth | Nodes |
|-------|-------|
| 1 | 20 |
| 2 | 400 |
| 3 | 8,902 |
| 4 | 197,281 |
| 5 | 4,865,609 |
| 6 | 119,060,324 |

**Kiwipete Position:**
```
r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1
```

| Depth | Nodes |
|-------|-------|
| 1 | 48 |
| 2 | 2,039 |
| 3 | 97,862 |
| 4 | 4,085,603 |

### 2. Tactical Tests

Test that the engine finds correct moves in tactical positions.

**Location:** `tests/tactical.rs`

```rust
#[test]
fn test_captures_hanging_piece() {
    // Engine should capture undefended piece
}

#[test]
fn test_material_after_queen_loss() {
    // Score should be negative after losing queen
}
```

### 3. Evaluation Tests

**Development Penalty:**
```bash
cargo test --test test_development_penalty
```

- Tests penalty applied after move 10
- Tests no penalty before move 10
- Tests cumulative penalties

**King Safety:**
```bash
cargo test --test test_king_safety
```

- Tests exposed king penalty
- Tests pawn shield bonus
- Tests castled position detection

### 4. UCI Tests

```bash
cargo test --test test_uci_integration
```

- Tests UCI command parsing
- Tests position setup
- Tests go command handling

### 5. Thread Manager Tests

```bash
cargo test --test test_thread_mgr
```

- Tests multi-threaded search
- Tests result aggregation
- Tests stop flag propagation

## Test Patterns

### Board Setup

```rust
use scacchista::*;

#[test]
fn test_example() {
    init();  // Initialize static tables

    let mut board = Board::new();
    board.set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    // Test assertions...
}
```

### Search Tests

```rust
#[test]
fn test_finds_mate() {
    init();

    let mut board = Board::new();
    board.set_from_fen("mate_position").unwrap();

    let (best_move, score) = search::Search::new(board, 16, SearchParams::default())
        .search(Some(6));

    assert_eq!(score, MATE - 1);  // Mate in 1
}
```

### Evaluation Tests

```rust
#[test]
fn test_material_balance() {
    init();

    let mut board = Board::new();
    board.set_from_fen("startpos").unwrap();

    let score = eval::evaluate(&board, 0);
    assert!(score.abs() < 50);  // Roughly balanced
}
```

## Writing New Tests

### Test File Structure

```rust
// tests/test_new_feature.rs

use scacchista::*;

fn setup() {
    init();  // Call once per test or use lazy_static
}

#[test]
fn test_case_1() {
    setup();
    // Test implementation
}

#[test]
fn test_case_2() {
    setup();
    // Test implementation
}
```

### Test Naming Convention

```
test_<component>_<scenario>_<expected_result>
```

Examples:
- `test_material_after_queen_loss`
- `test_king_safety_exposed_center`
- `test_perft_depth_5_startpos`

### Assertions

```rust
// Equality
assert_eq!(actual, expected);

// Inequality
assert_ne!(actual, unexpected);

// Boolean
assert!(condition);
assert!(!condition);

// With message
assert_eq!(score, expected_score,
    "Position {} should evaluate to {}, got {}",
    fen, expected_score, score);

// Approximate equality
assert!((score - expected).abs() < tolerance,
    "Score {} not within {} of expected {}",
    score, tolerance, expected);
```

## Continuous Integration

### CI Checks

```bash
# Format check
cargo fmt --all -- --check

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test

# Perft smoke test
cargo run --release --bin perft -- --depth 4
```

### Pre-commit Checklist

```bash
# Run all checks
cargo fmt --all -- --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test && \
cargo run --release --bin perft -- --depth 4
```

## Debugging Test Failures

### View Output

```bash
cargo test test_name -- --nocapture
```

### Backtrace

```bash
RUST_BACKTRACE=1 cargo test test_name
```

### Single-Threaded

```bash
cargo test -- --test-threads=1
```

### Verbose Output

```bash
cargo test -- --nocapture --test-threads=1
```

## Known Test Positions

### Tactical Positions

| Name | FEN | Test |
|------|-----|------|
| Back Rank Mate | `6k1/5ppp/8/8/8/8/5PPP/4Q1K1 w - - 0 1` | Qe8# |
| Scholar's Mate | After 1.e4 e5 2.Bc4 Nc6 3.Qh5 | Qxf7# |
| Hanging Queen | Various | Capture check |

### Perft Positions

| Name | FEN | Notes |
|------|-----|-------|
| Startpos | Standard | Baseline |
| Kiwipete | Complex with castling | Good stress test |
| Position 3 | En passant heavy | EP validation |
| Position 4 | Promotion heavy | Promo validation |

## Test Coverage

Current coverage areas:

- [x] Move generation (perft)
- [x] Legality checking
- [x] Make/unmake correctness
- [x] Material evaluation
- [x] PSQT scoring
- [x] King safety
- [x] Development penalty
- [x] Mate detection
- [x] UCI parsing
- [x] TT operations
- [ ] Draw detection (planned)
- [ ] Endgame recognition (planned)

---

**Related Documents:**
- [Development Setup](./setup.md)
- [Benchmarking Guide](./benchmarking.md)
- [Contributing Guidelines](./contributing.md)
