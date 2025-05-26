# Rustoku

**Lightning-fast Sudoku solving and generation, crafted in Rust.**

Rustoku is a highly optimized Sudoku puzzle solver and generator built with a focus on speed and efficiency.
It leverages [bitmasking] for efficient constraint tracking and a backtracking algorithm with [MRV] for
intelligent puzzle navigation.

- **Blazing-fast solving:** Find solutions in milliseconds, even for complex puzzles
- **Unique puzzle generation:** Create new, solvable Sudoku puzzles guaranteed to have only one solution
- **Configurable difficulty:** Generate puzzles with a specified number of initial clues
- **Intuitive command-line interface (CLI):** Generate and solve puzzles directly from your terminal

## Getting Started

```bash
# This compiles the binary and places it at ./target/release/rustoku
cargo build --release

# This compiles the binary and places it at $HOME/.cargo/bin
cargo install --path .

# Generate puzzles
rustoku generate
rustoku generate --clues 25
rustoku generate -c 40

# Solve puzzles
rustoku solve '53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79'
rustoku solve '2957438614318659..8761925433874592166123874955492167387635.41899286713541549386..' --all
rustoku solve '295743..14318659..8761925433874592166123874955492167387635.......................' --all

# Check solutions
rustoku check '295743861431865927876192543387459216612387495549216738763524189154938672928671354'

# Version
rustoku -V
```

That's it! Dive into the world of high-performance Sudoku.

[bitmasking]: https://www.geeksforgeeks.org/what-is-bitmasking/
[MRV]: https://www.alooba.com/skills/concepts/data-science-6/minimum-remaining-values/
