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

Rustoku is organized as a Cargo workspace with two main crates: `rustoku-lib` (core library) and `rustoku-cli` (command-line interface).

### Prerequisites

- Ensure you have a [crates.io](https://crates.io) account and are an owner of the `rustoku-lib` and `rustoku-cli` crates.
- Install Rust and Cargo (latest stable version recommended).
- Run tests and ensure everything passes: `cargo test --workspace`

### Steps to Publish

1. **Update Version Numbers**:
   - Edit `Cargo.toml` in the root (workspace) and update the `version` field.
   - Update `Cargo.toml` in `rustoku-lib/` and `rustoku-cli/` to match the new version.
   - Follow [semantic versioning](https://semver.org/) (e.g., patch for bug fixes, minor for new features).

2. **Update Documentation**:
   - Ensure `README.md` and inline docs (`///`) are up-to-date.
   - Run `cargo doc --open` to verify documentation builds correctly.

3. **Run Checks** (see Build & Quality Checks above).

4. **Login to crates.io**:
   - Run `cargo login` and enter your API token (from your crates.io account settings).

5. **Publish the Library First**:
   - Since `rustoku-cli` depends on `rustoku-lib`, publish the library first:
     ```bash
     cd rustoku-lib
     cargo publish
     ```
   - Wait for it to appear on crates.io (may take a few minutes).

6. **Publish the CLI**:
   - Once `rustoku-lib` is published:
     ```bash
     cd ../rustoku-cli
     cargo publish
     ```

7. **Verify**:
   - Check [crates.io](https://crates.io/crates/rustoku-lib) and [crates.io/crates/rustoku-cli](https://crates.io/crates/rustoku-cli) to confirm the new versions are live.
   - Test installation: `cargo install rustoku-cli`

### Common Issues

- **Dependency Conflicts**: If publishing fails due to dependencies, ensure all deps are published and compatible.
- **Yanking**: If you need to remove a version, use `cargo yank --vers <version>`.
- **Dry Run**: Test publishing without actually uploading: `cargo publish --dry-run`.

### Additional Notes

- For major changes, consider creating a GitHub release with changelog notes.
- If you're new to publishing, refer to the [Cargo documentation](https://doc.rust-lang.org/cargo/reference/publishing.html).

If you have questions or need help, open an issue or PR!
