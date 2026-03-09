# Library User Guide (`rustoku-lib`)

`rustoku-lib` is the core engine of the Rustoku project, designed for maximum performance and ease of use in other Rust projects.

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
rustoku-lib = "0.14.0"
```

## Core Concepts

### `Rustoku` Struct
The primary interface for solving. It maintains the board state and optimized bitmasks for constraint propagation.

### `Board` Struct
A simple wrapper around a `[[u8; 9]; 9]` array, providing convenient display and conversion methods.

## Basic Usage

### Solving a Puzzle
```rust
use rustoku_lib::Rustoku;

fn main() {
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut solver = Rustoku::new_from_str(puzzle).unwrap();
    
    if let Some(solution) = solver.solve_any() {
        println!("Solved board:\n{}", solution.board);
    }
}
```

### Generating a Puzzle
```rust
use rustoku_lib::{generate_board_by_difficulty, Difficulty};

fn main() {
    let board = generate_board_by_difficulty(Difficulty::Hard, 100).unwrap();
    println!("New puzzle:\n{}", board);
}
```

## Advanced Usage

### Enabling Specific Techniques
You can configure which "human" techniques the solver uses via `TechniqueFlags`.

```rust
use rustoku_lib::core::{Rustoku, TechniqueFlags};

let mut solver = Rustoku::new_from_str(puzzle).unwrap();
solver.with_techniques(TechniqueFlags::EASY | TechniqueFlags::X_WING);

let solution = solver.solve_any();
```

### Serializing with Serde
If the `serde` feature is enabled (at your choice, if implemented), core structs support JSON/YAML serialization.

## Error Handling
The library uses a custom `RustokuError` enum for granular error reporting (invalid board length, duplicate clues, etc.).
