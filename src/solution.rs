//! Solution module for Rustoku.

use crate::format::{format_grid, format_line, format_solve_path};
use std::fmt; // Assuming `format` is a sibling module

/// Represents a solved Sudoku board and the solution path.
///
/// Most of the time, users just want to see the solved board, but this struct also
/// provides the sequence of moves that led to the solution, which can be useful for debugging
/// or understanding the solving process.
#[derive(Debug, Clone)]
pub struct RustokuSolution {
    /// The solved Sudoku board, represented as a 2D array
    pub board: [[u8; 9]; 9],
    /// The sequence of moves (row, col, value) made to reach the solution
    pub solve_path: Vec<(usize, usize, u8)>,
}

/// Formats the board and solve path into a human-readable string representation.
///
/// First we format the board into a grid representation and line format.
/// Then we format the solve path into a string representation of moves.
impl fmt::Display for RustokuSolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", format_grid(&self.board).join("\n"))?;
        writeln!(f, "Line format: {}", format_line(&self.board))?;
        writeln!(
            f,
            "Solve path:\n{}",
            format_solve_path(&self.solve_path).join("\n")
        )?;
        Ok(())
    }
}
