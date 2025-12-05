# Contributing Guidelines

Thank you for your interest in contributing to Scacchista! This document explains how to contribute effectively.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Create a feature branch** from `master`
4. **Make changes** following the guidelines below
5. **Submit a Pull Request**

## Development Workflow

### Branch Naming

```
feature/description     # New features
fix/description         # Bug fixes
docs/description        # Documentation changes
perf/description        # Performance improvements
refactor/description    # Code refactoring
```

Examples:
- `feature/passed-pawn-eval`
- `fix/castling-rights-bug`
- `perf/lazy-evaluation`

### Before Making Changes

1. **Read the documentation**
   - [Architecture Overview](../architecture/overview.md)
   - [Development Setup](./setup.md)

2. **Understand the codebase**
   - Start with `src/main.rs` (entry point)
   - Review `CLAUDE.md` for project conventions

3. **Check existing issues/PRs**
   - Avoid duplicate work
   - Discuss major changes first

### Making Changes

1. **Keep changes focused**
   - One feature/fix per PR
   - Small, reviewable commits

2. **Write tests**
   - Test new functionality
   - Test edge cases
   - See [Testing Guide](./testing.md)

3. **Update documentation**
   - Update relevant docs
   - Add comments for complex code

### Commit Messages

Follow this format:

```
type: short description (50 chars)

Longer description if needed (wrap at 72 chars).
Explain what and why, not how.

Refs: #issue-number (if applicable)
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `perf`: Performance improvement
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

Examples:
```
feat: implement passed pawn evaluation

Add detection of passed pawns using bitboard masks.
Apply progressive bonus based on rank advancement.

Refs: #42
```

```
fix: correct castling rights after rook capture

The castling rights were not being cleared when an opponent
captured a rook on its starting square.
```

## Code Standards

### Rust Style

- Follow standard Rust formatting (`cargo fmt`)
- No clippy warnings (`cargo clippy`)
- Use meaningful variable names
- Add doc comments for public APIs

### Code Organization

```rust
// Module header
//! Brief description of the module.

// Imports (grouped and sorted)
use std::collections::HashMap;

use crate::board::Board;
use crate::types::Move;

// Constants
const MAX_PLY: usize = 128;

// Types
pub struct Searcher { ... }

// Implementations
impl Searcher { ... }

// Tests
#[cfg(test)]
mod tests { ... }
```

### Function Documentation

```rust
/// Evaluates the position from the perspective of the side to move.
///
/// # Arguments
///
/// * `board` - The current board position
/// * `ply` - Distance from root (for mate scoring)
///
/// # Returns
///
/// Score in centipawns, positive favors side to move.
///
/// # Example
///
/// ```
/// let score = evaluate(&board, 0);
/// ```
pub fn evaluate(board: &Board, ply: u8) -> i16 {
    // ...
}
```

### Testing Requirements

Every PR should:

1. **Pass all existing tests**
   ```bash
   cargo test
   ```

2. **Add new tests for new features**
   ```rust
   #[test]
   fn test_new_feature() { ... }
   ```

3. **Validate move generation**
   ```bash
   cargo run --release --bin perft -- --depth 5
   ```

4. **Check formatting and lints**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

## Pull Request Process

### Before Submitting

- [ ] All tests pass
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Perft results unchanged
- [ ] Documentation updated
- [ ] Commit messages follow convention

### PR Description Template

```markdown
## Summary

Brief description of the changes.

## Changes

- Change 1
- Change 2

## Testing

Describe how you tested the changes.

## Performance Impact

Any performance changes (benchmarks if applicable).

## Related Issues

Closes #issue-number
```

### Review Process

1. Automated checks run
2. Code review by maintainers
3. Address feedback
4. Squash and merge

## Types of Contributions

### Bug Fixes

- Search the issues first
- Include reproduction steps
- Add regression tests

### New Features

- Discuss in issues first
- Start small, iterate
- Include tests and documentation

### Performance Improvements

- Include before/after benchmarks
- Use profiling data
- See [Benchmarking Guide](./benchmarking.md)

### Documentation

- Fix typos
- Improve explanations
- Add examples

## Areas Looking for Help

### High Priority

- Draw detection (threefold repetition, 50-move)
- Endgame recognition (KR vs K, etc.)
- Lazy-SMP diversity for better scaling

### Medium Priority

- Passed pawn evaluation
- Bishop pair bonus
- Pawn structure evaluation

### Nice to Have

- Opening book improvements
- Syzygy tablebase integration
- NNUE evaluation (long-term)

## Code of Conduct

- Be respectful and constructive
- Welcome newcomers
- Focus on the code, not the person
- Assume good intent

## Getting Help

- Check documentation first
- Search existing issues
- Ask in issue discussions
- Be specific about your question

## Recognition

Contributors are recognized in:
- Git commit history
- PR acknowledgments
- Release notes

---

**Thank you for contributing to Scacchista!**

**Related Documents:**
- [Development Setup](./setup.md)
- [Testing Guide](./testing.md)
- [Architecture Overview](../architecture/overview.md)
