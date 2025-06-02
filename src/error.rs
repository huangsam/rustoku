//! Error module for Rustoku.

use thiserror::Error;

/// Represents the types of errors that can occur while working with Sudoku puzzles.
#[derive(Debug, Error)]
pub enum RustokuError {
    /// The number of clues provided for puzzle generation is not between 17 and 81.
    #[error("Clues must be between 17 and 81 for a valid Sudoku puzzle")]
    InvalidClueCount,
    /// The input string does not contain exactly 81 characters.
    #[error("Input string must be exactly 81 characters long")]
    InvalidInputLength,
    /// The input string contains characters other than digits `0-9` or `.` or `_`.
    #[error("Input string must contain only digits '0'-'9'")]
    InvalidInputCharacter,
    /// The initial board contains duplicate values in rows, columns, or 3x3 boxes.
    #[error("Initial board contains duplicates")]
    DuplicateValues,
    /// The puzzle generation process failed.
    #[error("Puzzle generation failed ")]
    GenerateFailure,
}
