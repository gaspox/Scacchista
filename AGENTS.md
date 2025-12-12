# AGENTS.md

This file provides guidance to coding agents when working with this repository.

> **Full documentation:** See [`docs/claude-code/project-guide.md`](./docs/claude-code/project-guide.md)

## Essential Commands
- **Build**: `cargo build` (debug) | `cargo build --release` (release with LTO)
- **Test all**: `cargo test`
- **Single test**: `cargo test <test_name>` (e.g., `cargo test perft`)
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Format**: `cargo fmt` (check: `cargo fmt --all -- --check`)
- **Run**: `cargo run` or `cargo run --bin scacchista`
- **Perft**: `cargo run --release --bin perft -- --depth <N>`

## After Changes
```bash
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

## Code Style Guidelines
- **Naming**: `snake_case` functions/variables, `PascalCase` types/enums/structs
- **Imports**: Group std → external crates → `crate::` internal modules
- **Types**: Prefer `Option<T>` over sentinel values; use `Result` for recoverable errors
- **Errors**: `panic!()` only for invariant violations; document with comments
- **Docs**: `//!` for module-level, `///` for items; explain "why" not just "what"
- **Formatting**: Run `cargo fmt` before committing; 100-char line limit (soft)
- **Performance**: Use release builds (`--release`) for performance testing

## Project Structure

```
src/
├── main.rs          # Entry point
├── lib.rs           # Module exports
├── board.rs         # Bitboard representation
├── eval.rs          # Evaluation function
├── zobrist.rs       # Zobrist hashing
├── utils.rs         # Utilities
├── uci/             # UCI protocol
├── search/          # Search engine
├── time/            # Time management
└── bin/             # Utility binaries
```

## Where to Start

- **UCI changes**: `src/uci/loop.rs`, `parser.rs`, `options.rs`
- **Search changes**: `src/search/search.rs`, `tt.rs`, `thread_mgr.rs`
- **Evaluation**: `src/eval.rs`
- **Board/moves**: `src/board.rs`, `zobrist.rs`

## After Changes

```bash
cargo fmt && cargo clippy && cargo test
```

For move generation changes, also run perft:
```bash
cargo run --release --bin perft -- --depth 5
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
