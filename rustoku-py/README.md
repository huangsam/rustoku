# rustoku

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/huangsam/rustoku/ci.yml)](https://github.com/huangsam/rustoku/actions)
[![PyPI](https://img.shields.io/pypi/v/rustoku)](https://pypi.org/project/rustoku/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/huangsam/rustoku/blob/main/LICENSE)

High-performance Sudoku solving and generation for Python, powered by a core Rust engine.

## Installation

```bash
pip install rustoku
```

## Quick Start

```python
import rustoku

# 1. Generate a puzzle
puzzle = rustoku.generate("medium")
print("Generated puzzle:", puzzle)

# 2. Solve a puzzle
solution = rustoku.solve(puzzle)
print("Solved puzzle:   ", solution)

# 3. Check solution validity
assert rustoku.check(solution)

# 4. Generate with symmetry and difficulty
symmetric = rustoku.generate_advanced(symmetry="rotational180", difficulty="hard")

# 5. Find all solutions (or check for uniqueness)
solutions = rustoku.solve_all(puzzle)
print(f"Found {len(solutions)} solution(s)")

# 6. Step-by-step human technique solve trace
trace = rustoku.solve_steps(puzzle, difficulty="expert")
print(f"Solved in {len(trace)} steps")
```

## Features

- **Blazing Fast**: Solves puzzles in microseconds using bitmask constraint tracking and MRV backtracking.
- **Human-like Techniques**: Explains steps using human solving strategies (Naked/Hidden Singles, Pairs, Triples, Quads, Pointing/Claiming, X-Wing, Swordfish, Jellyfish, Skyscraper, W-Wing, XY-Wing, XYZ-Wing, and AIC).
- **Flexible Generation**: Generate valid, uniquely solvable Sudoku boards across multiple difficulty levels (`easy`, `medium`, `hard`, `expert`) and symmetry modes (`rotational180`, `rotational90`, `mirrorvertical`, `mirrorhorizontal`, `mirrordiagonal`).

## Documentation

For full API documentation and advanced usage, see the [Python Guide](https://github.com/huangsam/rustoku/blob/main/docs/python.md) in the GitHub repository.
