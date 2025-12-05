# Development Setup

This guide covers how to build, run, and develop Scacchista.

## Prerequisites

- **Rust**: Edition 2021 or later
- **Cargo**: Included with Rust
- **Git**: For version control
- **Optional**: `perf` or `cargo-flamegraph` for profiling

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Getting Started

### Clone the Repository

```bash
git clone git@github.com:gaspox/Scacchista.git
cd Scacchista
```

### Build

```bash
# Debug build (fast compilation, slow execution)
cargo build

# Release build (slow compilation, fast execution)
# ALWAYS use for benchmarking!
cargo build --release
```

### Run

```bash
# Run the UCI engine (debug)
cargo run

# Run the UCI engine (release)
cargo run --release

# Or run the binary directly
./target/release/scacchista
```

## Quick Commands Reference

| Command | Description |
|---------|-------------|
| `cargo build` | Debug build |
| `cargo build --release` | Release build (LTO enabled) |
| `cargo run` | Run UCI engine |
| `cargo test` | Run all tests |
| `cargo test <name>` | Run specific test |
| `cargo clippy --all-targets --all-features -- -D warnings` | Lint |
| `cargo fmt --all -- --check` | Format check |
| `cargo fmt` | Auto-format code |

## Utility Binaries

### Perft (Move Generation Validation)

```bash
# Run perft at depth 5
cargo run --release --bin perft -- --depth 5

# Run perft on specific position
cargo run --release --bin perft -- --depth 4 \
  --fen "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"
```

### Search Tests

```bash
# Simple search test
cargo run --release --bin simple_search_test

# Stress test
cargo run --release --bin stress_search_test
```

### Debug Utilities

```bash
cargo run --bin debug_perft
cargo run --bin test_board
cargo run --bin test_qsearch_simple
cargo run --bin compare_moves
cargo run --bin debug_perft_divide
cargo run --bin test_make_unmake
```

## UCI Interaction

### Manual Testing

```bash
# Start the engine
./target/release/scacchista

# Type UCI commands:
uci
isready
position startpos
go depth 6
quit
```

### Scripted Testing

```bash
# One-liner test
echo -e "uci\nisready\nposition startpos\ngo depth 6\nquit" | ./target/release/scacchista

# With timing
time printf 'uci\nposition startpos\ngo depth 8\nquit\n' | ./target/release/scacchista
```

## Project Structure

```
Tal/
├── Cargo.toml           # Project configuration
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Library exports
│   ├── board.rs         # Board representation
│   ├── eval.rs          # Evaluation function
│   ├── zobrist.rs       # Zobrist hashing
│   ├── utils.rs         # Utilities
│   ├── uci/             # UCI protocol
│   │   ├── mod.rs
│   │   ├── loop.rs
│   │   ├── parser.rs
│   │   └── options.rs
│   ├── search/          # Search engine
│   │   ├── mod.rs
│   │   ├── search.rs
│   │   ├── tt.rs
│   │   ├── thread_mgr.rs
│   │   ├── stats.rs
│   │   └── params.rs
│   ├── time/            # Time management
│   │   └── mod.rs
│   └── bin/             # Utility binaries
│       ├── perft.rs
│       ├── simple_search_test.rs
│       └── stress_search_test.rs
├── tests/               # Integration tests
├── docs/                # Documentation
├── CLAUDE.md            # Claude Code instructions
└── README.md            # Project overview
```

## Configuration

### Cargo.toml Release Profile

```toml
[profile.release]
lto = true              # Link-Time Optimization
codegen-units = 1       # Single codegen unit for max optimization
opt-level = 3           # Maximum optimization
```

### RUSTFLAGS for Maximum Performance

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## IDE Setup

### VS Code

Recommended extensions:
- `rust-analyzer` - Rust language support
- `CodeLLDB` - Debugging support
- `Even Better TOML` - Cargo.toml support

### settings.json

```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all"
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level | `info` |
| `RUST_BACKTRACE` | Show backtrace on panic | `0` |

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable backtrace
RUST_BACKTRACE=1 cargo test
```

## Common Development Tasks

### Adding a Dependency

```bash
cargo add serde --features derive
```

### Running a Single Test

```bash
cargo test test_name -- --nocapture
```

### Checking for Issues

```bash
# Full check: format, lint, test
cargo fmt --all -- --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test
```

### Generating Documentation

```bash
cargo doc --open
```

## Troubleshooting

### Build Errors

```bash
# Clean and rebuild
cargo clean && cargo build
```

### Test Failures

```bash
# Run with output
cargo test -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test
```

### Performance Issues

Always use release builds for performance testing:

```bash
# Debug build: ~10x slower
cargo run

# Release build: optimized
cargo run --release
```

---

**Related Documents:**
- [Testing Guide](./testing.md)
- [Benchmarking Guide](./benchmarking.md)
- [Contributing Guidelines](./contributing.md)
