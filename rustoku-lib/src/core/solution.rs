use crate::core::TechniqueFlags;

use super::board::Board;

/// Solved board and its solution path.
///
/// Most of the time, users just want to see the solved board, but this struct also
/// provides the sequence of moves that led to the solution, which can be useful for debugging
/// or understanding the solving process.
#[derive(Debug, Clone)]
pub struct Solution {
    /// The solved Sudoku board, represented as a 2D array.
    pub board: Board,
    /// The sequence of moves (row, col, value) made to reach the solution.
    pub solve_path: SolvePath,
}

/// Solve path associated with a solution.
#[derive(Debug, Clone)]
pub struct SolvePath {
    /// The sequence of steps taken to solve the Sudoku puzzle.
    pub steps: Vec<SolveStep>,
}

impl SolvePath {
    pub fn new() -> Self {
        SolvePath { steps: Vec::new() }
    }
}

impl Default for SolvePath {
    fn default() -> Self {
        Self::new()
    }
}

/// Single step in the solving process.
#[derive(Debug, Clone)]
pub struct SolveStep {
    /// The 0-based row index of the cell being modified.
    pub row: usize,
    /// The 0-based column index of the cell being modified.
    pub col: usize,
    /// The value being placed in the cell.
    pub value: u8,
    /// Flags indicating the technique used for this step.
    pub flags: TechniqueFlags,
}

impl SolveStep {
    pub fn new(row: usize, col: usize, value: u8) -> Self {
        SolveStep {
            row,
            col,
            value,
            flags: TechniqueFlags::empty(),
        }
    }

    pub fn with_approach(mut self, approach: TechniqueFlags) -> Self {
        self.flags = approach;
        self
    }
}
