# Contributing to Rustoku

Thank you for your interest in contributing to Rustoku! This document covers development reference, coding guidelines, and publishing instructions.

## Potential Improvements

- **Property-based Testing**: Mathematically verify correctness against millions of random inputs.
- **Export Formats**: Support saving puzzles and solutions as JSON, PNG, or SVG.
- **Profile-Guided Optimization**: Use real-world execution profiles to optimize machine code.
- **Fuzz Testing**: Identify obscure panics by stress-testing the solver with mutated inputs.

## Build & Quality Checks

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release --workspace
cargo bench
```

## Publishing a New Version

Rustoku uses `cargo-release` to handle workspace-wide versioning and publishing.

1. **Update `CHANGELOG.md`**: Document changes since the last release according to existing conventions.
2. **Execute Release**:
   ```bash
   # For a new feature (minor version)
   cargo release minor --execute

   # For bug fixes (patch version)
   cargo release patch --execute
   ```

3. **Verify**: Ensure tags are pushed to GitHub and crates are visible on [crates.io](https://crates.io).

If you have questions or need help, open an issue or PR!
