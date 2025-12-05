# Scacchista Documentation

Welcome to the Scacchista chess engine documentation. This directory contains comprehensive technical documentation organized by topic and audience.

## Quick Navigation

| Section | Description | Audience |
|---------|-------------|----------|
| [Architecture](./architecture/) | Engine design and internals | Developers |
| [Development](./development/) | Setup, testing, contributing | Contributors |
| [Reference](./reference/) | UCI options, performance, roadmap | All users |
| [Claude Code](./claude-code/) | AI assistant integration | Claude Code |

## Documentation Structure

```
docs/
├── architecture/           # Engine design and internals
│   ├── overview.md        # High-level architecture
│   ├── search-engine.md   # Alpha-beta, TT, pruning
│   ├── evaluation.md      # HCE, PSQT, king safety
│   └── threading.md       # Lazy-SMP parallel search
│
├── development/           # For developers and contributors
│   ├── setup.md          # Build, run, test commands
│   ├── testing.md        # Test strategy and validation
│   ├── benchmarking.md   # Performance measurement
│   └── contributing.md   # Contribution guidelines
│
├── reference/             # Reference documentation
│   ├── uci-options.md    # Available UCI options
│   ├── performance.md    # Current performance metrics
│   ├── roadmap.md        # Future development plans
│   └── changelog.md      # Version history
│
└── claude-code/           # Claude Code specific
    ├── project-guide.md  # Project instructions
    ├── agents.md         # Specialized agents config
    └── handoff.md        # Technical handoff document
```

## Quick Links

- **Getting Started**: [Development Setup](./development/setup.md)
- **Architecture Overview**: [System Design](./architecture/overview.md)
- **UCI Options**: [Configuration Reference](./reference/uci-options.md)
- **Performance Data**: [Benchmark Results](./reference/performance.md)
- **Roadmap**: [Future Plans](./reference/roadmap.md)

## Document Conventions

- All paths are relative to the project root (`/home/gaspare/Documenti/Tal/`)
- Code examples use Rust syntax unless otherwise noted
- Performance metrics are measured on release builds with `--release` flag
- UCI commands are shown in lowercase as per protocol specification

## Contributing to Documentation

When adding or updating documentation:

1. Place files in the appropriate category directory
2. Update this index if adding new sections
3. Use relative links between documents
4. Follow the existing markdown style
5. Include practical examples where applicable

---

**Project**: Scacchista UCI Chess Engine
**Version**: 0.2.1-beta
**Last Updated**: December 2025
