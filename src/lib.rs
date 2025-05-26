//! A Sudoku puzzle solver implemented in Rust with efficient bitmasking for constraint tracking.
//!
//! This module provides a `Rustoku` struct that can solve 9x9 Sudoku puzzles using a
//! backtracking algorithm. The implementation uses bitmasks to efficiently track constraints
//! for rows, columns, and 3x3 boxes, enabling fast validation and candidate computation
//! during the solving process.
//!
//! # Features
//! - Supports initialization from a 2D array, a flat byte array, or a string representation
//! - Provides methods to solve for one solution, multiple solutions, all solutions
//! - Provides method to generate a random Sudoku puzzle
//! - Includes a utility function for printing the board

mod rustoku;

pub use rustoku::{Rustoku, RustokuError, print_board};
