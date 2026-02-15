# Rustoku

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/huangsam/rustoku/ci.yml)](https://github.com/huangsam/rustoku/actions)
[![docs.rs](https://img.shields.io/docsrs/rustoku-lib)](https://docs.rs/crate/rustoku-lib/latest)
[![License](https://img.shields.io/github/license/huangsam/rustoku)](https://github.com/huangsam/rustoku/blob/main/LICENSE)

**Lightning-fast Sudoku solving and generation, crafted in Rust.**

Rustoku is a highly optimized Sudoku puzzle solver and generator built with a focus on speed and clarity.
It leverages [bitmasking] for constraint tracking and a backtracking algorithm with [MRV] for
puzzle navigation. Available as a Rust [library] and [CLI].

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
rustoku-cli solve any 530070000600195000098000060800060003400803001700020006060000280000419005000080079
rustoku-cli solve all 295743001431865900876192543387459216612387495549216738763500000000000000000000000
rustoku-cli solve csv sudoku.csv

# Check solutions
rustoku-cli check 295743861431865927876192543387459216612387495549216738763524189154938672928671354

# Display puzzle
rustoku-cli show 900507400007080000830401006402000300090000065000050080200908000080074000700210803
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
Line format: 900507400007080000830401006402000300090000065000050080200908000080074000700210803
```

Try some more examples from these sites:

- [HoDoKu](https://hodoku.sourceforge.net/en/index.php)
- [heetbeet/purge-and-merge](https://github.com/heetbeet/purge-and-merge)
- [ruohanrao/sudoku](https://www.kaggle.com/datasets/rohanrao/sudoku)
- [Ritvik19/Sudoku-Dataset](https://huggingface.co/datasets/Ritvik19/Sudoku-Dataset)

Dive into the world of high-performance Sudoku today!

[bitmasking]: https://www.geeksforgeeks.org/what-is-bitmasking/
[MRV]: https://www.alooba.com/skills/concepts/data-science-6/minimum-remaining-values/
[library]: https://crates.io/crates/rustoku-lib
[CLI]: https://crates.io/crates/rustoku-cli
[Naked Singles]: https://hodoku.sourceforge.net/en/tech_singles.php#n1
[X-Wing]: https://hodoku.sourceforge.net/en/tech_fishb.php#bf2
