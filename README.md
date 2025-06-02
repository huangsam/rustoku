# Rustoku

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/huangsam/rustoku/ci.yml)](https://github.com/huangsam/rustoku/actions)
[![License](https://img.shields.io/github/license/huangsam/rustoku)](https://github.com/huangsam/rustoku/blob/main/LICENSE)

**Lightning-fast Sudoku solving and generation, crafted in Rust.**

Rustoku is a highly optimized Sudoku puzzle solver and generator built with a focus on speed and efficiency.
It leverages [bitmasking] for constraint tracking and a backtracking algorithm with [MRV] for
puzzle navigation.

- **Blazing-fast solving:** Find solutions in microseconds, even for complex puzzles
- **Unique puzzle generation:** Create Sudoku puzzles guaranteed to have only one solution
- **Configurable difficulty:** Generate puzzles with a specific number of clues
- **Intuitive CLI:** Generate and solve puzzles directly from your terminal

## Getting Started

```bash
# This compiles the binary and places it at $HOME/.cargo/bin
cargo install --path .

# Generate puzzles
rustoku generate
rustoku generate --clues 45

# Solve puzzles
rustoku solve any '53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79'
rustoku solve all '295743..14318659..8761925433874592166123874955492167387635.......................'
rustoku solve human '8..41.....1....35...47.3......3.7..24...9.6....9..4....6.941.2.1.287...695......8'

# Check solutions
rustoku check '295743861431865927876192543387459216612387495549216738763524189154938672928671354'

# Display puzzle
rustoku show '9..5.74....7.8....83.4.1..64.2...3...9.....65....5..8.2..9.8....8..74...7..21.8.3'
```

That's it! Dive into the world of high-performance Sudoku.

[bitmasking]: https://www.geeksforgeeks.org/what-is-bitmasking/
[MRV]: https://www.alooba.com/skills/concepts/data-science-6/minimum-remaining-values/
