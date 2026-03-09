# Python Bindings User Guide (`rustoku-py`)

The `rustoku` Python module provides high-performance Sudoku solving and generation, powered by the core Rust engine.

## Installation

Currently, Python bindings are built from source using `maturin`.

### Prerequisites
- Python 3.8+
- Rust toolchain
- `maturin` (install via `pip install maturin` or `brew install maturin`)

### Building from Source
```bash
cd rustoku-py
# Recommended: use a virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Build and install the module into your environment
maturin develop
```

## API Reference

### `solve(puzzle: str) -> str`
Solves a Sudoku puzzle.
- **Input**: 81-character string (empty cells: `0`, `.`, or `_`).
- **Output**: 81-character solved string, or an empty string if unsolvable.
- **Raises**: `ValueError` if the input string is malformed.

### `generate(difficulty: str) -> str`
Generates a new Sudoku puzzle with a unique solution.
- **Input**: Difficulty level (`"easy"`, `"medium"`, `"hard"`, or `"expert"`).
- **Output**: 81-character puzzle string.
- **Raises**: `ValueError` if the difficulty is invalid or generation fails.

### `check(board: str) -> bool`
Validates if a board is a completed, valid Sudoku solution.
- **Input**: 81-character string.
- **Output**: `True` if valid and complete, `False` otherwise.
- **Raises**: `ValueError` if the input string is malformed.

## Usage Example

```python
import rustoku

# 1. Generate a puzzle
puzzle = rustoku.generate("medium")
print(f"New Puzzle: {puzzle}")

# 2. Solve the puzzle
solution = rustoku.solve(puzzle)
if solution:
    print(f"Solved:     {solution}")

# 3. Check a board
if rustoku.check(solution):
    print("The solution is valid!")
```

## Performance
The Python module drops the GIL (Global Interpreter Lock) during heavy computation, allowing for true multi-core scale when combined with Python's `threading` or `multiprocessing` for batch processing.
