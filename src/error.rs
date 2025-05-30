//! Error module for Rustoku.

use thiserror::Error;

/// Represents the types of errors that can occur while working with Sudoku puzzles.
///
/// This enum defines various error cases that can occur while working with Sudoku puzzles:
/// - The number of clues provided for puzzle generation is not between 17 and 81
/// - The input string does not contain exactly 81 characters
/// - The input string contains characters other than digits `0-9` or `.` or `_`
/// - The initial board contains duplicate values in rows, columns, or 3x3 boxes
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
}
