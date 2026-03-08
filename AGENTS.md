# Agentic Documentation for Rustoku

## Overview

Rustoku is a Rust workspace with two crates: `rustoku-lib` (solver/generator library) and `rustoku-cli` (CLI tool). Rust 1.85, edition 2024, MIT license.

## Architecture

```
rustoku/
├── rustoku-lib/src/   # Core library
│   ├── core/          # Rustoku solver, Board, Masks, Candidates, TechniqueFlags
│   ├── error.rs       # RustokuError enum
│   └── format.rs      # Display impls for boards, solutions, solve paths
└── rustoku-cli/src/   # CLI using clap (generate, solve, check, show)
```

- **`Rustoku`** is the central struct (board + bitmask constraints + candidate cache + technique flags)
- Solving uses backtracking with MRV heuristic and optional human techniques (naked/hidden singles & pairs)
- Generation fills a board then removes clues while preserving solution uniqueness
- All constraints tracked via `u32` bitmasks for O(1) validation

## Key APIs

| Function | Purpose |
|---|---|
| `Rustoku::new(board)` / `::new_from_str(s)` | Construct solver |
| `.with_techniques(flags)` | Enable human-like techniques |
| `.solve_any()` → `Option<Solution>` | Find one solution |
| `.solve_all()` → `Vec<Solution>` | Find all solutions |
| `.solve_until(n)` → `Vec<Solution>` | Find up to *n* solutions |
| `.is_solved()` → `bool` | Validate board completeness |
| `generate_board(clues)` → `Result<Board>` | Generate puzzle (17–81 clues) |

## CLI Commands

```bash
rustoku-cli generate --clues 25
rustoku-cli solve any --verbose --human "<81-char puzzle>"
rustoku-cli solve all "<81-char puzzle>"
rustoku-cli check "<81-char solution>"
rustoku-cli show "<81-char puzzle>"
```

Input: 81-char string, `0`/`.`/`_` = empty. Output: formatted grid + line string.

## Agent Guidelines

### If modifying solving logic

- All solving lives in `rustoku-lib/src/core/`. Changes must preserve bitmask invariants across `Masks` and `Candidates`.
- If adding a new technique, add a variant to `TechniqueFlags` (uses `bitflags`) and integrate it into the constraint propagation loop.
- Run `cargo test --workspace` and `cargo bench` to verify correctness and performance.

### If modifying the CLI

- All CLI code is in `rustoku-cli/src/`. Commands use `clap` derive macros.
- If adding a new subcommand, follow the existing pattern: parse args → construct `Rustoku` → call method → display result.
- Error handling: propagate `RustokuError` and display user-friendly messages.

### If modifying error handling

- All errors are in `RustokuError` enum (`rustoku-lib/src/error.rs`) using `thiserror`.
- If adding a new error variant, add it to the enum and handle it in both lib and CLI.

### If modifying display/formatting

- Display logic lives in `rustoku-lib/src/format.rs`.
- Board display uses a grid format. Solutions include the solve path when verbose.

### Build & quality checks

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo bench                    # criterion benchmarks in rustoku-lib/benches/
```

### Performance expectations

- `solve_any`: ~10–50μs per typical puzzle
- `solve_all`: ~20–100μs depending on complexity
- `Rustoku` struct: ~1KB total (board 81B + masks 108B + candidates 324B)
- If a change regresses benchmarks significantly, investigate before merging.

### Dependencies

- `rustoku-lib`: `bitflags`, `rand`, `thiserror`
- `rustoku-cli`: `rustoku-lib`, `clap`
- Keep dependencies minimal. If adding a new dep, justify it.

### Versioning

- Root `Cargo.toml` defines shared workspace version. See `CONTRIBUTING.md` for publishing steps.
