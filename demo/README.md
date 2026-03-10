# Rustoku Web UI

A fully-featured web interface for Rustoku, showcasing all the WebAssembly APIs.

## Features

### Puzzle Generation
- **By Difficulty**: Generate Easy, Medium, Hard, or Expert puzzles
- **By Clue Count**: Generate puzzles with a specific number of clues (17-81)

### Solving
- **Solve Any**: Find the first solution quickly
- **Solve All**: Find all possible solutions (useful for checking puzzle uniqueness)
- **Solve with Steps**: Solve using human-like techniques (Naked Singles, Hidden Singles, Pairs, etc.) at your chosen difficulty level

### Analysis Tools
- **Show Candidates**: View all possible candidates for each empty cell, helping you understand the puzzle structure
- **Validate**: Check if the current board is a valid, complete solution

### Interactive Board
- 9×9 Sudoku grid with full mouse and keyboard support
- Clear button to reset the board
- Real-time input validation

## How to Run

```bash
# Install dependencies (first time only)
npm install

# Start dev server
npm run dev

# Build for production
npm run build
```

The WASM module will be automatically loaded when you open the app. Make sure you've built the Rust WASM package first:

```bash
# In rustoku-wasm/ directory
wasm-pack build --target web --release
```

## Using the APIs

The implementation leverages all available rustoku-wasm APIs:

- `solve(board_str)` - Quick single solution
- `solve_all(board_str)` - Find all solutions
- `solve_steps(board_str, difficulty)` - Detailed solving steps
- `candidates(board_str)` - Candidate analysis
- `generate(difficulty)` - Generate by difficulty
- `generate_clues(num_clues)` - Generate by clue count
- `check(board_str)` - Validate complete solutions

## Tech Stack

- **Vite** - Fast build tool
- **TypeScript** - Type-safe development
- **WASM** - Rust-powered solving via WebAssembly
- **CSS Grid** - Responsive Sudoku board
