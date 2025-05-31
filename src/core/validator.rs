use super::Rustoku;

/// Validation methods for Sudoku puzzles.
impl Rustoku {
    /// Checks if the Sudoku puzzle is solved correctly.
    ///
    /// A puzzle is considered solved if all cells are filled and the board does not
    /// contain duplicates across rows, columns, and 3x3 boxes.
    pub fn is_solved(&self) -> bool {
        if self.board.iter().flatten().any(|&val| val == 0) {
            return false;
        }
        Rustoku::new(self.board).is_ok()
    }
}
