# Agent Guidelines for Rustoku

## Core Invariants
- **Bitmask Integrity**: All cell constraints and candidate tracking live in `rustoku-lib/src/core/`. Solving and state changes must preserve bitmask invariants across `Masks` and `Candidates`.
- **Binding Parity**: Public bindings in `rustoku-py` and `rustoku-wasm` delegate to `rustoku-lib::bind`. Keep their exported APIs and signatures in sync.
- **Deterministic Solves**: Backtracking must not invoke PRNGs or shuffle operations; keep randomness isolated strictly to `solve_random` / `BoardGenerator`.
- **Lightweight Probing**: Puzzle generation and uniqueness validation must use non-allocating probes (`count_solutions_until(2)`) rather than full `solve_all` or collecting `Solution` instances.

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
cd rustoku-py && maturin develop && python3 tests/test_api.py
```

## Performance Expectations
- `solve_any`: ~1–20μs per typical puzzle (<10μs on modern hardware).
- `solve_all`: ~1–25μs depending on complexity.
- Solver memory footprint: ~1KB total (`Board` 81B + `Masks` 108B + `Candidates` 324B).
- Changes must not regress Criterion benchmarks.
