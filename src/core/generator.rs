use crate::error::RustokuError;
use rand::rng;
use rand::seq::SliceRandom;

use super::Rustoku;

/// Generation methods for Sudoku puzzles.
impl Rustoku {
    /// Generates a new Sudoku puzzle with a unique solution.
    ///
    /// The `num_clues` parameter specifies the desired number of initially
    /// filled cells (clues) in the generated puzzle. Fewer clues generally
    /// result in a harder puzzle. The actual number of clues may be slightly
    /// more than `num_clues` if it's impossible to remove more numbers
    /// while maintaining a unique solution.
    pub fn generate(num_clues: usize) -> Result<[[u8; 9]; 9], RustokuError> {
        if !(17..=81).contains(&num_clues) {
            return Err(RustokuError::InvalidClueCount);
        }

        // Start with a fully solved board
        let mut rustoku = Rustoku::new([[0; 9]; 9])?;
        let mut board = rustoku
            .solve_any()
            .ok_or(RustokuError::DuplicateValues)?
            .board;

        // Shuffle all cell coordinates
        let mut cells: [(usize, usize); 81] = {
            let mut arr = [(0, 0); 81];
            let mut idx = 0;
            for r in 0..9 {
                for c in 0..9 {
                    arr[idx] = (r, c);
                    idx += 1;
                }
            }
            arr
        };
        cells.shuffle(&mut rng());

        let mut clues = 81;

        // Remove numbers while maintaining a unique solution
        for &(r, c) in &cells {
            if clues <= num_clues {
                break;
            }

            let original = board[r][c];
            board[r][c] = 0;

            let mut temp_solver =
                Rustoku::new(board).expect("Board state should be valid after removal");

            if temp_solver.solve_until(2).len() != 1 {
                board[r][c] = original; // Restore if not unique
            } else {
                clues -= 1;
            }
        }

        Ok(board)
    }
}
