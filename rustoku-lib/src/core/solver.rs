use rand::prelude::SliceRandom;
use rand::rng;

use super::board::Board;
use super::candidates::{CandidateList, Candidates};
use super::masks::Masks;
use super::solution::{Solution, SolvePath, SolveStep};
use super::techniques::TechniquePropagator;
use super::techniques::flags::TechniqueFlags;
use crate::error::RustokuError;

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
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::core::Board;
    /// use rustoku_lib::Rustoku;
    ///
    /// let board = Board::default();
    /// let solver = Rustoku::new(board);
    /// assert!(solver.is_ok());
    /// ```
    pub fn new(initial_board: Board) -> Result<Self, RustokuError> {
        let board = initial_board; // Now takes a Board directly
        let mut masks = Masks::new();
        let mut candidates = Candidates::new();

        // 1. Constraint Mask Initialization & Duplicate Validation
        // Iterate through all 81 cells of the initial board. For each pre-filled clue,
        // verify that it does not violate row, column, or 3x3 box uniqueness constraints.
        for r in 0..9 {
            for c in 0..9 {
                let num = board.get(r, c);
                if num != 0 {
                    // Check if bit (1 << (num - 1)) is already set in this row, col, or box
                    if !masks.is_safe(r, c, num) {
                        return Err(RustokuError::DuplicateValues);
                    }
                    // Register the clue in the row, column, and box bitmasks
                    masks.add_number(r, c, num);
                }
            }
        }

        // 2. Candidate Cache Forward-Checking Initialization
        // For each empty cell, compute its candidate bitmask:
        // ~(row_mask | col_mask | box_mask) & 0x01FF.
        // Bit (v - 1) is set if and only if digit v (1..=9) is currently valid at (r, c).
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
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::core::Board;
    /// use rustoku_lib::Rustoku;
    ///
    /// let solver = Rustoku::builder()
    ///     .board(Board::default())
    ///     .build();
    /// assert!(solver.is_ok());
    /// ```
    pub fn builder() -> RustokuBuilder {
        RustokuBuilder::new()
    }

    /// Constructs a new `Rustoku` instance from a string representation of the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::Rustoku;
    ///
    /// let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    /// let solver = Rustoku::new_from_str(puzzle);
    /// assert!(solver.is_ok());
    /// ```
    pub fn new_from_str(s: &str) -> Result<Self, RustokuError> {
        let board = Board::try_from(s)?;
        Self::new(board)
    }

    /// Returns the existing Rustoku instance, with modified techniques.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::core::TechniqueFlags;
    /// use rustoku_lib::Rustoku;
    ///
    /// let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    /// let solver = Rustoku::new_from_str(puzzle).unwrap().with_techniques(TechniqueFlags::all());
    /// assert_eq!(solver.techniques, TechniqueFlags::all());
    /// ```
    pub fn with_techniques(mut self, techniques: TechniqueFlags) -> Self {
        self.techniques = techniques;
        self
    }

    /// Extracts a snapshot of all candidate lists as a 3D grid: `[row][col][candidates]`.
    ///
    /// Empty cells contain a sorted list of candidate digits (1–9).
    /// Filled cells return an empty list `vec![]`.
    ///
    /// Useful for telemetry, solving visualizers, and external bindings.
    pub(crate) fn candidate_grid_snapshot(&self) -> Vec<Vec<Vec<u8>>> {
        (0..9)
            .map(|r| {
                (0..9)
                    .map(|c| {
                        if self.board.get(r, c) != 0 {
                            vec![]
                        } else {
                            self.candidates.get_candidates(r, c)
                        }
                    })
                    .collect()
            })
            .collect()
    }

    /// Replays a single [`SolveStep`] on the current solver state.
    ///
    /// - For [`SolveStep::Placement`]: places `value` at `(row, col)`, synchronizing masks and candidate caches.
    /// - For [`SolveStep::CandidateElimination`]: clears the single candidate bit corresponding to `value`.
    pub(crate) fn apply_trace_step(&mut self, step: &SolveStep) {
        match *step {
            SolveStep::Placement {
                row, col, value, ..
            } => {
                self.place_number(row, col, value);
            }
            SolveStep::CandidateElimination {
                row, col, value, ..
            } => {
                let initial_mask = self.candidates.get(row, col);
                let refined_mask = initial_mask & !(1 << (value - 1));
                self.candidates.set(row, col, refined_mask);
            }
        }
    }

    /// Locates the empty cell with the fewest candidates (MRV heuristic).
    #[inline]
    pub(super) fn find_next_empty_cell(&self) -> Option<(usize, usize)> {
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

    /// Places a digit on the board and propagates constraints forward.
    #[inline]
    pub(super) fn place_number(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, num);
        self.masks.add_number(r, c, num);
        self.candidates
            .update_affected_cells_for(r, c, &self.masks, &self.board, Some(num));
    }

    /// Clears a digit from the board and recalculates affected candidate sets during backtracking.
    #[inline]
    pub(super) fn remove_number(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, 0); // Set back to empty
        self.masks.remove_number(r, c, num);
        self.candidates
            .update_affected_cells(r, c, &self.masks, &self.board);
        // Note: `update_affected_cells` will recalculate candidates for the removed cell.
    }

    /// Recursive depth-first backtracking search guided by MRV and forward checking.
    pub(super) fn solve_until_recursive(
        &mut self,
        solutions: &mut Vec<Solution>,
        path: &mut SolvePath,
        bound: usize,
    ) -> usize {
        self.solve_until_recursive_internal(solutions, path, bound, false)
    }

    /// Internal recursive depth-first backtracking search with optional candidate randomization.
    fn solve_until_recursive_internal(
        &mut self,
        solutions: &mut Vec<Solution>,
        path: &mut SolvePath,
        bound: usize,
        randomize: bool,
    ) -> usize {
        // Base case: no empty cells remain -> board is completely and validly solved
        let Some((r, c)) = self.find_next_empty_cell() else {
            solutions.push(Solution {
                board: self.board,
                solve_path: path.clone(),
            });
            return 1;
        };

        let mut count = 0;
        // Query candidate bitmask for the chosen MRV cell and unpack to stack candidate list
        let mask = self.candidates.get(r, c);
        let mut cands = CandidateList::from_mask(mask);
        if randomize {
            cands.as_mut_slice().shuffle(&mut rng());
        }

        for &num in cands.as_slice() {
            // Forward checking: ensure placement does not violate current masks
            if !self.masks.is_safe(r, c, num) {
                continue;
            }

            // Apply forward move: write to board, masks, and prune peer candidate caches
            self.place_number(r, c, num);
            let step_number = path.steps.len() as u32;
            path.steps.push(SolveStep::Placement {
                row: r,
                col: c,
                value: num,
                flags: TechniqueFlags::empty(),
                step_number,
                candidates_eliminated: 0,
                related_cell_count: 0,
                difficulty_point: 0,
            });

            // Recurse into child state space
            count += self.solve_until_recursive_internal(solutions, path, bound, randomize);

            // Backtrack: undo move, restore masks, recalculate candidates, pop solve step
            path.steps.pop();
            self.remove_number(r, c, num);

            // Early return if we have found the requested number of solutions
            if bound > 0 && solutions.len() >= bound {
                return count;
            }
        }

        count
    }

    /// Executes deterministic human solving techniques before starting backtracking search.
    pub(super) fn techniques_make_valid_changes(&mut self, path: &mut SolvePath) -> bool {
        let mut propagator = TechniquePropagator::new(
            &mut self.board,
            &mut self.masks,
            &mut self.candidates,
            self.techniques,
        );
        propagator.propagate_constraints(path, 0)
    }

    /// Solves the Sudoku puzzle up to `bound` solutions, returning solutions with their solve paths.
    ///
    /// Executes a two-phase strategy:
    /// 1. Applies deterministic human deduction techniques to place forced digits and eliminate candidates.
    /// 2. If cells remain, executes MRV-guided recursive backtracking depth-first search.
    ///
    /// Passing `bound = 0` searches for all possible solutions sequentially.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::Rustoku;
    ///
    /// let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    /// let mut solver = Rustoku::new_from_str(puzzle).unwrap();
    /// let solutions = solver.solve_until(1);
    /// assert_eq!(solutions.len(), 1);
    /// ```
    pub fn solve_until(&mut self, bound: usize) -> Vec<Solution> {
        let mut solutions = Vec::new();
        let mut path = SolvePath::default();

        // Phase 1: Run deterministic constraint propagation
        if !self.techniques_make_valid_changes(&mut path) {
            return solutions;
        }

        // Phase 2: Run recursive backtracking DFS with MRV
        self.solve_until_recursive(&mut solutions, &mut path, bound);
        solutions
    }

    /// Recursively counts valid solutions up to `bound` (or all if bound == 0) without telemetry allocations.
    fn count_solutions_recursive(&mut self, bound: usize) -> usize {
        let Some((r, c)) = self.find_next_empty_cell() else {
            return 1;
        };

        let mut count = 0;
        let mask = self.candidates.get(r, c);
        let cands = CandidateList::from_mask(mask);

        for &num in cands.as_slice() {
            if !self.masks.is_safe(r, c, num) {
                continue;
            }

            self.place_number(r, c, num);
            count += self.count_solutions_recursive(if bound > 0 {
                bound.saturating_sub(count)
            } else {
                0
            });
            self.remove_number(r, c, num);

            if bound > 0 && count >= bound {
                return count;
            }
        }

        count
    }

    /// Fast-path solution counter up to `bound` solutions (e.g. for uniqueness checking).
    ///
    /// Unlike [`Self::solve_until`], this method avoids constructing [`Solution`] objects,
    /// cloning boards, or recording telemetry paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::Rustoku;
    ///
    /// let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    /// let mut solver = Rustoku::new_from_str(puzzle).unwrap();
    /// assert_eq!(solver.count_solutions_until(2), 1);
    /// ```
    pub fn count_solutions_until(&mut self, bound: usize) -> usize {
        let mut path = SolvePath::default();
        if !self.techniques_make_valid_changes(&mut path) {
            return 0;
        }

        self.count_solutions_recursive(bound)
    }

    /// Attempts to solve the Sudoku puzzle using backtracking with MRV (Minimum Remaining Values).
    ///
    /// This is an optimized convenience wrapper around `solve_until(1)` to find the first valid solution.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::Rustoku;
    ///
    /// let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    /// let mut solver = Rustoku::new_from_str(puzzle).unwrap();
    /// let solution = solver.solve_any();
    /// assert!(solution.is_some());
    /// ```
    pub fn solve_any(&mut self) -> Option<Solution> {
        self.solve_until(1).into_iter().next()
    }

    /// Solves the Sudoku puzzle using randomized exploration (used during puzzle generation).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::core::Board;
    /// use rustoku_lib::Rustoku;
    ///
    /// let mut solver = Rustoku::new(Board::default()).unwrap();
    /// let solution = solver.solve_random();
    /// assert!(solution.is_some());
    /// ```
    pub fn solve_random(&mut self) -> Option<Solution> {
        let mut solutions = Vec::new();
        let mut path = SolvePath::default();

        if !self.techniques_make_valid_changes(&mut path) {
            return None;
        }

        self.solve_until_recursive_internal(&mut solutions, &mut path, 1, true);
        solutions.into_iter().next()
    }

    /// Finds all possible solutions for the Sudoku puzzle.
    ///
    /// Runs deterministic constraint propagation once on the root state, then executes
    /// depth-first backtracking search guided by MRV forward checking.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::Rustoku;
    ///
    /// let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    /// let mut solver = Rustoku::new_from_str(puzzle).unwrap();
    /// let solutions = solver.solve_all();
    /// assert_eq!(solutions.len(), 1);
    /// ```
    pub fn solve_all(&mut self) -> Vec<Solution> {
        let mut solutions = Vec::new();
        let mut path = SolvePath::default();

        if !self.techniques_make_valid_changes(&mut path) {
            return solutions;
        }

        self.solve_until_recursive(&mut solutions, &mut path, 0);
        solutions
    }

    /// Checks if the Sudoku puzzle is solved correctly.
    ///
    /// Validates that all 81 cells are non-empty and that no row, column, or 3x3 box
    /// contains duplicate values.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::Rustoku;
    ///
    /// let solved = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
    /// let solver = Rustoku::new_from_str(solved).unwrap();
    /// assert!(solver.is_solved());
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::core::Board;
    /// use rustoku_lib::Rustoku;
    ///
    /// let solver = Rustoku::builder()
    ///     .board(Board::default())
    ///     .build();
    /// assert!(solver.is_ok());
    /// ```
    pub fn build(self) -> Result<Rustoku, RustokuError> {
        let board = self.board.unwrap_or_default();
        let mut r = Rustoku::new(board)?;
        r.techniques = self.techniques;
        // If the user provided a max_solutions hint, we store it in techniques as not applicable
        // for now; the builder primarily configures creation state.
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_builder_configuration() {
        let builder = Rustoku::builder()
            .board_from_str(UNIQUE_PUZZLE)
            .expect("valid board string")
            .techniques(TechniqueFlags::all())
            .max_solutions(5);
        let solver = builder.build().expect("build solver");
        assert_eq!(solver.techniques, TechniqueFlags::all());
    }

    #[test]
    fn test_try_from_with_duplicate_initial_values() {
        let s = "530070000600195000098000060800060003400803001700020006060000280000419005500080079";
        let board = Board::try_from(s).expect("Board parsing failed before duplicate check");
        let rustoku = Rustoku::new(board);
        assert_eq!(rustoku.err(), Some(RustokuError::DuplicateValues));
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
