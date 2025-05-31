use crate::error::RustokuError;
use rand::rng;
use rand::seq::SliceRandom;

use super::Rustoku;

/// Generates a new Sudoku puzzle with a unique solution.
///
/// The `num_clues` parameter specifies the desired number of initially
/// filled cells (clues) in the generated puzzle. Fewer clues generally
/// result in a harder puzzle. The actual number of clues may be slightly
/// more than `num_clues` if it's impossible to remove more numbers
/// while maintaining a unique solution.
pub fn generate_puzzle(num_clues: usize) -> Result<[[u8; 9]; 9], RustokuError> {
    if !(17..=81).contains(&num_clues) {
        return Err(RustokuError::InvalidClueCount);
    }

    // Start with a fully solved board
    let mut board = Rustoku::new([[0; 9]; 9])?
        .solve_any()
        .ok_or(RustokuError::DuplicateValues)?
        .board;

    // Shuffle all cell coordinates
    let mut cells: Vec<(usize, usize)> = (0..9).flat_map(|r| (0..9).map(move |c| (r, c))).collect();
    cells.shuffle(&mut rng());

    let mut clues = 81;

    // Remove numbers while maintaining a unique solution
    for &(r, c) in &cells {
        if clues <= num_clues {
            break;
        }

        let original = board[r][c];
        board[r][c] = 0;

        if Rustoku::new(board)?.solve_until(2).len() != 1 {
            board[r][c] = original; // Restore if not unique
        } else {
            clues -= 1;
        }
    }

    // Check if the generated puzzle has a unique solution
    if Rustoku::new(board)?.solve_until(2).len() != 1 {
        // If not unique, return an error
        return Err(RustokuError::MissingUniqueSolution);
    }

    Ok(board)
}
