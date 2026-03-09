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
cargo build --release          # builds rustoku-lib + rustoku-cli
cargo bench
```

### Binding crates (`rustoku-py`, `rustoku-wasm`)

`rustoku-py` and `rustoku-wasm` are excluded from the default build because they
require external toolchains that link against Python / the WASM runtime.
The correct way to work with them:

```bash
# Type-check both binding crates on any platform
cargo check -p rustoku-py -p rustoku-wasm
cargo clippy --no-deps -p rustoku-py -p rustoku-wasm

# Build the Python extension (requires maturin)
cd rustoku-py && maturin develop

# Build the WASM module (requires wasm-pack)
cd rustoku-wasm && wasm-pack build --target web
```

## Publishing a New Version

Rustoku uses `cargo-release` to handle workspace-wide versioning and publishing.

`rustoku-lib` and `rustoku-cli` are published to [crates.io](https://crates.io).
`rustoku-py` and `rustoku-wasm` are marked `publish = false` — they are distributed
through their own ecosystems and are not part of the `cargo release` flow.

### 1. Update `CHANGELOG.md`
Document changes since the last release according to existing conventions.

### 2. Bump & publish Rust crates
```bash
# New feature (minor bump)
cargo release minor --execute

# Bug fix (patch bump)
cargo release patch --execute
```
This bumps the workspace version, tags the commit, and publishes `rustoku-lib`
and `rustoku-cli`. It will skip `rustoku-py` and `rustoku-wasm` automatically
because they set `publish = false`.

### 3. Publish Python package (PyPI)
```bash
cd rustoku-py
maturin publish
```
Requires PyPI credentials. `maturin publish` builds the wheel for the current
platform and uploads it. For a multi-platform release, use a CI workflow or
`maturin build --release` + `twine upload`.

### 4. Publish WASM package (npm)
```bash
cd rustoku-wasm
wasm-pack build --release --target web
wasm-pack publish
```
Requires an npm account and `wasm-pack` ≥ 0.13.

### 5. Verify
- Rust crates visible on [crates.io](https://crates.io)
- Python package visible on [PyPI](https://pypi.org)
- npm package visible on [npmjs.com](https://www.npmjs.com)
- Git tag pushed to GitHub

If you have questions or need help, open an issue or PR!
