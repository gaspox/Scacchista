# Scacchista

A UCI chess engine written in Rust.

## Features

- UCI protocol compliant
- Alpha-beta search with PVS, aspiration windows, quiescence
- Pruning: null-move, LMR, futility
- Move ordering: TT, MVV-LVA, killers, history heuristic
- HCE evaluation: material, PSQT, king safety, development
- Lazy-SMP parallel search
- Polyglot opening book support
- Syzygy tablebase probing

## Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/scacchista

# Test
cargo test
```

## UCI Example

```
uci
isready
position startpos
go depth 10
quit
```

## Documentation

Full documentation available in [`docs/`](./docs/):

- [Architecture Overview](./docs/architecture/overview.md)
- [Development Setup](./docs/development/setup.md)
- [UCI Options Reference](./docs/reference/uci-options.md)
- [Performance Metrics](./docs/reference/performance.md)
- [Roadmap](./docs/reference/roadmap.md)

## Performance

| Depth | Time |
|-------|------|
| 6 | ~568ms |
| 8 | ~15s |

Perft: ~4.3M nodes/sec

## License

This project uses the `shakmaty` crate (GPL-3.0). See license implications before distributing binaries.

## Contributing

See [Contributing Guidelines](./docs/development/contributing.md).
