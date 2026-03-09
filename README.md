# Scacchista

[![CI](https://github.com/gaspox/Scacchista/actions/workflows/ci.yml/badge.svg)](https://github.com/gaspox/Scacchista/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.4.1-green.svg)](https://github.com/gaspox/Scacchista/releases)

A UCI-compliant chess engine written in Rust, featuring alpha-beta search with parallel lazy-SMP threading and hand-crafted evaluation.

## Features

### Search
- **Alpha-beta** with PVS (Principal Variation Search)
- **Aspiration windows** for iterative deepening
- **Quiescence search** for tactical stability
- **Pruning techniques**: null-move, LMR (Late Move Reductions), futility
- **Move ordering**: transposition table, MVV-LVA, killer moves, history heuristic
- **Lazy-SMP** parallel search (multi-threaded)

### Evaluation (HCE)
- Material + Piece-Square Tables (PSQT)
- King safety (castling rights, center exposure, pawn shield)
- Development penalties for unmoved pieces
- Center control
- Advanced passed pawn bonuses

### UCI Protocol
- Full UCI compliance
- Configurable options (Hash, Threads, Style, etc.)
- Support for FEN positions
- Time management (movetime, wtime/btime, increment)

## Quick Start

### Build

```bash
# Clone the repository
git clone https://github.com/gaspox/Scacchista.git
cd Scacchista

# Build release binary (optimized)
cargo build --release

# Binary location
./target/release/scacchista
```

### Run

```bash
# Interactive UCI mode
./target/release/scacchista

# Example UCI commands
uci
isready
position startpos moves e2e4 e7e5
go depth 10
quit
```

### Test

```bash
# Run all tests
cargo test

# Run perft tests specifically
cargo test --test perft_deep

# Run with verbose output
cargo test -- --nocapture
```

## Performance

### Search Speed

| Depth | Time (startpos) |
|-------|-----------------|
| 6 | ~0.8s |
| 8 | ~14s |
| 10 | ~3 min |

**Perft**: ~4.3M nodes/sec

### Strength Estimate

**ELO**: ~1260-1430 (estimated, based on tactical test suites)

**Note**: This is a hobby engine focused on clean architecture and educational value rather than maximum strength.

## Documentation

Detailed documentation available in [`docs/`](./docs/):

- [Architecture Overview](./docs/architecture/overview.md) - System design and module structure
- [Search Engine](./docs/search-engine.md) - Alpha-beta, pruning, move ordering
- [Evaluation](./docs/evaluation.md) - HCE components and PSQT
- [Threading](./docs/threading.md) - Lazy-SMP parallel search
- [UCI Options Reference](./docs/reference/uci-options.md) - Available configuration
- [Development Setup](./docs/development/setup.md) - How to contribute

## Development

### Prerequisites

- Rust 1.75 or later
- Cargo (comes with Rust)

### Project Structure

```
scacchista/
├── src/
│   ├── main.rs           # UCI loop entry point
│   ├── board.rs          # Bitboard representation
│   ├── eval.rs           # Hand-crafted evaluation
│   ├── search/           # Search engine modules
│   │   ├── search.rs     # Alpha-beta implementation
│   │   ├── thread_mgr.rs # Lazy-SMP threading
│   │   ├── tt.rs         # Transposition table
│   │   └── ...
│   ├── uci/              # UCI protocol
│   └── time/             # Time management
├── tests/                # Integration tests
├── docs/                 # Documentation
└── Cargo.toml
```

### Running Checks

```bash
# Format code
cargo fmt

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Build release
cargo build --release
```

## License

This project is licensed under the **GNU General Public License v3.0 or later** (GPL-3.0-or-later).

See [LICENSE](LICENSE) for details.

### License Note

Scacchista uses the [`shakmaty`](https://github.com/niklasf/shakmaty) crate (GPL-3.0), which means all derivative works (including compiled binaries) must also be GPL-3.0 compatible.

## Releases

See [Releases](https://github.com/gaspox/Scacchista/releases) for precompiled binaries.

Latest: **v0.4.1** - Bug fixes (move generation for pawn promotions, time management for fixed movestogo/movestogo), and expanded regression test suite (tactical, draw detection, threading stress).

## Acknowledgments

- **shakmaty** by [@niklasf](https://github.com/niklasf) - Excellent Rust chess library for move validation
- Chess Programming Wiki - Invaluable resource for chess engine development
- Stockfish community - Inspiration and algorithmic insights

## Disclaimer

This is a personal learning project and experimental chess engine. It is **not intended for production use** or competitive play. The primary goals are:

- Educational: exploring chess engine algorithms and Rust programming
- Showcase: demonstrating software architecture and problem-solving skills
- Open Source: contributing to the Rust chess programming community

The engine prioritizes **code clarity and maintainability** over maximum playing strength.

---

**Made with Rust 🦀**
