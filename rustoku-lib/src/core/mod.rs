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
pub use candidates::Candidates;
pub use masks::Masks;
pub use solution::{Solution, SolvePath, SolveStep};
pub use techniques::flags::TechniqueFlags;

use crate::error::RustokuError;
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
    /// Bitmasks that check if a cell is safe in a row, column and box.
    pub masks: Masks,
    /// Candidate cache from computing the bitmasks.
    pub candidates: Candidates,
    /// Techniques used during the initial phase of solving.
    pub techniques: TechniqueFlags,
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

    /// Start building a configured `Rustoku` via a builder pattern.
    pub fn builder() -> RustokuBuilder {
        RustokuBuilder::new()
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

    /// Run techniques and check if they make valid changes.
    fn techniques_make_valid_changes(&mut self, path: &mut SolvePath) -> bool {
        let mut propagator = TechniquePropagator::new(
            &mut self.board,
            &mut self.masks,
            &mut self.candidates,
            self.techniques,
        );
        propagator.propagate_constraints(path, 0)
    }

    /// Solves the Sudoku puzzle up to a certain bound, returning solutions with their solve paths.
    pub fn solve_until(&mut self, bound: usize) -> Vec<Solution> {
        let mut solutions = Vec::new();
        let mut path = SolvePath::default();

        if !self.techniques_make_valid_changes(&mut path) {
            return solutions;
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

/// A simple builder for constructing `Rustoku` with fluent configuration.
pub struct RustokuBuilder {
    board: Option<Board>,
    techniques: TechniqueFlags,
    max_solutions: Option<usize>,
}

impl RustokuBuilder {
    /// Create a new builder with reasonable defaults.
    pub fn new() -> Self {
        RustokuBuilder {
            board: None,
            techniques: TechniqueFlags::EASY,
            max_solutions: None,
        }
    }
}

impl Default for RustokuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RustokuBuilder {
    /// Provide the initial `Board` for the solver.
    pub fn board(mut self, board: Board) -> Self {
        self.board = Some(board);
        self
    }

    /// Provide the initial board as a string (convenience).
    pub fn board_from_str(mut self, s: &str) -> Result<Self, RustokuError> {
        let board = Board::try_from(s)?;
        self.board = Some(board);
        Ok(self)
    }

    /// Configure which techniques the solver should use.
    pub fn techniques(mut self, techniques: TechniqueFlags) -> Self {
        self.techniques = techniques;
        self
    }

    /// Optionally hint the builder with a maximum number of solutions.
    pub fn max_solutions(mut self, max: usize) -> Self {
        self.max_solutions = Some(max);
        self
    }

    /// Finalize the builder and construct the `Rustoku` instance.
    pub fn build(self) -> Result<Rustoku, RustokuError> {
        let board = self.board.unwrap_or_default();
        let mut r = Rustoku::new(board)?;
        r.techniques = self.techniques;
        // If the user provided a max_solutions hint, we store it in techniques as not applicable
        // for now; the builder primarily configures creation state.
        Ok(r)
    }
}

/// Iterator wrapper for solutions. For simplicity this implementation eagerly
/// computes all solutions via `solve_all()` and yields them lazily.
#[derive(Debug, Clone)]
pub struct Solutions {
    solutions: Vec<Solution>,
    idx: usize,
}

impl Solutions {
    /// Construct a `Solutions` iterator from an existing `Rustoku` solver.
    pub fn from_solver(mut solver: Rustoku) -> Self {
        let solutions = solver.solve_all();
        Solutions { solutions, idx: 0 }
    }
}

impl Iterator for Solutions {
    type Item = Solution;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.solutions.len() {
            None
        } else {
            let s = self.solutions[self.idx].clone();
            self.idx += 1;
            Some(s)
        }
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
/// let puzzle = generate_board(30);
/// assert!(puzzle.is_ok());
/// ```
pub fn generate_board(num_clues: usize) -> Result<Board, RustokuError> {
    if !(17..=81).contains(&num_clues) {
        return Err(RustokuError::InvalidClueCount);
    }

    // Start with a fully solved board
    let mut rustoku = Rustoku::new(Board::default())?;
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
    fn test_builder_and_iterator() {
        let board = Board::try_from(UNIQUE_PUZZLE).expect("valid puzzle");
        let solver = Rustoku::builder()
            .board(board)
            .techniques(TechniqueFlags::all())
            .build()
            .expect("builder build");

        // Using the iterator wrapper (eager compute, lazy yield)
        let mut sols = Solutions::from_solver(solver);
        let first = sols.next();
        assert!(first.is_some());
        // For unique puzzle, there should be exactly one solution
        assert!(sols.next().is_none());
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
            format_line(&solution.board),
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
            format_line(&solution.board),
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
                .unwrap_or_else(|_| panic!("Board generation failed for {num_clues} clues"));
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
                "Expected at least {num_clues} clues, but found {clues_count} clues"
            );

            let solutions = rustoku.solve_all();
            assert_eq!(
                1,
                solutions.len(),
                "Generated puzzle with {num_clues} clues should have a unique solution"
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

    struct TechniqueTestCase<'a> {
        name: &'a str,
        trigger_string: &'a str,
        technique_flag: TechniqueFlags,
    }

    #[test]
    fn test_each_technique_makes_valid_changes() {
        let test_cases = vec![
            // Last digit is empty, no other option exists
            TechniqueTestCase {
                name: "Naked Singles",
                trigger_string: "385421967194756328627983145571892634839645271246137589462579813918364752753218490",
                technique_flag: TechniqueFlags::NAKED_SINGLES,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=h101&tech=Hidden+Single
            TechniqueTestCase {
                name: "Hidden Singles",
                trigger_string: "008007000016083000000000051107290000000000000000046307290000000000860140000300700",
                technique_flag: TechniqueFlags::HIDDEN_SINGLES,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=n201&tech=Naked+Pair
            TechniqueTestCase {
                name: "Naked Pairs",
                trigger_string: "700009030000105006400260009002083951007000000005600000000000003100000060000004010",
                technique_flag: TechniqueFlags::NAKED_PAIRS,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=h201&tech=Hidden+Pair
            TechniqueTestCase {
                name: "Hidden Pairs",
                trigger_string: "000032000000000000007600914096000800005008000030040005050200000700000560904010000",
                technique_flag: TechniqueFlags::HIDDEN_PAIRS,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=lc101&tech=Locked+Candidates+Type+1+%28Pointing%29
            TechniqueTestCase {
                name: "Locked Candidates",
                trigger_string: "984000000000500040000000002006097200003002000000000010005060003407051890030009700",
                technique_flag: TechniqueFlags::LOCKED_CANDIDATES,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=bf201&tech=X-Wing
            TechniqueTestCase {
                name: "X-Wing",
                trigger_string: "000000000760003002002640009403900070000004903005000020010560000370090041000000060",
                technique_flag: TechniqueFlags::XWING,
            },
        ];

        for test_case in test_cases {
            let rustoku = Rustoku::new_from_str(test_case.trigger_string)
                .unwrap_or_else(|_| panic!("Rustoku creation failed for '{}'", test_case.name));
            assert!(
                rustoku
                    .with_techniques(test_case.technique_flag)
                    .techniques_make_valid_changes(&mut SolvePath::default()),
                "The board should change for '{}'",
                test_case.name
            )
        }
    }
}
