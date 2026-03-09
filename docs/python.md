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

### `solve_all(puzzle: str) -> list[str]`
Finds every solution for a puzzle.
- **Output**: List of 81-character solved strings (empty list if unsolvable).
- **Raises**: `ValueError` if the input string is malformed.

### `solve_steps(puzzle: str, difficulty: str = "expert") -> dict | None`
Solves a puzzle and returns a full step-by-step trace.
- **Output**: `{"board": str, "steps": list[dict]}` or `None` if unsolvable.
  Each step dict contains: `type` (`"placement"` or `"elimination"`), `row`, `col`,
  `value`, `technique`, `step_number`, `candidates_eliminated`, `related_cell_count`,
  `difficulty_point`.
- **`difficulty`**: Controls which human techniques are applied — `"easy"`, `"medium"`,
  `"hard"`, or `"expert"`. Higher levels produce richer technique annotations.
- **Raises**: `ValueError` if the input or difficulty is invalid.

### `candidates(puzzle: str) -> list[list[list[int]]]`
Returns the valid candidate digits for every cell.
- **Output**: 9×9 list of lists. Each inner list contains the digits (1–9) still
  possible for that cell; filled cells return `[]`.
- **Raises**: `ValueError` if the input string is malformed.

### `generate_clues(n: int) -> str`
Generates a puzzle with exactly `n` given clues (17–81).
- **Output**: 81-character puzzle string.
- **Raises**: `ValueError` if `n` is out of range or generation fails.

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
print(f"New Puzzle:  {puzzle}")

# 2. Solve it
solution = rustoku.solve(puzzle)
if solution:
    print(f"Solved:      {solution}")

# 3. Check a board
if rustoku.check(solution):
    print("The solution is valid!")

# 4. Find all solutions (useful for checking uniqueness)
all_solutions = rustoku.solve_all(puzzle)
print(f"Solutions found: {len(all_solutions)}")

# 5. Step-by-step trace
result = rustoku.solve_steps(puzzle, difficulty="hard")
if result:
    for step in result["steps"][:3]:
        print(f"  R{step['row']}C{step['col']} = {step['value']} via {step['technique']}")

# 6. Pencil-mark candidates
grid = rustoku.candidates(puzzle)
print(f"Candidates at R0C2: {grid[0][2]}")

# 7. Generate by clue count
sparse = rustoku.generate_clues(22)
print(f"22-clue puzzle: {sparse}")
```

## Performance
The Python module drops the GIL (Global Interpreter Lock) during heavy computation, allowing for true multi-core scale when combined with Python's `threading` or `multiprocessing` for batch processing.
