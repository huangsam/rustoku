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

Rustoku uses automated GitHub Actions workflows with keyless **OIDC Trusted Publishing** for all package registries (crates.io, PyPI, npm).

### 1. Update `CHANGELOG.md`
Document changes for the release according to existing conventions and commit:
```bash
git add CHANGELOG.md
git commit -m "docs: prepare v0.X.Y changelog"
git push origin main
```

### 2. Bump & Tag Release
Use `cargo-release` to bump workspace versions, create tags, and push:
```bash
# New feature (minor bump)
cargo release minor --execute

# Bug fix / doc patch (patch bump)
cargo release patch --execute
```
*`cargo release` is configured via `release.toml` (`publish = false`) to handle version bumping, git committing, tagging (`rustoku-lib-v*`, `rustoku-cli-v*`), and pushing to GitHub without requiring local crates.io credentials.*

### 3. Automated CI/CD Publishing
Pushing the release tags automatically triggers three parallel keyless GitHub Actions workflows using OIDC Trusted Publishing:
- **`crates-release.yml`**: Authenticates via `rust-lang/crates-io-auth-action` and publishes `rustoku-lib` and `rustoku-cli` to [crates.io](https://crates.io).
- **`py-release.yml`**: Builds Linux, macOS, and Windows wheels + sdist and publishes `rustoku` to [PyPI](https://pypi.org/project/rustoku/).
- **`wasm-release.yml`**: Builds WASM binaries and publishes `rustoku-wasm` to [npm](https://www.npmjs.com/package/rustoku-wasm) with cryptographic provenance.

### 4. Verify
- Rust crates: [crates.io/crates/rustoku-lib](https://crates.io/crates/rustoku-lib) & [crates.io/crates/rustoku-cli](https://crates.io/crates/rustoku-cli)
- Python package: [pypi.org/project/rustoku/](https://pypi.org/project/rustoku/)
- npm package: [npmjs.com/package/rustoku-wasm](https://www.npmjs.com/package/rustoku-wasm)
- GitHub Actions: [Actions Dashboard](https://github.com/huangsam/rustoku/actions)
