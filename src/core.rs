//! Core module for the Rustoku solver and generator.
//!
//! This module includes the `Rustoku` struct for representing and solving Sudoku puzzles using
//! backtracking and Minimum Remaining Values (MRV). It provides functionality for solving
//! puzzles, checking solutions, and generating puzzles with unique solutions.
//!
//! This module also defines `RustokuSolution` for representing solved boards, along with their
//! solve paths, and `RustokuError` for handling errors like invalid inputs.

use crate::error::RustokuError;
use crate::solution::RustokuSolution;
use rand::rng;
use rand::seq::SliceRandom;

/// A core Sudoku primitive that uses backtracking and bitmasking for constraints.
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
/// use rustoku::core::Rustoku;
/// let board = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
/// let mut rustoku = Rustoku::try_from(board).unwrap();
/// assert!(rustoku.solve_any().is_some());
/// ```
///
/// Generate a Sudoku puzzle:
/// ```
/// use rustoku::core::Rustoku;
/// let puzzle = Rustoku::generate(30).unwrap();
/// let solution = Rustoku::new(puzzle).unwrap().solve_all();
/// assert_eq!(solution.len(), 1);
/// ```
///
/// Check if a Sudoku puzzle is solved:
/// ```
/// use rustoku::core::Rustoku;
/// let board = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
/// let rustoku = Rustoku::try_from(board).unwrap();
/// assert!(rustoku.is_solved());
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Rustoku {
    /// The Sudoku board itself, where 0 represents an empty cell
    pub board: [[u8; 9]; 9],
    /// Bitmask to track used numbers in each row (1-9 mapped to bits 0-8)
    row_masks: [u16; 9],
    /// Bitmask to track used numbers in each column (1-9 mapped to bits 0-8)
    col_masks: [u16; 9],
    /// Bitmask to track used numbers in each 3x3 box (1-9 mapped to bits 0-8)
    box_masks: [u16; 9],
}

impl TryFrom<[u8; 81]> for Rustoku {
    type Error = RustokuError;

    fn try_from(bytes: [u8; 81]) -> Result<Self, Self::Error> {
        let mut board = [[0u8; 9]; 9];
        for i in 0..81 {
            board[i / 9][i % 9] = bytes[i];
        }
        Self::new(board)
    }
}

impl TryFrom<&str> for Rustoku {
    type Error = RustokuError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.len() != 81 {
            return Err(RustokuError::InvalidInputLength);
        }
        let mut bytes = [0u8; 81];
        for (i, ch) in s.bytes().enumerate() {
            match ch {
                b'0'..=b'9' => bytes[i] = ch - b'0',
                b'.' | b'_' => bytes[i] = 0, // Treat '.' and '_' as empty cells
                _ => return Err(RustokuError::InvalidInputCharacter),
            }
        }
        bytes.try_into()
    }
}

impl Rustoku {
    pub fn new(initial_board: [[u8; 9]; 9]) -> Result<Self, RustokuError> {
        let mut rustoku = Self {
            board: initial_board,
            row_masks: [0; 9],
            col_masks: [0; 9],
            box_masks: [0; 9],
        };

        // Initialize the masks based on the given initial board
        for (i, &num) in initial_board.iter().flatten().enumerate() {
            if num != 0 {
                let (r, c) = (i / 9, i % 9);
                if !rustoku.is_safe(r, c, num) {
                    return Err(RustokuError::DuplicateValues);
                }
                rustoku.place_number(r, c, num);
            }
        }
        Ok(rustoku)
    }

    /// Returns the index of the 3x3 box for a given row and column.
    fn get_box_idx(r: usize, c: usize) -> usize {
        (r / 3) * 3 + (c / 3)
    }

    /// Checks if placing a number in the given cell is safe according to Sudoku rules.
    fn is_safe(&self, r: usize, c: usize, num: u8) -> bool {
        let bit_to_check = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);

        (self.row_masks[r] & bit_to_check == 0)
            && (self.col_masks[c] & bit_to_check == 0)
            && (self.box_masks[box_idx] & bit_to_check == 0)
    }

    /// Places a number in the Sudoku board and updates the corresponding masks.
    fn place_number(&mut self, r: usize, c: usize, num: u8) {
        let bit_to_set = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);

        self.board[r][c] = num;
        self.row_masks[r] |= bit_to_set;
        self.col_masks[c] |= bit_to_set;
        self.box_masks[box_idx] |= bit_to_set;
    }

    /// Removes a number from the Sudoku board and updates the masks accordingly.
    fn remove_number(&mut self, r: usize, c: usize, num: u8) {
        let bit_to_unset = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);

        self.board[r][c] = 0; // Set back to empty
        self.row_masks[r] &= !bit_to_unset;
        self.col_masks[c] &= !bit_to_unset;
        self.box_masks[box_idx] &= !bit_to_unset;
    }

    /// Returns a bitmask of possible candidates for a given empty cell.
    ///
    /// Computes the bitmask of legal candidates for a cell by combining the row, column,
    /// and box masks, inverting, and masking with `0x1FF` to keep only the lower 9 bits.
    fn get_possible_candidates_mask(&self, r: usize, c: usize) -> u16 {
        let row_mask = self.row_masks[r];
        let col_mask = self.col_masks[c];
        let box_mask = self.box_masks[Self::get_box_idx(r, c)];
        let used = row_mask | col_mask | box_mask;
        // 0x1FF is 111111111 in binary (all 9 bits set)
        // Inverting used and ANDing with 0x1FF gives us the available candidates.
        !used & 0x1FF
    }

    /// Finds the next empty cell in the Sudoku board using MRV (Minimum Remaining Values).
    fn find_next_empty_cell(&self) -> Option<(usize, usize)> {
        let mut min = (10, None);
        for r in 0..9 {
            for c in 0..9 {
                if self.board[r][c] == 0 {
                    let count = self.get_possible_candidates_mask(r, c).count_ones() as u8;
                    if count < min.0 {
                        min = (count, Some((r, c)));
                        if count == 1 {
                            return min.1;
                        }
                    }
                }
            }
        }
        min.1
    }

    /// Applies the naked singles technique.
    ///
    /// This technique fills in cells that have only one possible value.
    /// Returns true if any placements were made.
    fn naked_singles(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut changed = false;
        let mut progress = Vec::new();
        for r in 0..9 {
            for c in 0..9 {
                if self.board[r][c] == 0 {
                    let mask = self.get_possible_candidates_mask(r, c);
                    if mask.count_ones() == 1 {
                        let num = mask.trailing_zeros() as u8 + 1;
                        self.place_number(r, c, num);
                        progress.push((r, c, num));
                        changed = true;
                    }
                }
            }
        }
        for (r, c, num) in progress {
            path.push((r, c, num));
        }
        changed
    }

    /// Applies the hidden singles technique.
    ///
    /// This technique fills in cells where a value can only go in one place in a row, column, or box.
    /// Returns true if any placements were made.
    fn hidden_singles(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut changed = false;

        // Helper closure to avoid code duplication for rows, columns, and boxes
        let mut check_unit_for_hidden_singles = |cells_in_unit: &[(usize, usize)]| {
            let mut unit_changed = false;
            // For each possible number (1-9)
            for num_val in 1..=9 {
                let bit = 1 << (num_val - 1);
                let mut found_pos = None;
                let mut count = 0;

                // Iterate through cells in the current unit
                for &(r, c) in cells_in_unit.iter() {
                    if self.board[r][c] == 0 {
                        // If cell is empty
                        // Check if 'num_val' is a possible candidate in this cell
                        let possible_candidates_mask = self.get_possible_candidates_mask(r, c);
                        if possible_candidates_mask & bit != 0 {
                            count += 1;
                            found_pos = Some((r, c)); // Store the last valid position found
                        }
                    }
                }

                // If 'num_val' was a candidate in exactly one empty cell in this unit
                if count == 1 {
                    if let Some((hr, hc)) = found_pos {
                        self.place_number(hr, hc, num_val);
                        path.push((hr, hc, num_val));
                        unit_changed = true;
                    }
                }
            }
            unit_changed
        };

        // Check rows
        for r in 0..9 {
            let cells_in_row: Vec<(usize, usize)> = (0..9).map(|c| (r, c)).collect();
            changed |= check_unit_for_hidden_singles(&cells_in_row);
        }

        // Check columns
        for c in 0..9 {
            let cells_in_col: Vec<(usize, usize)> = (0..9).map(|r| (r, c)).collect();
            changed |= check_unit_for_hidden_singles(&cells_in_col);
        }

        // Check boxes
        for box_idx in 0..9 {
            let mut cells_in_box: Vec<(usize, usize)> = Vec::new();
            let start_r = (box_idx / 3) * 3;
            let start_c = (box_idx % 3) * 3;
            for dr in 0..3 {
                for dc in 0..3 {
                    cells_in_box.push((start_r + dr, start_c + dc));
                }
            }
            changed |= check_unit_for_hidden_singles(&cells_in_box);
        }

        changed
    }

    /// Helper function to apply all deterministic constraint propagation strategies.
    ///
    /// Returns `true` if propagation was successful (no contradiction), `false` otherwise.
    /// Updates the `path` with any numbers placed by propagation.
    fn propagate_constraints(&mut self, path: &mut Vec<(usize, usize, u8)>, path_len_before: usize) -> bool {
        loop {
            let changed = self.naked_singles(path) | self.hidden_singles(path);

            // Fast contradiction check: for each empty cell, if no possible value, fail
            for r in 0..9 {
                for c in 0..9 {
                    if self.board[r][c] == 0 && self.get_possible_candidates_mask(r, c) == 0 {
                        while path.len() > path_len_before {
                            let (r, c, num) = path.pop().unwrap();
                            self.remove_number(r, c, num);
                        }
                        return false;
                    }
                }
            }

            if !changed {
                break;
            }
        }
        true
    }

    /// Recursively solves the Sudoku puzzle up to a certain bound, tracking the solve path.
    fn solve_until_recursive(
        &mut self,
        solutions: &mut Vec<RustokuSolution>,
        path: &mut Vec<(usize, usize, u8)>,
        bound: usize,
    ) -> usize {
        let path_len_before = path.len();

        if !self.propagate_constraints(path, path_len_before) {
            return 0;
        }

        let result = if let Some((r, c)) = self.find_next_empty_cell() {
            let mut count = 0;
            let mut nums: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
            nums.shuffle(&mut rng());
            for &num in &nums {
                if self.is_safe(r, c, num) {
                    self.place_number(r, c, num);
                    path.push((r, c, num));
                    count += self.solve_until_recursive(solutions, path, bound);
                    path.pop();
                    self.remove_number(r, c, num);

                    if bound > 0 && solutions.len() >= bound {
                        break;
                    }
                }
            }
            count
        } else {
            solutions.push(RustokuSolution {
                board: self.board,
                solve_path: path.clone(),
            });
            1
        };

        while path.len() > path_len_before {
            let (r, c, num) = path.pop().unwrap();
            self.remove_number(r, c, num);
        }

        result
    }

    /// Solves the Sudoku puzzle up to a certain bound, returning solutions with their solve paths.
    ///
    /// If `bound` is `0`, it finds all solutions. Otherwise, it finds up to `bound` solutions.
    pub fn solve_until(&mut self, bound: usize) -> Vec<RustokuSolution> {
        let mut solutions = Vec::new();
        let mut path = Vec::new();
        self.solve_until_recursive(&mut solutions, &mut path, bound);
        solutions
    }

    /// Attempts to solve the Sudoku puzzle using backtracking with MRV (Minimum Remaining Values).
    ///
    /// Returns `Some(RustokuSolution)` if a solution is found, `None` otherwise.
    pub fn solve_any(&mut self) -> Option<RustokuSolution> {
        self.solve_until(1).into_iter().next()
    }

    /// Finds all possible solutions for the Sudoku puzzle.
    ///
    /// Returns a vector of all solutions found.
    pub fn solve_all(&mut self) -> Vec<RustokuSolution> {
        self.solve_until(0)
    }

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

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_generate_with_enough_clues() {
        for &num_clues in &[17, 30, 50, 70, 81] {
            let puzzle = Rustoku::generate(num_clues).unwrap();
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
        }
    }

    #[test]
    fn test_generate_with_too_few_clues() {
        let num_clues = 16; // Below the minimum valid clue count
        let result = Rustoku::generate(num_clues);
        assert!(matches!(result, Err(RustokuError::InvalidClueCount)));
    }

    #[test]
    fn test_generate_with_too_many_clues() {
        let num_clues = 82; // Above the maximum valid clue count
        let result = Rustoku::generate(num_clues);
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
