# AGENTS.md

This file provides guidance to coding agents when working with this repository.

> **Full documentation:** See [`docs/claude-code/project-guide.md`](./docs/claude-code/project-guide.md)

## Essential Commands

- **Build**: `cargo build` (debug) | `cargo build --release` (release with LTO)
- **Test all**: `cargo test`
- **Single test**: `cargo test <test_name>` (e.g., `cargo test test_material_eval_sign`)
- **Test specific file**: `cargo test --test <test_file>` (e.g., `cargo test --test test_material_eval_direct`)
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Format**: `cargo fmt` (check: `cargo fmt --all -- --check`)
- **Run**: `cargo run` or `cargo run --bin scacchista`
- **Perft**: `cargo run --release --bin perft -- --depth <N>`

## After Changes

```bash
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

For move generation changes, also run perft:
```bash
cargo run --release --bin perft -- --depth 5
```

**NOTE**: The lib.rs file currently has `#![allow(...)]` for many clippy lints, so clippy warnings are not enforced. However, you should still run clippy to check for real issues.

## Code Style Guidelines

### Naming Conventions
- **Functions/variables**: `snake_case` (e.g., `calculate_lmr_reduction`, `piece_index`)
- **Types/Enums/Structs**: `PascalCase` (e.g., `Move`, `Color`, `PieceKind`)
- **Constants**: `UPPERCASE_SNAKE_CASE` (e.g., `FLAG_EN_PASSANT`, `INFINITE`)
- **Type aliases**: `PascalCase` (e.g., `pub type Move = u32;`)

### Import Organization
Group imports in this order:
1. `use std::...` (standard library)
2. External crates (e.g., `use shakmaty::...`)
3. `use crate::...` (internal modules)

```rust
use std::collections::HashMap;
use std::sync::Arc;

use crate::board::{Board, Move};
use crate::search::Search;
```

### Types and Error Handling
- Prefer `Option<T>` over sentinel values for nullable data
- Use `Result<T, E>` for recoverable errors
- Use `unwrap()` only when you're certain the operation cannot fail (tests, initialization)
- Use `expect()` with a descriptive message when you want better panic output
- Avoid `panic!()` in production code except for invariant violations

### Documentation
- `//!` for module-level documentation at top of file
- `///` for items (functions, structs, enums)
- Explain "why" not just "what"
- Include examples for complex logic

### Formatting
- Run `cargo fmt` before committing
- 100-char line limit (soft)
- Use 4-space indentation (Rust default)

### Performance
- Use `--release` builds for performance testing
- Profile with `cargo run --release --bin profile_search`
- Prefer bit operations for board representation
- Use `#[inline]` for small, hot functions

### Move Encoding
Moves are 32-bit integers:
- Bits 0-5: from square (0-63)
- Bits 6-11: to square (0-63)
- Bits 12-15: piece type
- Bits 16-19: captured piece
- Bits 20-23: promotion piece
- Bits 24-31: flags (en passant, castling, promotion, capture)

Use helper functions: `move_from_sq()`, `move_to_sq()`, `move_piece()`, `move_captured()`, `move_flag()`

## Project Structure

```
src/
├── main.rs          # Entry point
├── lib.rs           # Module exports (allows clippy lints)
├── board.rs         # Bitboard representation, move generation
├── eval.rs          # Evaluation function (PSQT, material)
├── zobrist.rs       # Zobrist hashing
├── utils.rs         # Utilities (attack tables, bit operations)
├── magic.rs         # Magic bitboards for slider attacks
├── uci/             # UCI protocol implementation
│   ├── mod.rs       # Module exports
│   ├── loop.rs      # UCI main loop
│   ├── parser.rs    # UCI command parsing
│   └── options.rs   # UCI options handling
├── search/          # Search engine
│   ├── mod.rs       # Module exports
│   ├── search.rs    # Main alpha-beta search
│   ├── tt.rs        # Transposition table
│   ├── thread_mgr.rs# Thread management (lazy-SMP)
│   ├── stats.rs     # Search statistics
│   └── params.rs    # Search parameters
├── time/            # Time management
└── bin/             # Utility binaries
    ├── perft.rs     # Perft testing
    ├── simple_search_test.rs
    └── stress_search_test.rs
tests/               # Integration tests
```

## Where to Start

- **UCI changes**: `src/uci/loop.rs`, `parser.rs`, `options.rs`
- **Search changes**: `src/search/search.rs`, `tt.rs`, `thread_mgr.rs`
- **Evaluation**: `src/eval.rs` (PSQT tables, material counting)
- **Board/moves**: `src/board.rs`, `zobrist.rs`
- **Move generation bugs**: Run `cargo run --release --bin perft -- --depth 5`

## Testing Guidelines

- Call `scacchista::init()` at start of tests to initialize static tables
- Use `assert!` with descriptive messages
- Test both positive and negative cases
- Include edge cases (empty board, single piece, etc.)

```rust
#[test]
fn test_material_eval_sign() {
    init();
    let mut board = Board::new();
    board.set_from_fen("4k3/8/8/8/8/8/8/4K2Q w - - 0 1").unwrap();
    // ... assertions
}
```

## Documentation

- [Architecture](./docs/architecture/overview.md)
- [Development Setup](./docs/development/setup.md)
- [Testing Guide](./docs/development/testing.md)
- [UCI Options](./docs/reference/uci-options.md)
- [Roadmap](./docs/reference/roadmap.md)

## Communication Style

- Follow user's language (Italian if they write in Italian)
- Use step-by-step explanations
- Define technical terms on first use
- Include code snippets and commands
- Ask clarifying questions when ambiguous
