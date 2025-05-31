//! Core module for the Rustoku solver and generator.
//!
//! This module includes the `Rustoku` struct for representing and solving Sudoku puzzles using
//! backtracking and Minimum Remaining Values (MRV). It provides functionality for solving
//! puzzles and checking solutions.
//!
//! This module also includes a function to generate new Sudoku puzzles with a specified
//! number of clues, ensuring that the generated puzzle has a unique solution.

mod board;
mod client;

pub use self::board::{Rustoku, SolverTechniques};
use crate::error::RustokuError;
use rand::rng;
use rand::seq::SliceRandom;

/// Generates a new Sudoku puzzle with a unique solution.
///
/// The `num_clues` parameter specifies the desired number of initially
/// filled cells (clues) in the generated puzzle. Fewer clues generally
/// result in a harder puzzle. The actual number of clues may be slightly
/// more than `num_clues` if it's impossible to remove more numbers
/// while maintaining a unique solution.
///
/// # Example
///
/// Generate a puzzle with 30 clues:
/// ```
/// use rustoku::core::generate_puzzle;
/// let puzzle = generate_puzzle(30).unwrap();
/// assert_eq!(puzzle.len(), 9);
/// assert_eq!(puzzle[0].len(), 9);
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RustokuError;
    use crate::format::format_line;

    const UNIQUE_PUZZLE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    const UNIQUE_SOLUTION: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
    const TWO_PUZZLE: &str =
        "2957438614318659..8761925433874592166123874955492167387635.41899286713541549386..";

    #[test]
    fn test_new_with_bytes_and_str() {
        let board = [
            [5, 3, 0, 0, 7, 0, 0, 0, 0],
            [6, 0, 0, 1, 9, 5, 0, 0, 0],
            [0, 9, 8, 0, 0, 0, 0, 6, 0],
            [8, 0, 0, 0, 6, 0, 0, 0, 3],
            [4, 0, 0, 8, 0, 3, 0, 0, 1],
            [7, 0, 0, 0, 2, 0, 0, 0, 6],
            [0, 6, 0, 0, 0, 0, 2, 8, 0],
            [0, 0, 0, 4, 1, 9, 0, 0, 5],
            [0, 0, 0, 0, 8, 0, 0, 7, 9],
        ];

        let flat_bytes: [u8; 81] = board.concat().try_into().unwrap();
        let board_str: String = flat_bytes.iter().map(|&b| (b + b'0') as char).collect();

        let solver_from_new = Rustoku::new(board).unwrap();
        let solver_from_bytes = Rustoku::try_from(flat_bytes).unwrap();
        let solver_from_str = Rustoku::try_from(board_str.as_str()).unwrap();

        assert_eq!(solver_from_new.board, solver_from_bytes.board);
        assert_eq!(solver_from_new.board, solver_from_str.board);
        assert_eq!(solver_from_bytes.board, solver_from_str.board);
    }

    #[test]
    fn test_try_from_with_valid_input() {
        let rustoku = Rustoku::try_from(UNIQUE_PUZZLE);
        assert!(rustoku.is_ok());
    }

    #[test]
    fn test_try_from_with_invalid_length() {
        let s = "53..7...."; // Too short
        let rustoku = Rustoku::try_from(s);
        assert!(matches!(rustoku, Err(RustokuError::InvalidInputLength)));
    }

    #[test]
    fn test_try_from_with_invalid_character() {
        let s = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..7X"; // 'X'
        let rustoku = Rustoku::try_from(s);
        assert!(matches!(rustoku, Err(RustokuError::InvalidInputCharacter)));
    }

    #[test]
    fn test_try_from_with_duplicate_initial_values() {
        let s = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..55...8..79";
        let rustoku = Rustoku::try_from(s);
        assert!(matches!(rustoku, Err(RustokuError::DuplicateValues)));
    }

    #[test]
    fn test_solve_any_with_solvable_sudoku() {
        let s = UNIQUE_PUZZLE;
        let mut rustoku = Rustoku::try_from(s).unwrap();
        let solution = rustoku.solve_any().unwrap();

        assert_eq!(
            UNIQUE_SOLUTION,
            format_line(&solution.board),
            "Solution does not match the expected result"
        );
    }

    #[test]
    fn test_solve_until_with_bound() {
        let s = UNIQUE_PUZZLE;
        let mut rustoku = Rustoku::try_from(s).unwrap();

        // Test with bound = 1 (find only one solution)
        let solutions = rustoku.solve_until(1);
        assert_eq!(
            1,
            solutions.len(),
            "Expected exactly one solution with bound = 1"
        );

        // Test with bound = 0 (find all solutions)
        let all_solutions = rustoku.solve_until(0);
        assert_eq!(
            1,
            all_solutions.len(),
            "Expected exactly one solution for this board with bound = 0"
        );

        // Ensure the solution found with bound = 1 matches the solution found with bound = 0
        assert_eq!(
            solutions[0].board, all_solutions[0].board,
            "Solution with bound = 1 does not match the solution with bound = 0"
        );
    }

    #[test]
    fn test_solve_all_with_unique_solution() {
        let s = UNIQUE_PUZZLE;
        let mut rustoku = Rustoku::try_from(s).unwrap();
        let solutions = rustoku.solve_all();
        assert_eq!(
            1,
            solutions.len(),
            "Expected a unique solution for the board"
        );
    }

    #[test]
    fn test_solve_all_with_multiple_solutions() {
        let s = TWO_PUZZLE;
        let mut rustoku = Rustoku::try_from(s).unwrap();
        let solutions = rustoku.solve_all();
        assert_eq!(
            2,
            solutions.len(),
            "Expected two solutions for the given board"
        );
    }

    #[test]
    fn test_solve_any_with_all_techniques() {
        let s = UNIQUE_PUZZLE;
        let mut rustoku = Rustoku::try_from(s).unwrap();
        rustoku = rustoku.with_techniques(SolverTechniques::ALL);
        let solution = rustoku.solve_any().unwrap();

        assert_eq!(
            UNIQUE_SOLUTION,
            format_line(&solution.board),
            "Solution does not match the expected result with all techniques"
        );
    }

    #[test]
    fn test_solve_all_with_all_techniques() {
        let s = TWO_PUZZLE;
        let mut rustoku = Rustoku::try_from(s).unwrap();
        rustoku = rustoku.with_techniques(SolverTechniques::ALL);
        let solutions = rustoku.solve_all();

        assert_eq!(
            2,
            solutions.len(),
            "Expected two solutions for the given board with all techniques"
        );
    }

    #[test]
    fn test_generate_with_enough_clues() {
        (20..=80).step_by(20).for_each(|num_clues| {
            let puzzle = generate_puzzle(num_clues).unwrap();
            let mut rustoku = Rustoku::new(puzzle).unwrap();

            // Ensure the puzzle has at least the specified number of clues
            let clues_count = puzzle.iter().flatten().filter(|&&cell| cell != 0).count();
            assert!(
                clues_count >= num_clues,
                "Expected at least {} clues, but found {} clues",
                num_clues,
                clues_count
            );

            // Ensure the puzzle has a unique solution
            let solutions = rustoku.solve_all();
            assert_eq!(
                1,
                solutions.len(),
                "Generated puzzle with {} clues should have a unique solution",
                num_clues
            );
        })
    }

    #[test]
    fn test_generate_with_too_few_clues() {
        let num_clues = 16; // Below the minimum valid clue count
        let result = generate_puzzle(num_clues);
        assert!(matches!(result, Err(RustokuError::InvalidClueCount)));
    }

    #[test]
    fn test_generate_with_too_many_clues() {
        let num_clues = 82; // Above the maximum valid clue count
        let result = generate_puzzle(num_clues);
        assert!(matches!(result, Err(RustokuError::InvalidClueCount)));
    }

    #[test]
    fn test_is_solved_with_valid_solution() {
        let s = UNIQUE_SOLUTION;
        let rustoku = Rustoku::try_from(s).unwrap();
        assert!(rustoku.is_solved(), "The Sudoku puzzle should be solved");
    }

    #[test]
    fn test_is_solved_with_unsolved_board() {
        let s = UNIQUE_PUZZLE;
        let rustoku = Rustoku::try_from(s).unwrap();
        assert!(!rustoku.is_solved(), "The board should not be valid");
    }
}
