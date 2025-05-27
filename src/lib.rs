//! A Sudoku library implemented in Rust with efficient bitmasking for constraint tracking.
//!
//! This library provides a `Rustoku` struct that can solve and generate 9x9 Sudoku puzzles.
//! The implementation uses bitmasks to efficiently track constraints for
//! rows, columns, and 3x3 boxes, enabling fast validation and candidate computation
//! during the solving process.
//!
//! The library also provides a `print_board` utility to print the Sudoku board in a
//! human-readable format. The output includes the matrix-like representation of the
//! board as well as the one-line representation for easy copying and pasting.

pub mod core;
pub mod format;
