# Contributing to Rustoku

Thank you for your interest in contributing to Rustoku! This document covers development reference, coding guidelines, and publishing instructions.

## Potential Improvements

- **Profile-Guided Optimization (PGO)**: Squeeze out an additional 10-20% performance by using real-world execution profiles to optimize machine code layout.
- **WebAssembly (WASM) Support**: Enable zero-latency Sudoku analysis directly in the browser for web-based games and tools.
- **Python Bindings (PyO3)**: Offer high-performance generation and solving to the Python data science and AI community.
- **Serde Serialization**: Facilitate easy persistence and JSON exchange of boards, solutions, and solve paths.
- **Property-based Testing (`proptest`)**: Mathematically verify correctness against millions of randomized valid/invalid inputs.
- **Fuzz Testing (`cargo-fuzz`)**: Automatically identify obscure panics or edge cases by bombarding the solver with mutated inputs.
- **Export Formats**: Allow users to save puzzles and solutions as JSON (for automation) or PNG/SVG (for sharing).

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
