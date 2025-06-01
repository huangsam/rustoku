//! Core module for the Rustoku solver and generator.
//!
//! This module includes the `Rustoku` struct for representing and solving Sudoku puzzles using
//! backtracking and Minimum Remaining Values (MRV). It provides functionality for solving
//! puzzles and checking solutions.
//!
//! This module also includes a function to generate new Sudoku puzzles with a specified
//! number of clues, ensuring that the generated puzzle has a unique solution.

mod board;
mod candidates;
mod entrypoint;
mod masks;
mod solution;
mod techniques;

use crate::error::RustokuError;
pub use board::RustokuBoard;
pub use entrypoint::Rustoku;
pub use solution::RustokuSolution;
pub use techniques::RustokuTechniques;

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
/// use rustoku::core::generate_board;
/// let puzzle = generate_board(30).unwrap();
/// assert_eq!(puzzle.cells.len(), 9);
/// assert_eq!(puzzle.cells[0].len(), 9);
/// ```
pub fn generate_board(num_clues: usize) -> Result<RustokuBoard, RustokuError> {
    if !(17..=81).contains(&num_clues) {
        return Err(RustokuError::InvalidClueCount);
    }

    // Start with a fully solved board
    let mut rustoku = Rustoku::new(RustokuBoard::empty())?;
    let solution = rustoku.solve_any().ok_or(RustokuError::DuplicateValues)?;
    let mut board = solution.board;

    // Shuffle all cell coordinates
    let mut cells: Vec<(usize, usize)> = board.iter_cells().collect();
    cells.shuffle(&mut rng());

    let mut clues = 81;

    // Remove numbers while maintaining a unique solution
    for &(r, c) in &cells {
        if clues <= num_clues {
            break;
        }

        let original = board.cells[r][c];
        board.cells[r][c] = 0;

        if Rustoku::new(board)?.solve_until(2).len() != 1 {
            board.cells[r][c] = original; // Restore if not unique
        } else {
            clues -= 1;
        }
    }

    // Check if the generated puzzle has a unique solution
    if Rustoku::new(board)?.solve_until(2).len() != 1 {
        // If not unique, return an error
        return Err(RustokuError::PuzzleGenerationFailed);
    }

    Ok(board)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::board::RustokuBoard;
    use crate::error::RustokuError;
    use crate::format::format_line;

    const UNIQUE_PUZZLE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    const UNIQUE_SOLUTION: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
    const TWO_PUZZLE: &str =
        "2957438614318659..8761925433874592166123874955492167387635.41899286713541549386..";
    const SIX_PUZZLE: &str =
        "295743..14318659..8761925433874592166123874955492167387635.......................";

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

        let flat_bytes: [u8; 81] = board
            .concat()
            .try_into()
            .expect("Concat board to bytes failed");
        let board_str: String = flat_bytes.iter().map(|&b| (b + b'0') as char).collect();

        let board_from_new = RustokuBoard::new(board);
        let board_from_bytes =
            RustokuBoard::try_from(flat_bytes).expect("Board from flat bytes failed");
        let board_from_str =
            RustokuBoard::try_from(board_str.as_str()).expect("Board from string failed");

        assert_eq!(board_from_new, board_from_bytes);
        assert_eq!(board_from_new, board_from_str);
        assert_eq!(board_from_bytes, board_from_str);
    }

    #[test]
    fn test_try_from_with_valid_input() {
        let rustoku = RustokuBoard::try_from(UNIQUE_PUZZLE);
        assert!(rustoku.is_ok());
    }

    #[test]
    fn test_try_from_with_invalid_length() {
        let s = "53..7...."; // Too short
        let rustoku = RustokuBoard::try_from(s);
        assert!(matches!(rustoku, Err(RustokuError::InvalidInputLength)));
    }

    #[test]
    fn test_try_from_with_invalid_character() {
        let s = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..7X"; // 'X'
        let rustoku = RustokuBoard::try_from(s);
        assert!(matches!(rustoku, Err(RustokuError::InvalidInputCharacter)));
    }

    #[test]
    fn test_try_from_with_duplicate_initial_values() {
        let s = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..55...8..79";
        let board = RustokuBoard::try_from(s).expect("Board parsing failed before duplicate check");
        let rustoku = Rustoku::new(board);
        assert!(matches!(rustoku, Err(RustokuError::DuplicateValues)));
    }

    #[test]
    fn test_solve_any_with_solvable_sudoku() {
        let s = UNIQUE_PUZZLE;
        let mut rustoku =
            Rustoku::new_from_str(s).expect("Rustoku creation failed from puzzle string");
        let solution = rustoku.solve_any().expect("Solving solvable puzzle failed");

        assert_eq!(
            UNIQUE_SOLUTION,
            format_line(&solution.board.cells),
            "Solution does not match the expected result"
        );
    }

    #[test]
    fn test_solve_any_with_unsolvable_sudoku() {
        let s = ".78..26.9.3...8.2...2....83.......4..43.9......73...9.2....1.36..184.9.2.5...3..7";
        let mut rustoku = Rustoku::new_from_str(s).expect("Rustoku creation failed");
        let solution = rustoku.solve_any();
        assert!(
            solution.is_none(),
            "Expected no solution for this unsolvable puzzle"
        );
    }

    #[test]
    fn test_solve_until_with_bound() {
        let s = UNIQUE_PUZZLE;
        let mut rustoku =
            Rustoku::new_from_str(s).expect("Rustoku creation failed from puzzle string");

        let solutions = rustoku.solve_until(1);
        assert_eq!(
            1,
            solutions.len(),
            "Expected exactly one solution with bound = 1"
        );

        let all_solutions = rustoku.solve_until(0);
        assert_eq!(
            1,
            all_solutions.len(),
            "Expected exactly one solution for this board with bound = 0"
        );

        assert_eq!(
            solutions[0].board, all_solutions[0].board,
            "Solution with bound = 1 does not match the solution with bound = 0"
        );
    }

    #[test]
    fn test_solve_all_with_unique_puzzle() {
        let s = UNIQUE_PUZZLE;
        let mut rustoku =
            Rustoku::new_from_str(s).expect("Rustoku creation failed from unique puzzle string");
        let solutions = rustoku.solve_all();
        assert_eq!(
            1,
            solutions.len(),
            "Expected a unique solution for the board"
        );
    }

    #[test]
    fn test_solve_all_with_two_puzzle() {
        let s = TWO_PUZZLE;
        let mut rustoku =
            Rustoku::new_from_str(s).expect("Rustoku creation failed from two puzzle string");
        let solutions = rustoku.solve_all();
        assert_eq!(
            2,
            solutions.len(),
            "Expected two solutions for the given board"
        );
    }

    #[test]
    fn test_solve_all_with_six_puzzle() {
        let s = SIX_PUZZLE;
        let mut rustoku =
            Rustoku::new_from_str(s).expect("Rustoku creation failed from six puzzle string");
        let solutions = rustoku.solve_all();
        assert_eq!(
            6,
            solutions.len(),
            "Expected one solution for the six puzzle"
        );
    }

    #[test]
    fn test_solve_any_with_all_techniques() {
        let s = UNIQUE_PUZZLE;
        let rustoku = Rustoku::new_from_str(s).expect("Rustoku creation failed for technique test");
        let solution = rustoku
            .with_techniques(RustokuTechniques::ALL)
            .solve_any()
            .expect("Solving with all techniques failed");

        assert_eq!(
            UNIQUE_SOLUTION,
            format_line(&solution.board.cells),
            "Solution does not match the expected result with all techniques"
        );
    }

    #[test]
    fn test_solve_all_with_all_techniques() {
        let s = TWO_PUZZLE;
        let rustoku = Rustoku::new_from_str(s)
            .expect("Rustoku creation failed for multi-solution technique test");
        let solutions = rustoku.with_techniques(RustokuTechniques::ALL).solve_all();

        assert_eq!(
            2,
            solutions.len(),
            "Expected two solutions for the given board with all techniques"
        );
    }

    #[test]
    fn test_generate_with_enough_clues() {
        (20..=80).step_by(20).for_each(|num_clues| {
            let board = generate_board(num_clues)
                .expect(&format!("Board generation failed for {} clues", num_clues));
            let mut rustoku =
                Rustoku::new(board).expect("Rustoku creation failed from generated board");
            let clues_count = board
                .cells
                .iter()
                .flatten()
                .filter(|&&cell| cell != 0)
                .count();
            assert!(
                clues_count >= num_clues,
                "Expected at least {} clues, but found {} clues",
                num_clues,
                clues_count
            );

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
        let num_clues = 16;
        let result = generate_board(num_clues);
        assert!(matches!(result, Err(RustokuError::InvalidClueCount)));
    }

    #[test]
    fn test_generate_with_too_many_clues() {
        let num_clues = 82;
        let result = generate_board(num_clues);
        assert!(matches!(result, Err(RustokuError::InvalidClueCount)));
    }

    #[test]
    fn test_is_solved_with_valid_solution() {
        let s = UNIQUE_SOLUTION;
        let board = RustokuBoard::try_from(s).expect("Parsing valid solution failed");
        let rustoku = Rustoku::new(board).expect("Rustoku creation failed for solved check");
        assert!(rustoku.is_solved(), "The Sudoku puzzle should be solved");
    }

    #[test]
    fn test_is_solved_with_unsolved_board() {
        let s = UNIQUE_PUZZLE;
        let board = RustokuBoard::try_from(s).expect("Parsing unsolved puzzle failed");
        let rustoku = Rustoku::new(board).expect("Rustoku creation failed for unsolved check");
        assert!(!rustoku.is_solved(), "The board should not be valid");
    }
}
