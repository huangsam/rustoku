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
pub enum SolveStep {
    /// A placement of a single value on the Sudoku board.
    Placement {
        /// The row where the value is placed.
        row: usize,
        /// The column where the value is placed.
        col: usize,
        /// The value being placed in the Sudoku board.
        value: u8,
        /// Flags indicating the technique used for this placement.
        flags: TechniqueFlags,
    },
    /// A removal of a candidate value from the Sudoku board.
    CandidateElimination {
        /// The row where the candidate is eliminated.
        row: usize,
        /// The column where the candidate is eliminated.
        col: usize,
        /// The value being eliminated as a candidate.
        value: u8,
        /// Flags indicating the technique used for this elimination.
        flags: TechniqueFlags,
    },
}

impl SolveStep {
    /// 4-letter code for the solve step
    pub fn code(&self) -> &str {
        match self {
            Self::CandidateElimination { .. } => "elim",
            Self::Placement { .. } => "plac",
        }
    }
}
