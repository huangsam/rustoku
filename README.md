# Rustoku

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/huangsam/rustoku/ci.yml)](https://github.com/huangsam/rustoku/actions)
[![docs.rs](https://img.shields.io/docsrs/rustoku-lib)](https://docs.rs/crate/rustoku-lib/latest)
[![License](https://img.shields.io/github/license/huangsam/rustoku)](https://github.com/huangsam/rustoku/blob/main/LICENSE)

**Lightning-fast Sudoku solving and generation, crafted in Rust.**

Rustoku is a highly optimized Sudoku puzzle solver and generator built with a focus on speed and clarity.
It leverages [bitmasking] for constraint tracking and a backtracking algorithm with [MRV] for
puzzle navigation. Available as a Rust library and command-line tool.

- **Fast solving with clear solve paths:** Rustoku can find solutions in microseconds, even
for the most complex puzzles. Plus, it shows you the step-by-step solve path for understanding
how the solution was reached
- **Unique and configurable puzzle generation:** Create Sudoku puzzles that are guaranteed to
have only one solution. You can also customize the difficulty by generating puzzles with a
specific number of clues
- **Human-like puzzle solving:** Take control with Rustoku's CLI interface. Beyond raw speed, it
mimics human expertise, employing a full range of techniques from [Naked Singles] to [X-Wing]
to solve puzzles with precision

## Getting Started

Commands to get you going:

```bash
# This installs the binary at $HOME/.cargo/bin
cargo install rustoku-cli

# Generate puzzles
rustoku-cli generate
rustoku-cli generate --clues 45

# Solve puzzles
rustoku-cli solve any '53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79'
rustoku-cli solve all '295743..14318659..8761925433874592166123874955492167387635.......................'

# Check solutions
rustoku-cli check '295743861431865927876192543387459216612387495549216738763524189154938672928671354'

# Display puzzle
rustoku-cli show '9..5.74....7.8....83.4.1..64.2...3...9.....65....5..8.2..9.8....8..74...7..21.8.3'
```

Example output from the `show` command:

```
+-------+-------+-------+
| 9 . . | 5 . 7 | 4 . . |
| . . 7 | . 8 . | . . . |
| 8 3 . | 4 . 1 | . . 6 |
+-------+-------+-------+
| 4 . 2 | . . . | 3 . . |
| . 9 . | . . . | . 6 5 |
| . . . | . 5 . | . 8 . |
+-------+-------+-------+
| 2 . . | 9 . 8 | . . . |
| . 8 . | . 7 4 | . . . |
| 7 . . | 2 1 . | 8 . 3 |
+-------+-------+-------+
Line format: 9..5.74....7.8....83.4.1..64.2...3...9.....65....5..8.2..9.8....8..74...7..21.8.3
```

Dive into the world of high-performance Sudoku today!

[bitmasking]: https://www.geeksforgeeks.org/what-is-bitmasking/
[MRV]: https://www.alooba.com/skills/concepts/data-science-6/minimum-remaining-values/
[Naked Singles]: https://hodoku.sourceforge.net/en/tech_singles.php#n1
[X-Wing]: https://hodoku.sourceforge.net/en/tech_fishb.php#bf2
