use super::board::Board;

/// Solved board and its solution path.
///
/// Most of the time, users just want to see the solved board, but this struct also
/// provides the sequence of moves that led to the solution, which can be useful for debugging
/// or understanding the solving process.
#[derive(Debug, Clone)]
pub struct Solution {
    /// The solved Sudoku board, represented as a 2D array
    pub board: Board,
    /// The sequence of moves (row, col, value) made to reach the solution
    pub solve_path: Vec<(usize, usize, u8)>,
}
