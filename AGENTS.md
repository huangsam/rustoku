# Agentic Documentation for Rustoku

## Project Overview

Rustoku is a high-performance Sudoku puzzle solver and generator implemented in Rust. The project is structured as a workspace with two main components: `rustoku-lib`, a core library for Sudoku logic, and `rustoku-cli`, a command-line interface that leverages the library. The project emphasizes speed, correctness, and ease of use, utilizing advanced techniques like bitmasking for constraint tracking and backtracking with Minimum Remaining Values (MRV) for solving.

Key features include:

- **Solving**: Efficient backtracking solver with MRV heuristic
- **Generation**: Creates puzzles with unique solutions and configurable clue counts
- **Validation**: Checks puzzle correctness and solution validity
- **CLI**: User-friendly command-line tools for puzzle manipulation
- **Performance**: Optimized with bitmasks and candidate caching

The project is licensed under MIT and targets Rust 1.85 with edition 2024.

## Component Descriptions

### rustoku-lib

The core library (`rustoku-lib`) provides the fundamental Sudoku solving and generation capabilities. It exports the main `Rustoku` struct and utility functions like `generate_board`.

#### Key Modules

- **`core`**: Contains the `Rustoku` solver struct, `Board` representation, constraint `Masks`, `Candidates` cache, and solving logic
- **`error`**: Defines `RustokuError` enum for various failure modes
- **`format`**: Provides display implementations for boards, solutions, and solve paths

#### Dependencies

- `bitflags`: For efficient flag management (e.g., technique flags)
- `rand`: For randomization in puzzle generation and candidate shuffling
- `thiserror`: For ergonomic error handling

#### Data Structures

- **`Board`**: 9x9 grid representation with methods for cell access and iteration
- **`Masks`**: Bitmask arrays tracking row, column, and box constraints
- **`Candidates`**: Cache of possible values for each cell using bitmasks
- **`Solution`**: Contains solved board and solve path
- **`SolvePath`**: Sequence of solve steps with technique information

#### Algorithms

- **Backtracking with MRV**: Finds empty cells with fewest candidates first
- **Constraint Propagation**: Uses techniques like naked/hidden singles and pairs
- **Puzzle Generation**: Starts with solved board, removes numbers while ensuring uniqueness

### rustoku-cli

The command-line interface (`rustoku-cli`) provides user-friendly access to the library's functionality through subcommands.

#### Commands

- **`generate`**: Creates new puzzles with specified clue counts (default: 30)
- **`solve any`**: Finds one solution with optional verbose solve path and human techniques
- **`solve all`**: Finds all solutions with optional human techniques (useful for validation)
- **`check`**: Validates if a puzzle is correctly solved
- **`show`**: Displays puzzle in formatted grid

#### Dependencies

- `rustoku-lib`: Core functionality
- `clap`: Command-line argument parsing with derive macros

#### Example Usage

```bash
# Generate a puzzle with 25 clues
rustoku-cli generate --clues 25

# Solve a puzzle and show solve path
rustoku-cli solve any --verbose "530070000600195000098000060800060003400803001700020006060000280000419005000080079"

# Solve a puzzle with all human techniques enabled
rustoku-cli solve any --human --verbose "530070000600195000098000060800060003400803001700020006060000280000419005000080079"

# Check if a puzzle is solved
rustoku-cli check "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
```

### Cargo.toml (Workspace Configuration)

The root `Cargo.toml` defines a workspace with shared metadata:

- **Version**: 0.12.3
- **Edition**: 2024
- **Rust Version**: 1.85
- **Keywords**: sudoku, puzzles, solver, generator
- **Categories**: games

Individual package `Cargo.toml` files specify component-specific dependencies and build configurations.

## API Design

### Core API (`rustoku-lib`)

The library's API is designed for simplicity and performance, with a focus on immutable operations where possible and clear error handling.

#### Main Struct

```rust
#[derive(Debug, Copy, Clone)]
pub struct Rustoku {
    pub board: Board,
    pub masks: Masks,
    pub candidates: Candidates,
    pub techniques: TechniqueFlags,
}
```

#### Key Methods


**Construction:**

- `Rustoku::new(board: Board)`: Creates solver from board
- `Rustoku::new_from_str(s: &str)`: Parses string representation
- `with_techniques(techniques: TechniqueFlags)`: Configures solving techniques

**Solving:**

- `solve_any() -> Option<Solution>`: Finds one solution
- `solve_all() -> Vec<Solution>`: Finds all solutions
- `solve_until(bound: usize) -> Vec<Solution>`: Finds up to bound solutions

**Validation:**

- `is_solved() -> bool`: Checks if current board is complete and valid

#### Utility Functions

- `generate_board(num_clues: usize) -> Result<Board, RustokuError>`: Creates puzzle with unique solution

#### Error Handling

```rust
#[derive(Debug, Error)]
pub enum RustokuError {
    #[error("Clues must be between 17 and 81 for a valid Sudoku puzzle")]
    InvalidClueCount,
    #[error("Input string must be exactly 81 characters long")]
    InvalidInputLength,
    #[error("Input string must contain only digits '0'-'9'")]
    InvalidInputCharacter,
    #[error("Initial board contains duplicates")]
    DuplicateValues,
    #[error("Puzzle generation failed ")]
    GenerateFailure,
}
```

### CLI API

The CLI uses `clap` for argument parsing, providing a hierarchical command structure with subcommands for different operations. All commands return `Result<(), RustokuError>` for consistent error handling.

## Performance Considerations

### Bitmasking Optimization

The library uses 32-bit masks for constraint tracking:

- **Rows/Columns/Boxes**: Arrays of `u32` where each bit represents a digit (1-9)
- **Candidates**: Per-cell bitmasks of possible values
- **Safety Checks**: O(1) constraint validation using bitwise operations

### MRV Heuristic

The solver prioritizes cells with fewest candidates, reducing search space significantly compared to naive backtracking.

### Candidate Caching

Maintains a cache of possible values for each cell, updated incrementally during solving to avoid recomputation.

### Benchmark Results

From criterion benchmarks:

- **solve_any_unique**: ~10-50μs for typical puzzles
- **solve_all_unique**: ~20-100μs depending on complexity
- Performance scales well with puzzle difficulty due to MRV

### Memory Usage

- `Rustoku`: ~1KB (board: 81 bytes + masks: ~300 bytes + candidates: ~300 bytes)
- `Board`: 81 bytes (9x9 u8 array)
- `Masks`: 27 u32 values (108 bytes)
- `Candidates`: 81 u32 values (324 bytes)

## CLI Interactions

### Input/Output Formats

**Input**: 81-character strings using '0'-'9', '.', or '_' for empty cells
**Output**: Formatted grid with line representation for easy copying

### Error Handling

CLI catches `RustokuError` variants and displays user-friendly messages with emojis for better UX.

### Verbose Mode

Solve commands support `--verbose` flag to show detailed solve paths, including:

- Step-by-step placements
- Techniques used at each step
- Final solution board

### Integration with Library

CLI directly instantiates `Rustoku` structs and calls methods, with minimal abstraction layer for maximum performance.

## Potential Improvements

### API Design Enhancements

1. **Async Support**: Consider tokio-based async solving for very large search spaces
2. **Serialization**: Add serde support for JSON/binary puzzle formats

### Performance Optimizations

1. **SIMD Acceleration**: Use SIMD for bulk mask operations on modern CPUs
2. **Memory Pool**: Object pooling for frequent allocations during backtracking
3. **Profile-guided Optimization**: Use PGO for better branch prediction

### Algorithm Improvements

1. **Advanced Techniques**: Implement more Sudoku techniques (Swordfish, XY-Wing, etc.)
2. **Heuristic Tuning**: Dynamic MRV with technique-based scoring
3. **Constraint Learning**: Learn from solving patterns to improve generation
4. **Approximate Solving**: Fast approximate solvers for very hard puzzles

### CLI Enhancements

1. **Interactive Mode**: TUI for step-by-step solving with user input
2. **Progress Indicators**: Enhanced progress bars for long-running puzzles
3. **Export Formats**: Support for various puzzle formats (JSON, images)

### Testing and Quality

1. **Property-based Testing**: Use proptest for comprehensive puzzle generation testing
2. **Fuzzing**: Integrate cargo-fuzz for input validation robustness
3. **Performance Regression Tests**: Automated benchmark comparisons in CI
4. **Code Coverage**: Aim for >95% coverage with additional edge case tests

### Distribution and Ecosystem

1. **WASM Support**: WebAssembly compilation for browser-based solving
2. **Python Bindings**: PyO3 bindings for Python integration
3. **Plugin System**: Extensible technique system for custom solving methods
4. **Database Integration**: Store and retrieve puzzles from databases

### Documentation and Usability

1. **Interactive Tutorial**: Built-in CLI tutorial for new users
2. **Visualization**: ASCII art solve animations
3. **Benchmark Suite**: Public benchmark results and comparisons
