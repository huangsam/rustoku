# Agent Guidelines for Rustoku

## Core Invariants
- **Bitmask Integrity**: All cell constraints and candidate tracking live in `rustoku-lib/src/core/`. Solving and state changes must preserve bitmask invariants across `Masks` and `Candidates`.
- **Binding Parity**: Public bindings in `rustoku-py` and `rustoku-wasm` delegate to `rustoku-lib::bind`. Keep their exported APIs and signatures in sync.

## Build & Quality Checks
Run before completing tasks:
```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo bench                    # in rustoku-lib/benches/

# Binding checks
cargo check -p rustoku-py -p rustoku-wasm
cd rustoku-wasm && wasm-pack build && npm test
```

## Performance Expectations
- `solve_any`: ~10–50μs per typical puzzle.
- `solve_all`: ~20–100μs depending on complexity.
- Solver memory footprint: ~1KB total (`Board` 81B + `Masks` 108B + `Candidates` 324B).
- Changes must not regress Criterion benchmarks.
