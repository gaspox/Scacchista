# Project Guide for Claude Code

This document provides guidance for Claude Code when working with the Scacchista codebase.

> **Note:** This file is the canonical reference for Claude Code. The root `CLAUDE.md` file may contain a subset of this information.

## Quick Commands

| Command | Description |
|---------|-------------|
| `cargo build` | Debug build |
| `cargo build --release` | Release build (LTO enabled) |
| `cargo run` | Run UCI engine |
| `cargo run --bin perft -- --depth <N>` | Run perft |
| `cargo test` | Run all tests |
| `cargo test <name>` | Run specific test |
| `cargo clippy --all-targets --all-features -- -D warnings` | Lint |
| `cargo fmt --all -- --check` | Format check |

## High-Level Architecture

```
UCI Interface (src/uci/)
    │
    ▼
Search Engine (src/search/search.rs)
    │
    ├─ Transposition Table (src/search/tt.rs)
    ├─ Thread Manager (src/search/thread_mgr.rs)
    └─ Time Manager (src/time/mod.rs)
    │
    ▼
Evaluation (src/eval.rs)
    │
    ▼
Board Representation (src/board.rs)
```

### Key Components

- **UCI Interface**: `src/uci/` - Command parsing, main loop, options
- **Search Core**: `src/search/search.rs` - Alpha-beta, PVS, quiescence
- **Evaluation**: `src/eval.rs` - Material, PSQT, king safety
- **Board**: `src/board.rs` - Bitboards, move generation, make/unmake
- **Time**: `src/time/mod.rs` - Time allocation
- **Threading**: `src/search/thread_mgr.rs` - Lazy-SMP

## Where to Start

### For UCI Changes
1. `src/uci/mod.rs` - Re-exports
2. `src/uci/loop.rs` - Main UCI loop
3. `src/uci/parser.rs` - Command parsing
4. `src/uci/options.rs` - UCI options

### For Search Changes
1. `src/search/search.rs` - Core alpha-beta
2. `src/search/tt.rs` - Transposition table
3. `src/search/thread_mgr.rs` - Parallelism
4. `src/search/params.rs` - Search parameters

### For Evaluation Changes
1. `src/eval.rs` - All evaluation logic
   - PSQT tables
   - King safety
   - Development penalty
   - Center control

### For Board/Move Generation
1. `src/board.rs` - Bitboard representation
2. `src/zobrist.rs` - Hashing
3. `src/utils.rs` - Attack tables

## Project Structure

```
Tal/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Module exports
│   ├── board.rs             # Board representation (~2000 lines)
│   ├── eval.rs              # Evaluation (~500 lines)
│   ├── zobrist.rs           # Zobrist hashing
│   ├── utils.rs             # Utilities
│   ├── uci/                 # UCI protocol
│   ├── search/              # Search engine
│   ├── time/                # Time management
│   └── bin/                 # Utility binaries
├── tests/                   # Integration tests
├── docs/                    # Documentation
└── CLAUDE.md               # Quick reference
```

## Important Crates

| Crate | Purpose |
|-------|---------|
| `shakmaty` | Chess rules, FEN, validation |
| `shakmaty-uci` | UCI move parsing |
| `shakmaty-syzygy` | Tablebase probing |
| `serde` + `bincode` | Serialization |

## Editing Guidelines

### Preferences

1. **Edit existing files** over creating new ones
2. **Run validation** after changes:
   ```bash
   cargo fmt && cargo clippy && cargo test
   ```
3. **Match project style** for dependencies and code
4. **Use release builds** for performance testing

### Testing Requirements

After any change:

```bash
# Format
cargo fmt

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Test
cargo test

# Perft (for move gen changes)
cargo run --release --bin perft -- --depth 5
```

### Performance Testing

Always use release builds:

```bash
cargo build --release
time printf 'uci\nposition startpos\ngo depth 8\nquit\n' | ./target/release/scacchista
```

## Implementation Status

### Complete

- [x] Basic Alpha-Beta + TT
- [x] Move ordering (MVV-LVA, killers, history)
- [x] Aspiration windows
- [x] Quiescence search
- [x] UCI interface
- [x] Time management
- [x] Lazy-SMP threading
- [x] Null-move pruning
- [x] LMR
- [x] Futility pruning
- [x] PSQT evaluation
- [x] King safety

### Planned

- [ ] Draw detection
- [ ] Endgame recognition
- [ ] Passed pawn evaluation
- [ ] Magic bitboards
- [ ] NNUE (long-term)

## CI Expectations

```bash
# Pre-commit checks
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Communication Style

When working with this codebase:

- **Language**: Follow user's language (Italian if they write in Italian)
- **Style**: Conversational, step-by-step explanations
- **Analogies**: Use chess metaphors where helpful
- **Technical terms**: Define on first use
- **Verification**: Ask clarifying questions when ambiguous
- **Examples**: Include code snippets and commands

## Known Issues

1. **Lazy-SMP scaling limited** - Needs diversity layer
2. **King legality check** - Some illegal moves at depth 3+
3. **Draw detection missing** - Threefold, 50-move not implemented

## Quick References

### Perft Values (startpos)

| Depth | Nodes |
|-------|-------|
| 5 | 4,865,609 |
| 6 | 119,060,324 |

### Performance Targets

| Depth | Time |
|-------|------|
| 6 | ~568ms |
| 8 | ~15s |

---

**Related Documents:**
- [Architecture Overview](../architecture/overview.md)
- [Development Setup](../development/setup.md)
- [Handoff Document](./handoff.md)
