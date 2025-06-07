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
mod masks;
mod solution;
mod techniques;

pub use board::Board;
pub use solution::{Solution, SolvePath, SolveStep};
pub use techniques::flags::TechniqueFlags;

use crate::error::RustokuError;
use candidates::Candidates;
use masks::Masks;
use rand::prelude::SliceRandom;
use rand::rng;
use techniques::TechniquePropagator;

/// Solver primitive that uses backtracking and bitmasking for constraints.
///
/// This struct supports the ability to:
/// - Initialize from a 2D array, a flat byte array, or a string representation
/// - Solve a Sudoku puzzle using backtracking with Minimum Remaining Values (MRV)
/// - Generate a Sudoku puzzle with a unique solution based on the number of clues specified
/// - Check if a Sudoku puzzle is solved correctly
///
/// # Examples
///
/// Solve a Sudoku puzzle:
/// ```
/// use rustoku_lib::Rustoku;
/// let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
/// let mut rustoku = Rustoku::new_from_str(puzzle).unwrap();
/// assert!(rustoku.solve_any().is_some());
/// ```
///
/// Generate a Sudoku puzzle:
/// ```
/// use rustoku_lib::{Rustoku, generate_board};
/// let board = generate_board(30).unwrap();
/// let solution = Rustoku::new(board).unwrap().solve_all();
/// assert_eq!(solution.len(), 1);
/// ```
///
/// Check if a Sudoku puzzle is solved:
/// ```
/// use rustoku_lib::Rustoku;
/// let puzzle = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
/// let rustoku = Rustoku::new_from_str(puzzle).unwrap();
/// assert!(rustoku.is_solved());
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Rustoku {
    /// The current state of the Sudoku board.
    pub board: Board,
    masks: Masks,
    candidates: Candidates,
    techniques: TechniqueFlags,
}

impl Rustoku {
    /// Constructs a new `Rustoku` instance from an initial `Board`.
    pub fn new(initial_board: Board) -> Result<Self, RustokuError> {
        let board = initial_board; // Now takes a Board directly
        let mut masks = Masks::new();
        let mut candidates = Candidates::new();

        // Initialize masks and check for duplicates based on the provided board
        for r in 0..9 {
            for c in 0..9 {
                let num = board.get(r, c);
                if num != 0 {
                    if !masks.is_safe(r, c, num) {
                        return Err(RustokuError::DuplicateValues);
                    }
                    masks.add_number(r, c, num);
                }
            }
        }

        // Initialize the candidates cache for empty cells based on initial masks and board
        for r in 0..9 {
            for c in 0..9 {
                if board.is_empty(r, c) {
                    candidates.set(r, c, masks.compute_candidates_mask_for_cell(r, c));
                }
            }
        }

        Ok(Self {
            board,
            masks,
            candidates,
            techniques: TechniqueFlags::EASY, // Default
        })
    }

    /// Constructs a new `Rustoku` instance from a string representation of the board.
    pub fn new_from_str(s: &str) -> Result<Self, RustokuError> {
        let board = Board::try_from(s)?;
        Self::new(board)
    }

    /// Returns the existing Rustoku instance, with modified techniques.
    pub fn with_techniques(mut self, techniques: TechniqueFlags) -> Self {
        self.techniques = techniques;
        self
    }

    /// Helper for solver to find the next empty cell (MRV).
    fn find_next_empty_cell(&self) -> Option<(usize, usize)> {
        let mut min = (10, None); // Min candidates, (r, c)
        for (r, c) in self.board.iter_empty_cells() {
            let count = self.candidates.get(r, c).count_ones() as u8;
            if count < min.0 {
                min = (count, Some((r, c)));
                if count == 1 {
                    return min.1;
                }
            }
        }
        min.1
    }

    /// Place and remove operations for the solver, updated to use the new structs.
    fn place_number(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, num);
        self.masks.add_number(r, c, num);
        self.candidates
            .update_affected_cells(r, c, &self.masks, &self.board);
    }

    /// Remove a number from the board and update masks and candidates.
    fn remove_number(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, 0); // Set back to empty
        self.masks.remove_number(r, c, num);
        self.candidates
            .update_affected_cells(r, c, &self.masks, &self.board);
        // Note: `update_affected_cells` will recalculate candidates for the removed cell.
    }

    /// Recursive function to solve the Sudoku puzzle with backtracking.
    fn solve_until_recursive(
        &mut self,
        solutions: &mut Vec<Solution>,
        path: &mut SolvePath,
        bound: usize,
    ) -> usize {
        if let Some((r, c)) = self.find_next_empty_cell() {
            let mut count = 0;
            let mut nums: Vec<u8> = (1..=9).collect();
            nums.shuffle(&mut rng());

            for &num in &nums {
                if self.masks.is_safe(r, c, num) {
                    self.place_number(r, c, num);
                    path.steps.push(SolveStep::Placement {
                        row: r,
                        col: c,
                        value: num,
                        flags: TechniqueFlags::empty(),
                    });
                    count += self.solve_until_recursive(solutions, path, bound);
                    path.steps.pop();
                    self.remove_number(r, c, num);

                    if bound > 0 && solutions.len() >= bound {
                        return count;
                    }
                }
            }
            count
        } else {
            solutions.push(Solution {
                board: self.board,
                solve_path: path.clone(),
            });
            1
        }
    }

    /// Solves the Sudoku puzzle up to a certain bound, returning solutions with their solve paths.
    pub fn solve_until(&mut self, bound: usize) -> Vec<Solution> {
        let mut solutions = Vec::new();
        let mut path = SolvePath::default();

        let mut propagator = TechniquePropagator::new(
            &mut self.board,
            &mut self.masks,
            &mut self.candidates,
            self.techniques,
        );

        if !propagator.propagate_constraints(&mut path, 0) {
            return solutions; // Early exit if initial constraints are inconsistent
        }

        self.solve_until_recursive(&mut solutions, &mut path, bound);
        solutions
    }

    /// Attempts to solve the Sudoku puzzle using backtracking with MRV (Minimum Remaining Values).
    pub fn solve_any(&mut self) -> Option<Solution> {
        self.solve_until(1).into_iter().next()
    }

    /// Finds all possible solutions for the Sudoku puzzle.
    pub fn solve_all(&mut self) -> Vec<Solution> {
        self.solve_until(0)
    }

    /// Checks if the Sudoku puzzle is solved correctly.
    pub fn is_solved(&self) -> bool {
        self.board.cells.iter().flatten().all(|&val| val != 0) && Rustoku::new(self.board).is_ok()
    }
}

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
/// use rustoku_lib::generate_board;
/// let puzzle = generate_board(30).unwrap();
/// assert_eq!(puzzle.cells.len(), 9);
/// assert_eq!(puzzle.cells[0].len(), 9);
/// ```
pub fn generate_board(num_clues: usize) -> Result<Board, RustokuError> {
    if !(17..=81).contains(&num_clues) {
        return Err(RustokuError::InvalidClueCount);
    }

    // Start with a fully solved board
    let mut rustoku = Rustoku::new(Board::empty())?;
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
        return Err(RustokuError::GenerateFailure);
    }

    Ok(board)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::board::Board;
    use crate::error::RustokuError;
    use crate::format::format_line;

    const UNIQUE_PUZZLE: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const UNIQUE_SOLUTION: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
    const TWO_PUZZLE: &str =
        "295743861431865900876192543387459216612387495549216738763504189928671354154938600";
    const SIX_PUZZLE: &str =
        "295743001431865900876192543387459216612387495549216738763500000000000000000000000";

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

        let board_from_new = Board::new(board);
        let board_from_bytes = Board::try_from(flat_bytes).expect("Board from flat bytes failed");
        let board_from_str = Board::try_from(board_str.as_str()).expect("Board from string failed");

        assert_eq!(board_from_new, board_from_bytes);
        assert_eq!(board_from_new, board_from_str);
        assert_eq!(board_from_bytes, board_from_str);
    }

    #[test]
    fn test_try_from_with_valid_input() {
        let rustoku = Board::try_from(UNIQUE_PUZZLE);
        assert!(rustoku.is_ok());
    }

    #[test]
    fn test_try_from_with_invalid_length() {
        let s = "530070000"; // Too short
        let rustoku = Board::try_from(s);
        assert!(matches!(rustoku, Err(RustokuError::InvalidInputLength)));
    }

    #[test]
    fn test_try_from_with_invalid_character() {
        let s = "53007000060019500009800006080006000340080300170002000606000028000041900500008007X"; // 'X'
        let rustoku = Board::try_from(s);
        assert!(matches!(rustoku, Err(RustokuError::InvalidInputCharacter)));
    }

    #[test]
    fn test_try_from_with_duplicate_initial_values() {
        let s = "530070000600195000098000060800060003400803001700020006060000280000419005500080079";
        let board = Board::try_from(s).expect("Board parsing failed before duplicate check");
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
        let s = "078002609030008020002000083000000040043090000007300090200001036001840902050003007";
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
            .with_techniques(TechniqueFlags::all())
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
        let solutions = rustoku.with_techniques(TechniqueFlags::all()).solve_all();

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
                .unwrap_or_else(|_| panic!("Board generation failed for {} clues", num_clues));
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
        let rustoku = Rustoku::new_from_str(s).expect("Rustoku creation failed for solved check");
        assert!(rustoku.is_solved(), "The Sudoku puzzle should be solved");
    }

    #[test]
    fn test_is_solved_with_unsolved_board() {
        let s = UNIQUE_PUZZLE;
        let rustoku = Rustoku::new_from_str(s).expect("Rustoku creation failed for unsolved check");
        assert!(!rustoku.is_solved(), "The board should not be valid");
    }
}
