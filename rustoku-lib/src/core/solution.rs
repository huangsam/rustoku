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
#[derive(Debug, Clone, Default)]
pub struct SolvePath {
    /// The sequence of steps taken to solve the Sudoku puzzle.
    pub steps: Vec<SolveStep>,
}

impl SolvePath {
    pub fn new() -> Self {
        SolvePath { steps: Vec::new() }
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
        /// Position of this step in the solve sequence (0-indexed).
        step_number: u32,
        /// Bitmask of candidate values eliminated by this placement.
        candidates_eliminated: u32,
        /// Count of related cells involved in determining this placement.
        related_cell_count: u8,
        /// Difficulty metric for this step (0-10): 0=trivial, 10=hardest.
        difficulty_point: u8,
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
        /// Position of this step in the solve sequence (0-indexed).
        step_number: u32,
        /// Bitmask of other candidate values eliminated along with this one.
        candidates_eliminated: u32,
        /// Count of related cells involved in determining this elimination.
        related_cell_count: u8,
        /// Difficulty metric for this step (0-10): 0=trivial, 10=hardest.
        difficulty_point: u8,
    },
}

impl SolveStep {
    /// 4-letter code for the solve step.
    pub fn code(&self) -> &str {
        match self {
            Self::CandidateElimination { .. } => "elim",
            Self::Placement { .. } => "plac",
        }
    }

    /// Returns the step number (position in solve sequence).
    pub fn step_number(&self) -> u32 {
        match self {
            Self::Placement { step_number, .. }
            | Self::CandidateElimination { step_number, .. } => *step_number,
        }
    }

    /// Returns the bitmask of candidates eliminated by this step.
    pub fn candidates_eliminated(&self) -> u32 {
        match self {
            Self::Placement {
                candidates_eliminated,
                ..
            }
            | Self::CandidateElimination {
                candidates_eliminated,
                ..
            } => *candidates_eliminated,
        }
    }

    /// Returns the count of related cells involved in this step.
    pub fn related_cell_count(&self) -> u8 {
        match self {
            Self::Placement {
                related_cell_count, ..
            }
            | Self::CandidateElimination {
                related_cell_count, ..
            } => *related_cell_count,
        }
    }

    /// Returns the difficulty metric for this step (0-10).
    pub fn difficulty_point(&self) -> u8 {
        match self {
            Self::Placement {
                difficulty_point, ..
            }
            | Self::CandidateElimination {
                difficulty_point, ..
            } => *difficulty_point,
        }
    }
}
