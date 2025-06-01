# Rustoku

**Lightning-fast Sudoku solving and generation, crafted in Rust.**

Rustoku is a highly optimized Sudoku puzzle solver and generator built with a focus on speed and efficiency.
It leverages [bitmasking] for constraint tracking and a backtracking algorithm with [MRV] for
puzzle navigation.

- **Blazing-fast solving:** Find solutions in microseconds, even for complex puzzles
- **Unique puzzle generation:** Create new, solvable Sudoku puzzles guaranteed to have only one solution
- **Configurable difficulty:** Generate puzzles with a specified number of initial clues
- **Intuitive command-line interface (CLI):** Generate and solve puzzles directly from your terminal

## Getting Started

```bash
# This compiles the binary and places it at $HOME/.cargo/bin
cargo install --path .

# Generate puzzles
rustoku generate

# Solve puzzles
rustoku solve '53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79'
rustoku solve '...............942128394567936417285584263179271859436392781654867...............' --all

# Check solutions
rustoku check '295743861431865927876192543387459216612387495549216738763524189154938672928671354'

# Display puzzle
rustoku show '9..5.74....7.8....83.4.1..64.2...3...9.....65....5..8.2..9.8....8..74...7..21.8.3'

# Version
rustoku -V
```

That's it! Dive into the world of high-performance Sudoku.

[bitmasking]: https://www.geeksforgeeks.org/what-is-bitmasking/
[MRV]: https://www.alooba.com/skills/concepts/data-science-6/minimum-remaining-values/
