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
# This compiles the binary and places it at ./target/release/rustoku
cargo build --release

# This compiles the binary and places it at $HOME/.cargo/bin
cargo install --path .

# Generate puzzles
rustoku generate
rustoku generate --clues 25
rustoku generate -c 40

# Solve any puzzle
rustoku solve '53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79'
rustoku solve '749625813653178942128394567936417285584263179271859436392781654867542391415936...'

# Solve empty puzzle
rustoku solve '.78..26.9.3...8.2...2....83.......4..43.9......73...9.2....1.36..184.9.2.5...3..7'

# Solve multiple puzzles
rustoku solve '...............942128394567936417285584263179271859436392781654867...............' --all
rustoku solve '2957438614318659..8761925433874592166123874955492167387635.41899286713541549386..' --all
rustoku solve '295743..14318659..8761925433874592166123874955492167387635.......................' --all

# Check solutions
rustoku check '295743861431865927876192543387459216612387495549216738763524189154938672928671354'

# Display puzzle
rustoku show '9..5.74....7.8....83.4.1..64.2...3...9.....65....5..8.2..9.8....8..74...7..21.8.3'
rustoku show '921567438647389512835421796452896371198743265376152984214938657583674129769215843'

# Version
rustoku -V
```

That's it! Dive into the world of high-performance Sudoku.

[bitmasking]: https://www.geeksforgeeks.org/what-is-bitmasking/
[MRV]: https://www.alooba.com/skills/concepts/data-science-6/minimum-remaining-values/
