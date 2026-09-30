//! Error types and handling for Rustoku.
//!
//! All fallible operations return [`RustokuError`], categorized by operational phase:
//!
//! | Phase | Variants | Description |
//! | :--- | :--- | :--- |
//! | **Parsing** | [`InvalidInputLength`](RustokuError::InvalidInputLength), [`InvalidInputCharacter`](RustokuError::InvalidInputCharacter) | Malformed 81-character puzzle string |
//! | **Validation** | [`DuplicateValues`](RustokuError::DuplicateValues) | Initial board contains duplicate clues |
//! | **Generation** | [`InvalidClueCount`](RustokuError::InvalidClueCount), [`GenerateFailure`](RustokuError::GenerateFailure) | Out-of-bounds clues (<17 or >81) or timeout |
//! | **Config** | [`UnknownDifficulty`](RustokuError::UnknownDifficulty) | Unrecognized difficulty string |
//!
//! # Example
//!
//! ```rust
//! use rustoku_lib::core::Board;
//! use rustoku_lib::RustokuError;
//!
//! assert_eq!(Board::try_from("too short"), Err(RustokuError::InvalidInputLength));
//! ```

use std::fmt;

/// Represents the types of errors that can occur while working with Sudoku puzzles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustokuError {
    /// The number of clues provided for puzzle generation is not between 17 and 81.
    InvalidClueCount,
    /// The input string does not contain exactly 81 characters.
    InvalidInputLength,
    /// The input string contains characters other than digits `0-9` or `.` or `_`.
    InvalidInputCharacter,
    /// The initial board contains duplicate values in rows, columns, or 3x3 boxes.
    DuplicateValues,
    /// The puzzle generation process failed.
    GenerateFailure,
    /// The difficulty string is not one of: easy, medium, hard, expert.
    UnknownDifficulty(String),
}

impl fmt::Display for RustokuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidClueCount => {
                write!(
                    f,
                    "Clues must be between 17 and 81 for a valid Sudoku puzzle"
                )
            }
            Self::InvalidInputLength => {
                write!(f, "Input string must be exactly 81 characters long")
            }
            Self::InvalidInputCharacter => {
                write!(f, "Input string must contain only digits '0'-'9'")
            }
            Self::DuplicateValues => write!(f, "Initial board contains duplicates"),
            Self::GenerateFailure => write!(f, "Puzzle generation failed "),
            Self::UnknownDifficulty(diff) => {
                write!(
                    f,
                    "Unknown difficulty {diff:?}; expected one of: easy, medium, hard, expert"
                )
            }
        }
    }
}

impl std::error::Error for RustokuError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            RustokuError::InvalidClueCount.to_string(),
            "Clues must be between 17 and 81 for a valid Sudoku puzzle"
        );
        assert_eq!(
            RustokuError::InvalidInputLength.to_string(),
            "Input string must be exactly 81 characters long"
        );
        assert_eq!(
            RustokuError::InvalidInputCharacter.to_string(),
            "Input string must contain only digits '0'-'9'"
        );
        assert_eq!(
            RustokuError::DuplicateValues.to_string(),
            "Initial board contains duplicates"
        );
        assert_eq!(
            RustokuError::GenerateFailure.to_string(),
            "Puzzle generation failed "
        );
        assert_eq!(
            RustokuError::UnknownDifficulty("nightmare".to_string()).to_string(),
            "Unknown difficulty \"nightmare\"; expected one of: easy, medium, hard, expert"
        );
    }
}
