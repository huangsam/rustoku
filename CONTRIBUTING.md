# Contributing to Rustoku

Thank you for your interest in contributing to Rustoku! This document covers development workflows, quality checks, and maintainer release steps.

## Build & Quality Checks

Run before submitting a PR:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release          # builds rustoku-lib + rustoku-cli
cargo bench                    # in rustoku-lib/benches/

# Binding crates checks
cargo check -p rustoku-py -p rustoku-wasm
cargo clippy --no-deps -p rustoku-py -p rustoku-wasm
cd rustoku-wasm && wasm-pack build && npm test
```

## Maintainer Release Process

Rustoku uses `cargo-release` for workspace versioning and crates.io publishing. `rustoku-py` and `rustoku-wasm` are marked `publish = false` in Cargo and are distributed via their respective package registries.

### 1. Update `CHANGELOG.md`
Document changes for the release according to existing conventions.

### 2. Bump & Publish Rust Crates (crates.io)
```bash
# New feature (minor bump)
cargo release minor --execute

# Bug fix / doc patch (patch bump)
cargo release patch --execute
```
*Requires `cargo login` with a valid crates.io API token. This bumps workspace versions, creates git tags, publishes `rustoku-lib` and `rustoku-cli`, and pushes commits and tags to GitHub.*

### 3. Publish Python Package (PyPI)
```bash
cd rustoku-py
maturin publish --token <PYPI_TOKEN>
# or: MATURIN_PYPI_TOKEN=... maturin publish --non-interactive
```
*Builds the release wheel + source distribution (`.tar.gz`) and uploads to PyPI.*

### 4. Publish WebAssembly Package (npm)
```bash
cd rustoku-wasm
wasm-pack build --release
cd pkg
npm publish --access public
```
*Requires `npm login`. Publishes compiled WASM binary, JS glue code, and TypeScript typings to npm.*

### 5. Verify
- Rust crates: [crates.io/crates/rustoku-lib](https://crates.io/crates/rustoku-lib)
- Python package: [pypi.org/project/rustoku/](https://pypi.org/project/rustoku/)
- npm package: [npmjs.com/package/rustoku-wasm](https://www.npmjs.com/package/rustoku-wasm)
- Git tags visible on GitHub
