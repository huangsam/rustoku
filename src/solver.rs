use rand::rng;
use rand::seq::SliceRandom;
use thiserror::Error;

/// Represents the types of errors that can occur while working with Sudoku puzzles.
///
/// This enum defines various error cases that can occur while working with Sudoku puzzles:
/// - The number of clues provided for puzzle generation is not between 17 and 81
/// - The input string does not contain exactly 81 characters
/// - The input string contains characters other than digits `0-9` or `.` or `_`
/// - The initial board contains duplicate values in rows, columns, or 3x3 boxes
#[derive(Debug, Error)]
pub enum SudokuError {
    #[error("Clues must be between 17 and 81 for a valid Sudoku puzzle")]
    InvalidClueCount,
    #[error("Input string must be exactly 81 characters long")]
    InvalidInputLength,
    #[error("Input string must contain only digits '0'-'9'")]
    InvalidInputCharacter,
    #[error("Initial board contains duplicates")]
    DuplicateValues,
}

/// A Sudoku puzzle solver that uses backtracking and bitmasking for efficient constraint tracking.
///
/// This struct provides methods to initialize a Sudoku board, check if a number can be placed in a cell,
/// place and remove numbers, and solve the Sudoku puzzle using a recursive backtracking algorithm.
/// It supports finding one solution, multiple solutions, or all solutions to the puzzle.
///
/// # Examples
///
/// Solving a Sudoku puzzle:
/// ```
/// use rustoku::SudokuSolver;
/// let board = [
///     [5, 3, 0, 0, 7, 0, 0, 0, 0],
///     [6, 0, 0, 1, 9, 5, 0, 0, 0],
///     [0, 9, 8, 0, 0, 0, 0, 6, 0],
///     [8, 0, 0, 0, 6, 0, 0, 0, 3],
///     [4, 0, 0, 8, 0, 3, 0, 0, 1],
///     [7, 0, 0, 0, 2, 0, 0, 0, 6],
///     [0, 6, 0, 0, 0, 0, 2, 8, 0],
///     [0, 0, 0, 4, 1, 9, 0, 0, 5],
///     [0, 0, 0, 0, 8, 0, 0, 7, 9],
/// ];
/// let mut solver = SudokuSolver::new(board).unwrap();
/// println!("Solved: {}", solver.solve_any().is_some());
/// ```
///
/// Generating a Sudoku puzzle:
/// ```
/// use rustoku::SudokuSolver;
/// let puzzle = SudokuSolver::generate(30).unwrap();
/// let solution = SudokuSolver::new(puzzle).unwrap().solve_all();
/// println!("Number of solutions: {}", solution.len());
/// ```
#[derive(Debug, Clone)]
pub struct SudokuSolver {
    /// The Sudoku board itself, where 0 represents an empty cell
    board: [[u8; 9]; 9],
    /// Bitmask to track used numbers in each row (1-9 mapped to bits 0-8)
    row_masks: [u16; 9],
    /// Bitmask to track used numbers in each column (1-9 mapped to bits 0-8)
    col_masks: [u16; 9],
    /// Bitmask to track used numbers in each 3x3 box (1-9 mapped to bits 0-8)
    box_masks: [u16; 9],
}

/// Creates a new `SudokuSolver` from a flat 81-byte array (row-major order).
/// Useful for compact board representations.
/// Returns an error if the board contains duplicates.
impl TryFrom<[u8; 81]> for SudokuSolver {
    type Error = SudokuError;

    fn try_from(bytes: [u8; 81]) -> Result<Self, Self::Error> {
        let mut board = [[0u8; 9]; 9];
        for i in 0..81 {
            board[i / 9][i % 9] = bytes[i];
        }
        Self::new(board)
    }
}

/// Creates a new `SudokuSolver` from a static string of 81 characters (`0-9` or `.` or `_`).
/// Returns an error if the string is not valid or contains duplicates.
impl TryFrom<&str> for SudokuSolver {
    type Error = SudokuError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.len() != 81 {
            return Err(SudokuError::InvalidInputLength);
        }
        let mut bytes = [0u8; 81];
        for (i, ch) in s.bytes().enumerate() {
            match ch {
                b'0'..=b'9' => bytes[i] = ch - b'0',
                b'.' => bytes[i] = 0, // Treat '.' as empty cell
                b'_' => bytes[i] = 0, // Treat '_' as empty cell
                _ => return Err(SudokuError::InvalidInputCharacter),
            }
        }
        bytes.try_into()
    }
}

impl SudokuSolver {
    /// Creates a new SudokuSolver from a 9x9 board.
    /// Initializes the bitmasks for rows, columns, and boxes based on the initial board.
    /// Returns an error if the board contains duplicates.
    pub fn new(initial_board: [[u8; 9]; 9]) -> Result<Self, SudokuError> {
        let mut solver = Self {
            board: initial_board,
            row_masks: [0; 9],
            col_masks: [0; 9],
            box_masks: [0; 9],
        };

        // Initialize the masks based on the given initial board
        for (i, &num) in initial_board.iter().flatten().enumerate() {
            if num != 0 {
                let (r, c) = (i / 9, i % 9);
                if !solver.is_safe(r, c, num) {
                    return Err(SudokuError::DuplicateValues);
                }
                solver.place_number(r, c, num);
            }
        }
        Ok(solver)
    }

    /// Helper to calculate the box index for a given row and column.
    fn get_box_idx(r: usize, c: usize) -> usize {
        (r / 3) * 3 + (c / 3)
    }

    /// Checks if a number can be placed at a given row and column.
    /// Uses bitmasks for efficient checking.
    fn is_safe(&self, r: usize, c: usize, num: u8) -> bool {
        let bit_to_check = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);

        // Check if the bit is already set in any of the masks
        !((self.row_masks[r] & bit_to_check != 0)
            || (self.col_masks[c] & bit_to_check != 0)
            || (self.box_masks[box_idx] & bit_to_check != 0))
    }

    /// Places a number on the board and updates the masks.
    /// Assumes `is_safe` has already been called and returned true.
    fn place_number(&mut self, r: usize, c: usize, num: u8) {
        let bit_to_set = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);

        self.board[r][c] = num;
        self.row_masks[r] |= bit_to_set;
        self.col_masks[c] |= bit_to_set;
        self.box_masks[box_idx] |= bit_to_set;
    }

    /// Removes a number from the board and updates the masks.
    /// Used during backtracking.
    fn remove_number(&mut self, r: usize, c: usize, num: u8) {
        let bit_to_unset = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);

        self.board[r][c] = 0; // Set back to empty
        self.row_masks[r] &= !bit_to_unset;
        self.col_masks[c] &= !bit_to_unset;
        self.box_masks[box_idx] &= !bit_to_unset;
    }

    /// Finds the next empty cell using MRV (Minimum Remaining Values).
    /// Returns `Some((r, c))` or `None` if the board is full.
    fn find_next_empty_cell(&self) -> Option<(usize, usize)> {
        (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter(|&(r, c)| self.board[r][c] == 0)
            .min_by_key(|&(r, c)| (1..=9).filter(|&num| self.is_safe(r, c, num)).count())
    }

    /// Recursively solves the Sudoku puzzle up to a certain bound.
    /// If `bound` is `0`, it finds all solutions.
    /// Returns the number of solutions found so far.
    fn solve_until_recursive(&mut self, solutions: &mut Vec<[[u8; 9]; 9]>, bound: usize) -> usize {
        if let Some((r, c)) = self.find_next_empty_cell() {
            let mut count = 0;
            let mut nums: Vec<u8> = (1..=9).collect();
            nums.shuffle(&mut rng());
            for num in nums {
                if self.is_safe(r, c, num) {
                    self.place_number(r, c, num);
                    count += self.solve_until_recursive(solutions, bound);
                    self.remove_number(r, c, num);

                    // Stop if we've reached the bound
                    if bound > 0 && solutions.len() >= bound {
                        return count;
                    }
                }
            }
            count
        } else {
            // If no empty cell, a solution is found. Add it to the list
            solutions.push(self.board);
            1
        }
    }

    /// Solves the Sudoku puzzle up to a certain bound.
    /// If `bound` is `0`, it finds all solutions.
    pub fn solve_until(&mut self, bound: usize) -> Vec<[[u8; 9]; 9]> {
        let mut solutions = Vec::new();
        self.solve_until_recursive(&mut solutions, bound);
        solutions
    }

    /// Attempts to solve the Sudoku puzzle using backtracking with MRV (Minimum Remaining Values).
    /// Returns `Some([[u8; 9]; 9])` if a solution is found, `None` otherwise.
    pub fn solve_any(&mut self) -> Option<[[u8; 9]; 9]> {
        self.solve_until(1).into_iter().next()
    }

    /// Finds all possible solutions for the Sudoku puzzle.
    /// Returns a vector of all solutions found.
    pub fn solve_all(&mut self) -> Vec<[[u8; 9]; 9]> {
        self.solve_until(0)
    }

    /// Generates a new Sudoku puzzle with a unique solution.
    ///
    /// The `num_clues` parameter specifies the desired number of initially
    /// filled cells (clues) in the generated puzzle. Fewer clues generally
    /// result in a harder puzzle. The actual number of clues may be slightly
    /// more than `num_clues` if it's impossible to remove more numbers
    /// while maintaining a unique solution.
    pub fn generate(num_clues: usize) -> Result<[[u8; 9]; 9], SudokuError> {
        if !(17..=81).contains(&num_clues) {
            return Err(SudokuError::InvalidClueCount);
        }

        // Start with a fully solved board
        let mut solver = SudokuSolver::new([[0; 9]; 9])?;
        let mut board = solver.solve_any().ok_or(SudokuError::DuplicateValues)?;

        // Shuffle all cell coordinates
        let mut cells: Vec<(usize, usize)> =
            (0..9).flat_map(|r| (0..9).map(move |c| (r, c))).collect();
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
                SudokuSolver::new(board).expect("Board state should be valid after removal");

            if temp_solver.solve_until(2).len() != 1 {
                board[r][c] = original; // Restore if not unique
            } else {
                clues -= 1;
            }
        }

        Ok(board)
    }
}

/// Prints the current state of the Sudoku board to the console.
/// Empty cells are shown as dots for clarity.
///
/// # Example
/// ```
/// use rustoku::print_sudoku_board;
/// let board = [
///     [5, 3, 4, 6, 7, 8, 9, 1, 2],
///     [6, 7, 2, 1, 9, 5, 3, 4, 8],
///     [1, 9, 8, 3, 4, 2, 5, 6, 7],
///     [8, 5, 9, 7, 6, 1, 4, 2, 3],
///     [4, 2, 6, 8, 5, 3, 7, 9, 1],
///     [7, 1, 3, 9, 2, 4, 8, 5, 6],
///     [9, 6, 1, 5, 3, 7, 2, 8, 4],
///     [2, 8, 7, 4, 1, 9, 6, 3, 5],
///     [3, 4, 5, 2, 8, 6, 1, 7, 9],
/// ];
/// print_sudoku_board(&board);
/// ```
pub fn print_sudoku_board(board: &[[u8; 9]; 9]) {
    let horizontal_line = "+-------+-------+-------+";

    println!("{}", horizontal_line); // Top line

    for (r, row) in board.iter().enumerate().take(9) {
        print!("|"); // Start of the row
        for (c, &cell) in row.iter().enumerate().take(9) {
            match cell {
                0 => print!(" ."),     // Empty cell, two spaces for alignment
                n => print!(" {}", n), // Number, two spaces for alignment
            }
            if (c + 1) % 3 == 0 {
                print!(" |"); // Vertical separator after every 3rd column
            }
        }
        println!(); // Newline at the end of the row

        if (r + 1) % 3 == 0 {
            println!("{}", horizontal_line); // Horizontal separator after every 3rd row
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SudokuError, SudokuSolver};

    const UNIQUE_PUZZLE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
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

        let solver_from_new = SudokuSolver::new(board).unwrap();
        let solver_from_bytes = SudokuSolver::try_from(flat_bytes).unwrap();
        let solver_from_str = SudokuSolver::try_from(board_str.as_str()).unwrap();

        assert_eq!(solver_from_new.board, solver_from_bytes.board);
        assert_eq!(solver_from_new.board, solver_from_str.board);
        assert_eq!(solver_from_bytes.board, solver_from_str.board);
    }

    #[test]
    fn test_try_from_with_valid_input() {
        let solver = SudokuSolver::try_from(UNIQUE_PUZZLE);
        assert!(solver.is_ok());
    }

    #[test]
    fn test_try_from_with_invalid_length() {
        let s = "53..7...."; // Too short
        let solver = SudokuSolver::try_from(s);
        assert!(matches!(solver, Err(SudokuError::InvalidInputLength)));
    }

    #[test]
    fn test_try_from_with_invalid_character() {
        let s = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..7X"; // 'X'
        let solver = SudokuSolver::try_from(s);
        assert!(matches!(solver, Err(SudokuError::InvalidInputCharacter)));
    }

    #[test]
    fn test_try_from_with_duplicate_initial_values() {
        let s = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..55...8..79";
        let solver = SudokuSolver::try_from(s);
        assert!(matches!(solver, Err(SudokuError::DuplicateValues)));
    }

    #[test]
    fn test_solve_any_with_solvable_sudoku() {
        let s = UNIQUE_PUZZLE;
        let mut solver = SudokuSolver::try_from(s).unwrap();
        let option = solver.solve_any();
        assert!(option.is_some());

        let expected_solution = [
            [5, 3, 4, 6, 7, 8, 9, 1, 2],
            [6, 7, 2, 1, 9, 5, 3, 4, 8],
            [1, 9, 8, 3, 4, 2, 5, 6, 7],
            [8, 5, 9, 7, 6, 1, 4, 2, 3],
            [4, 2, 6, 8, 5, 3, 7, 9, 1],
            [7, 1, 3, 9, 2, 4, 8, 5, 6],
            [9, 6, 1, 5, 3, 7, 2, 8, 4],
            [2, 8, 7, 4, 1, 9, 6, 3, 5],
            [3, 4, 5, 2, 8, 6, 1, 7, 9],
        ];

        assert_eq!(
            option.unwrap(),
            expected_solution,
            "Solution does not match the expected result"
        );
    }

    #[test]
    fn test_solve_until_with_bound() {
        let s = UNIQUE_PUZZLE;
        let mut solver = SudokuSolver::try_from(s).unwrap();

        // Test with bound = 1 (find only one solution)
        let solutions = solver.solve_until(1);
        assert_eq!(
            solutions.len(),
            1,
            "Expected exactly one solution with bound = 1"
        );

        // Test with bound = 0 (find all solutions)
        let all_solutions = solver.solve_until(0);
        assert_eq!(
            all_solutions.len(),
            1,
            "Expected exactly one solution for this board with bound = 0"
        );

        // Ensure the solution found with bound = 1 matches the solution found with bound = 0
        assert_eq!(
            solutions[0], all_solutions[0],
            "Solution with bound = 1 does not match the solution with bound = 0"
        );
    }

    #[test]
    fn test_solve_all_with_unique_solution() {
        let s = UNIQUE_PUZZLE;
        let mut solver = SudokuSolver::try_from(s).unwrap();
        let solutions = solver.solve_all();
        assert_eq!(
            solutions.len(),
            1,
            "Expected a unique solution for the board"
        );
    }

    #[test]
    fn test_solve_all_with_multiple_solutions() {
        let s = TWO_PUZZLE;
        let mut solver = SudokuSolver::try_from(s).unwrap();
        let solutions = solver.solve_all();
        assert_eq!(
            solutions.len(),
            2,
            "Expected two solutions for the given board"
        );
    }

    #[test]
    fn test_generate_with_30_clues() {
        let num_clues = 30;
        let puzzle = SudokuSolver::generate(num_clues).unwrap();
        let mut solver = SudokuSolver::new(puzzle).unwrap();

        // Ensure the puzzle has the correct number of clues
        let clues_count = puzzle.iter().flatten().filter(|&&cell| cell != 0).count();
        assert_eq!(
            clues_count, num_clues,
            "Expected {} clues, but found {} clues",
            num_clues, clues_count
        );

        // Ensure the puzzle has a unique solution
        let solutions = solver.solve_all();
        assert_eq!(
            solutions.len(),
            1,
            "Generated puzzle should have a unique solution"
        );
    }

    #[test]
    fn test_generate_with_too_few_clues() {
        let num_clues = 16; // Below the minimum valid clue count
        let result = SudokuSolver::generate(num_clues);
        assert!(matches!(result, Err(SudokuError::InvalidClueCount)));
    }

    #[test]
    fn test_generate_with_too_many_clues() {
        let num_clues = 82; // Above the maximum valid clue count
        let result = SudokuSolver::generate(num_clues);
        assert!(matches!(result, Err(SudokuError::InvalidClueCount)));
    }
}
