# HANDOFF.md - Scacchista Chess Engine

> **Full handoff document:** See [`docs/claude-code/handoff.md`](./docs/claude-code/handoff.md)

**Date:** December 2025
**Version:** 0.2.1-beta
**Status:** Active Development

## Quick Summary

Scacchista is a UCI chess engine in Rust with:
- 3.1x performance improvement (depth 6: 568ms)
- 80+ tests passing
- Lazy-SMP infrastructure (needs diversity for scaling)

## Quick Start

```bash
cargo build --release
cargo test
time printf 'uci\nposition startpos\ngo depth 7\nquit\n' | ./target/release/scacchista
# Expected: ~2.2s
```

## Next Tasks (Priority Order)

1. **SEE cache array** (30 min, ~5-10% speedup)
2. **Razoring** (1 hour, ~2-3% speedup)
3. **Lazy-SMP diversity** (1-2 days, +50-80% on 2 threads)
4. **Draw detection** (2-3 hours, +80-100 ELO)

## Documentation

All documentation in [`docs/`](./docs/):

| Section | Location |
|---------|----------|
| Architecture | `docs/architecture/` |
| Development | `docs/development/` |
| Reference | `docs/reference/` |
| Claude Code | `docs/claude-code/` |

## Known Issues

1. Lazy-SMP limited scaling (needs diversity)
2. King legality check bug at depth 3+
3. Draw detection not implemented

---

See full details in [`docs/claude-code/handoff.md`](./docs/claude-code/handoff.md)
