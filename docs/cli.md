# CLI User Guide (`rustoku-cli`)

The `rustoku-cli` provides a powerful command-line interface for generating, solving, and checking Sudoku puzzles.

## Installation

```bash
cargo install rustoku-cli
```

## Commands

### `generate`
Create new Sudoku puzzles with a guaranteed unique solution.

- `--clues <N>`: Specify the number of clues (17-81). Default is 25.
- `--difficulty <LEVEL>`: Generate by difficulty (`easy`, `medium`, `hard`, `expert`).
- `--verbose`: Show the solve path used to verify uniqueness.

**Example:**
```bash
rustoku-cli generate --difficulty hard
```

### `solve`
Find solutions to existing puzzles.

#### `solve any <PUZZLE>`
Find the first valid solution.
- `--human`: Use human-like techniques (Naked Singles, X-Wing, etc.) before falling back to backtracking.
- `--verbose`: Print the step-by-step solve path.

#### `solve all <PUZZLE>`
Find all possible solutions (useful for checking uniqueness).
- `--until <N>`: Stop after finding *N* solutions.

#### `solve csv <FILE>`
Batch solve puzzles from a CSV file. Expects a column named `puzzle` (or the first column).

**Example:**
```bash
rustoku-cli solve any --human --verbose "530070000..."
```

### `check <BOARD>`
Validate if a board is a completed, valid Sudoku solution.

### `show <BOARD>`
Display an 81-character string in a pretty 9x9 grid format.

## Input Formats
Puzzles are accepted as 81-character strings where:
- `1`-`9` are clues.
- `0`, `.`, or `_` are empty cells.

## Technique Flags (for `--human`)
When using `--human`, the solver employs various Sudoku techniques:
- **Easy**: Naked/Hidden Singles.
- **Medium**: Naked/Hidden Pairs, Locked Candidates, Naked/Hidden Triples.
- **Hard**: X-Wing, Swordfish, Jellyfish, Skyscraper.
- **Expert**: XY-Wing, XYZ-Wing, W-Wing, AIC.
