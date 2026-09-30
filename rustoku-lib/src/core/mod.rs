//! Core module for the Rustoku solver and generator.
//!
//! Provides the [`Rustoku`] solver, [`BoardGenerator`], and solution types.
//!
//! ### Architecture & Solving Pipeline
//! The engine operates across three synchronized data structures maintaining a ~1KB memory footprint:
//! - [`Board`]: 9x9 byte matrix storing current cell assignments (0 for empty, 1–9 for digits).
//! - [`Masks`]: Bitmasks for each row, column, and 3x3 box tracking placed digits.
//! - [`Candidates`]: Precomputed 9-bit bitmask cache per empty cell representing valid candidate digits.
//!
//! The solving pipeline combines deterministic human deduction with depth-first search:
//! 1. **Deterministic Constraint Propagation**: Enabled human techniques run in [`TechniquePropagator`]
//!    to eliminate candidates and place forced digits without guessing.
//! 2. **MRV-Guided Backtracking**: If techniques do not completely solve the puzzle, recursive DFS
//!    explores the state space using the Minimum Remaining Values (MRV / "fail-first") heuristic.
//! 3. **Parallel Search**: In [`Rustoku::solve_all`], top-level branches of the root MRV cell are
//!    explored in parallel across CPU threads via Rayon.
//! 4. **Lazy Iteration**: For on-demand solution streaming, [`Solutions`] provides an explicit-stack
//!    iterator that avoids computing all solutions up front.

mod board;
mod candidates;
mod masks;
mod solution;
mod techniques;

pub use board::Board;
pub use candidates::Candidates;
pub use masks::Masks;
pub use solution::{Solution, SolvePath, SolveStep};
pub use techniques::flags::{Difficulty, TechniqueFlags};

use crate::error::RustokuError;
use rand::prelude::SliceRandom;
use rand::rng;
use std::collections::HashSet;
use techniques::TechniquePropagator;

/// Represents the type of symmetry to apply during board generation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Symmetry {
    /// No symmetry (clues placed randomly).
    #[default]
    None,
    /// 180-degree rotational (point) symmetry.
    Rotational180,
    /// 90-degree rotational symmetry.
    Rotational90,
    /// Mirror symmetry across the vertical center line.
    MirrorVertical,
    /// Mirror symmetry across the horizontal center line.
    MirrorHorizontal,
    /// Mirror symmetry across the main diagonal (top-left to bottom-right).
    MirrorDiagonal,
}

impl Symmetry {
    /// Returns the symmetric partners for a given cell (r, c).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::core::Symmetry;
    ///
    /// let partners = Symmetry::Rotational180.get_partners(0, 0);
    /// assert!(partners.contains(&(0, 0)));
    /// assert!(partners.contains(&(8, 8)));
    /// ```
    pub fn get_partners(&self, r: usize, c: usize) -> Vec<(usize, usize)> {
        let mut partners = HashSet::new();
        partners.insert((r, c));

        match self {
            Symmetry::None => {}
            Symmetry::Rotational180 => {
                partners.insert((8 - r, 8 - c));
            }
            Symmetry::Rotational90 => {
                partners.insert((c, 8 - r));
                partners.insert((8 - r, 8 - c));
                partners.insert((8 - c, r));
            }
            Symmetry::MirrorVertical => {
                partners.insert((r, 8 - c));
            }
            Symmetry::MirrorHorizontal => {
                partners.insert((8 - r, c));
            }
            Symmetry::MirrorDiagonal => {
                partners.insert((c, r));
            }
        }

        partners.into_iter().collect()
    }
}

/// A builder for generating Sudoku puzzles with various constraints and properties.
///
/// `BoardGenerator` provides a unified interface for creating puzzles with specific
/// clue counts, symmetry types, and difficulty levels.
///
/// # Example
///
/// ```
/// use rustoku_lib::{BoardGenerator, Symmetry};
///
/// let board = BoardGenerator::new()
///     .clues(25)
///     .symmetry(Symmetry::Rotational180)
///     .generate();
///
/// assert!(board.is_ok());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BoardGenerator {
    num_clues: usize,
    symmetry: Symmetry,
    difficulty: Option<Difficulty>,
    max_attempts: usize,
}

impl Default for BoardGenerator {
    fn default() -> Self {
        Self {
            num_clues: 30,
            symmetry: Symmetry::None,
            difficulty: None,
            max_attempts: 1,
        }
    }
}

impl BoardGenerator {
    /// Creates a new `BoardGenerator` with default settings (30 clues, no symmetry).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the target number of clues for the generated puzzle.
    pub fn clues(mut self, num_clues: usize) -> Self {
        self.num_clues = num_clues;
        self
    }

    /// Sets the type of symmetry to apply to the generated puzzle.
    pub fn symmetry(mut self, symmetry: Symmetry) -> Self {
        self.symmetry = symmetry;
        self
    }

    /// Sets the target difficulty for the generated puzzle.
    ///
    /// If a difficulty is set, the generator will attempt to find a puzzle
    /// of that exact difficulty by repeatedly generating candidates.
    pub fn difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = Some(difficulty);
        self
    }

    /// Sets the maximum number of attempts when generating a puzzle with a specific difficulty.
    pub fn max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Generates a new Sudoku puzzle based on the current configuration.
    pub fn generate(&self) -> Result<Board, RustokuError> {
        if let Some(target_difficulty) = self.difficulty {
            self.generate_with_difficulty(target_difficulty)
        } else {
            self.generate_single()
        }
    }

    /// Generates a valid Sudoku board with a unique solution and requested symmetry using the dig-hole method.
    fn generate_single(&self) -> Result<Board, RustokuError> {
        if !(17..=81).contains(&self.num_clues) {
            return Err(RustokuError::InvalidClueCount);
        }

        // 1. Generate a complete, randomized, valid 81-cell solved board
        let mut rustoku = Rustoku::new(Board::default())?;
        let solution = rustoku.solve_any().ok_or(RustokuError::DuplicateValues)?;
        let mut board = solution.board;

        // 2. Partition all 81 cells into symmetry groups
        let mut visited = [[false; 9]; 9];
        let mut groups = Vec::new();

        // Iterate in a deterministic but shuffled order to ensure variety
        let mut cells: Vec<(usize, usize)> = board.iter_cells().collect();
        cells.shuffle(&mut rng());

        for (r, c) in cells {
            if !visited[r][c] {
                let partners = self.symmetry.get_partners(r, c);
                for &(pr, pc) in &partners {
                    visited[pr][pc] = true;
                }
                groups.push(partners);
            }
        }

        // Re-shuffle groups to ensure we don't always try to remove the same regions first
        groups.shuffle(&mut rng());

        let mut clues = 81;

        // 3. Iteratively carve out clues while maintaining a unique solution
        for group in groups {
            if clues <= self.num_clues {
                break;
            }

            // Collect clues currently filled in this symmetry group
            let mut group_clues = Vec::new();
            for &(r, c) in &group {
                let val = board.cells[r][c];
                if val != 0 {
                    group_clues.push((r, c, val));
                }
            }

            if group_clues.is_empty() {
                continue;
            }

            // Tentatively clear the entire symmetry group
            for &(r, c, _) in &group_clues {
                board.cells[r][c] = 0;
            }

            // Uniqueness check: bounded solve to verify exactly 1 solution exists
            if Rustoku::new(board)?.solve_until(2).len() != 1 {
                // More than 1 solution found (or 0): restore clues to maintain uniqueness
                for &(r, c, val) in &group_clues {
                    board.cells[r][c] = val;
                }
            } else {
                // Successfully removed clues while maintaining unique solvability
                clues -= group_clues.len();
            }
        }

        // Final safety check to ensure generated board is valid and uniquely solvable
        if Rustoku::new(board)?.solve_until(2).len() != 1 {
            return Err(RustokuError::GenerateFailure);
        }

        Ok(board)
    }

    /// Generates a Sudoku board classified at a specific difficulty level without requiring guessing.
    fn generate_with_difficulty(
        &self,
        target_difficulty: Difficulty,
    ) -> Result<Board, RustokuError> {
        use rand::RngExt;

        for _ in 0..self.max_attempts {
            // Heuristic clue count range tailored to the desired difficulty level
            let clues = match target_difficulty {
                Difficulty::Easy => rand::rng().random_range(34..=42),
                Difficulty::Medium => rand::rng().random_range(28..=34),
                Difficulty::Hard => rand::rng().random_range(22..=28),
                Difficulty::Expert => rand::rng().random_range(17..=22),
            };

            // Generate a uniquely solvable candidate board with symmetry
            let mut sub_generator = *self;
            sub_generator.num_clues = clues;
            sub_generator.difficulty = None; // Avoid recursion

            if let Ok(board) = sub_generator.generate_single() {
                let mut rustoku = Rustoku::builder()
                    .board(board)
                    .techniques(TechniqueFlags::all())
                    .build()?;

                // Evaluate whether human deduction techniques can fully solve the board
                let solutions = rustoku.solve_all();
                if solutions.len() == 1 {
                    let solution = &solutions[0];

                    // Inspect the solve path to evaluate difficulty and verify no guessing occurred
                    let mut max_difficulty = Difficulty::Easy;
                    let mut required_guessing = false;

                    for step in &solution.solve_path.steps {
                        match step {
                            SolveStep::Placement { flags, .. } => {
                                // Placements with empty flags indicate DFS backtracking / guessing
                                if flags.is_empty() {
                                    required_guessing = true;
                                    break;
                                }
                                let step_diff = flags.difficulty();
                                if step_diff > max_difficulty {
                                    max_difficulty = step_diff;
                                }
                            }
                            SolveStep::CandidateElimination { flags, .. } => {
                                if !flags.is_empty() {
                                    let step_diff = flags.difficulty();
                                    if step_diff > max_difficulty {
                                        max_difficulty = step_diff;
                                    }
                                }
                            }
                        }
                    }

                    // Accept board only if solvable purely through logic and matching target difficulty
                    if !required_guessing && max_difficulty == target_difficulty {
                        return Ok(board);
                    }
                }
            }
        }

        Err(RustokuError::GenerateFailure)
    }
}

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

    /// Extracts candidate digits (1–9) from a bitmask into a `Vec<u8>`.
    fn candidates_from_mask(mask: u16) -> Vec<u8> {
        let mut nums = Vec::with_capacity(mask.count_ones() as usize);
        for v in 1..=9u8 {
            if mask & (1 << (v - 1)) != 0 {
                nums.push(v);
            }
        }
        nums
    }

    /// Locates the empty cell with the fewest candidates (MRV heuristic).
    #[inline]
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

    /// Places a digit on the board and propagates constraints forward.
    #[inline]
    fn place_number(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, num);
        self.masks.add_number(r, c, num);
        self.candidates
            .update_affected_cells_for(r, c, &self.masks, &self.board, Some(num));
    }

    /// Clears a digit from the board and recalculates affected candidate sets during backtracking.
    #[inline]
    fn remove_number(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, 0); // Set back to empty
        self.masks.remove_number(r, c, num);
        self.candidates
            .update_affected_cells(r, c, &self.masks, &self.board);
        // Note: `update_affected_cells` will recalculate candidates for the removed cell.
    }

    /// Recursive depth-first backtracking search guided by MRV and forward checking.
    fn solve_until_recursive(
        &mut self,
        solutions: &mut Vec<Solution>,
        path: &mut SolvePath,
        bound: usize,
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
        // Query candidate bitmask for the chosen MRV cell and unpack to candidate numbers
        let mask = self.candidates.get(r, c);
        let mut nums = Self::candidates_from_mask(mask);
        // Shuffle candidates for randomized exploration (useful during puzzle generation)
        nums.shuffle(&mut rng());

        for &num in &nums {
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
            count += self.solve_until_recursive(solutions, path, bound);

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
    fn techniques_make_valid_changes(&mut self, path: &mut SolvePath) -> bool {
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

    /// Finds all possible solutions for the Sudoku puzzle, parallelizing top-level MRV branches.
    ///
    /// Runs deterministic constraint propagation once on the root state, then uses Rayon (`par_iter`)
    /// to explore independent candidate branches of the first MRV cell across CPU worker threads.
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
        use rayon::prelude::*;

        // Phase 1: Run technique propagation once on the current solver state.
        let mut path = SolvePath::default();
        if !self.techniques_make_valid_changes(&mut path) {
            return Vec::new();
        }

        // Phase 2: If empty cells remain, parallelize search across first MRV cell's candidates.
        if let Some((r, c)) = self.find_next_empty_cell() {
            let mask = self.candidates.get(r, c);
            let nums = Self::candidates_from_mask(mask);

            let initial_path = path.clone();

            // Parallelize each top-level candidate branch across worker threads.
            let chunks: Vec<Vec<Solution>> = nums
                .par_iter()
                .map(|&num| {
                    let mut cloned = *self; // Rustoku is Copy/Clone (cheap 1KB copy)
                    let mut local_solutions: Vec<Solution> = Vec::new();
                    let mut local_path = initial_path.clone();

                    // Place the candidate and record the placement in the thread-local path.
                    cloned.place_number(r, c, num);
                    let step_number = local_path.steps.len() as u32;
                    local_path.steps.push(SolveStep::Placement {
                        row: r,
                        col: c,
                        value: num,
                        flags: TechniqueFlags::empty(),
                        step_number,
                        candidates_eliminated: 0,
                        related_cell_count: 0,
                        difficulty_point: 0,
                    });

                    // Continue DFS from this state without re-running the propagator.
                    cloned.solve_until_recursive(&mut local_solutions, &mut local_path, 0);
                    local_solutions
                })
                .collect();

            // Flatten results collected from all parallel branches
            let mut solutions = Vec::new();
            for mut s in chunks {
                solutions.append(&mut s);
            }
            solutions
        } else {
            // Already solved after propagation
            vec![Solution {
                board: self.board,
                solve_path: path,
            }]
        }
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

/// Lazy iterator wrapper for solutions. Uses an explicit DFS stack and yields
/// solutions one-by-one without computing them all up-front.
///
/// Unlike recursive search, `Solutions` maintains its state in an explicit heap-allocated
/// stack of [`Frame`] structures. This allows caller-driven, memory-bounded solution streaming.
#[derive(Debug)]
pub struct Solutions {
    solver: Rustoku,
    path: SolvePath,
    stack: Vec<Frame>,
    finished: bool,
}

/// Represents an active level in the explicit depth-first backtracking search stack.
#[derive(Debug)]
struct Frame {
    /// Row index of the cell being decided at this search depth.
    r: usize,
    /// Column index of the cell being decided at this search depth.
    c: usize,
    /// Remaining candidate digits available for cell `(r, c)`.
    nums: Vec<u8>,
    /// Index into `nums` pointing to the next candidate to evaluate.
    idx: usize,
    /// Digit currently placed on the board by this frame, if any.
    placed: Option<u8>,
}

impl Solutions {
    /// Construct a `Solutions` iterator from an existing `Rustoku` solver.
    /// This will run the technique propagator once before starting DFS.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::{Rustoku, Solutions};
    ///
    /// let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    /// let solver = Rustoku::new_from_str(puzzle).unwrap();
    /// let mut solutions = Solutions::from_solver(solver);
    /// assert!(solutions.next().is_some());
    /// ```
    pub fn from_solver(mut solver: Rustoku) -> Self {
        let mut path = SolvePath::default();
        let mut finished = false;

        // 1. Initial Constraint Propagation Pass
        // Run deterministic techniques once. If a contradiction is detected, terminate immediately.
        if !solver.techniques_make_valid_changes(&mut path) {
            finished = true;
        }

        let mut stack = Vec::new();
        if !finished {
            // 2. Initialize the Root Frame
            // Find the most constrained variable (MRV) on the board to form the root of the DFS stack.
            if let Some((r, c)) = solver.find_next_empty_cell() {
                let mask = solver.candidates.get(r, c);
                let mut nums = Rustoku::candidates_from_mask(mask);
                nums.shuffle(&mut rng());
                stack.push(Frame {
                    r,
                    c,
                    nums,
                    idx: 0,
                    placed: None,
                });
            } else {
                // Puzzle is already fully solved after deterministic propagation alone;
                // leave stack empty and let next() yield the board on the first call.
            }
        }

        Solutions {
            solver,
            path,
            stack,
            finished,
        }
    }
}

impl Iterator for Solutions {
    type Item = Solution;

    /// Advances the explicit DFS stack to discover and yield the next valid solution.
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            // 1. Stack Empty State: Check whether the board is already solved or needs a root frame
            if self.stack.is_empty() {
                if let Some((r, c)) = self.solver.find_next_empty_cell() {
                    let mask = self.solver.candidates.get(r, c);
                    let mut nums = Rustoku::candidates_from_mask(mask);
                    nums.shuffle(&mut rng());
                    self.stack.push(Frame {
                        r,
                        c,
                        nums,
                        idx: 0,
                        placed: None,
                    });
                    continue;
                } else {
                    // No empty cells remain -> current board configuration is a valid solution
                    let sol = Solution {
                        board: self.solver.board,
                        solve_path: self.path.clone(),
                    };
                    self.finished = true;
                    return Some(sol);
                }
            }

            let last_idx = self.stack.len() - 1;
            let frame = &mut self.stack[last_idx];

            // 2. Candidate Exhaustion & Backtracking
            // If all candidate digits at this frame have been evaluated:
            if frame.idx >= frame.nums.len() {
                if let Some(num) = frame.placed {
                    // Backtrack the active placement made by this frame
                    self.solver.remove_number(frame.r, frame.c, num);
                    self.path.steps.pop();
                    frame.placed = None;
                } else {
                    // All candidates tried and placement undone: pop frame to backtrack to parent
                    self.stack.pop();
                }
                continue;
            }

            // 3. Evaluate Next Candidate Digit
            let num = frame.nums[frame.idx];
            frame.idx += 1;

            if self.solver.masks.is_safe(frame.r, frame.c, num) {
                // Forward move: place digit and record placement step
                self.solver.place_number(frame.r, frame.c, num);
                let step_number = self.path.steps.len() as u32;
                self.path.steps.push(SolveStep::Placement {
                    row: frame.r,
                    col: frame.c,
                    value: num,
                    flags: TechniqueFlags::empty(),
                    step_number,
                    candidates_eliminated: 0,
                    related_cell_count: 0,
                    difficulty_point: 0,
                });
                frame.placed = Some(num);

                // 4. Branch Descent or Goal Reached
                if let Some((nr, nc)) = self.solver.find_next_empty_cell() {
                    // Empty cells remain: push new child frame for the next MRV cell and descend
                    let mask = self.solver.candidates.get(nr, nc);
                    let mut nums2 = Rustoku::candidates_from_mask(mask);
                    nums2.shuffle(&mut rng());
                    self.stack.push(Frame {
                        r: nr,
                        c: nc,
                        nums: nums2,
                        idx: 0,
                        placed: None,
                    });
                    continue;
                } else {
                    // Goal reached: all 81 cells filled. Capture the solution.
                    let solution = Solution {
                        board: self.solver.board,
                        solve_path: self.path.clone(),
                    };
                    // Backtrack the placement just made on this frame so subsequent next() calls
                    // can cleanly resume exploring alternative branches
                    if let Some(pnum) = frame.placed {
                        self.solver.remove_number(frame.r, frame.c, pnum);
                        self.path.steps.pop();
                        frame.placed = None;
                    }
                    return Some(solution);
                }
            }
            // else candidate was not safe; try next candidate in this frame
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
/// This is a convenience shim for `BoardGenerator::new().clues(num_clues).generate()`.
///
/// # Examples
///
/// ```
/// use rustoku_lib::generate_board;
///
/// let puzzle = generate_board(30);
/// assert!(puzzle.is_ok());
/// ```
pub fn generate_board(num_clues: usize) -> Result<Board, RustokuError> {
    BoardGenerator::new().clues(num_clues).generate()
}

/// Generates a new Sudoku puzzle that matches a specific difficulty level.
///
/// This is a convenience shim for `BoardGenerator::new().difficulty(difficulty).max_attempts(max_attempts).generate()`.
///
/// # Examples
///
/// ```
/// use rustoku_lib::{Difficulty, generate_board_by_difficulty};
///
/// let board = generate_board_by_difficulty(Difficulty::Easy, 100);
/// assert!(board.is_ok());
/// ```
pub fn generate_board_by_difficulty(
    difficulty: Difficulty,
    max_attempts: usize,
) -> Result<Board, RustokuError> {
    BoardGenerator::new()
        .difficulty(difficulty)
        .max_attempts(max_attempts)
        .generate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::board::Board;
    use crate::core::techniques::flags::Difficulty;
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
                .expect("Board generation failed - check clue count is between 17 and 81");
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

    #[test]
    fn test_generate_by_difficulty_easy() {
        let board = generate_board_by_difficulty(Difficulty::Easy, 100)
            .expect("Failed to generate an Easy puzzle within 100 attempts");

        let mut rustoku = Rustoku::builder()
            .board(board)
            .techniques(TechniqueFlags::all())
            .build()
            .unwrap();

        let solutions = rustoku.solve_all();
        assert_eq!(solutions.len(), 1);

        let mut required_guessing = false;
        let mut max_difficulty = Difficulty::Easy;

        for step in &solutions[0].solve_path.steps {
            match step {
                crate::core::solution::SolveStep::Placement { flags, .. } => {
                    if flags.is_empty() {
                        required_guessing = true;
                    }
                    if flags.difficulty() > max_difficulty {
                        max_difficulty = flags.difficulty();
                    }
                }
                crate::core::solution::SolveStep::CandidateElimination { flags, .. } => {
                    if flags.difficulty() > max_difficulty {
                        max_difficulty = flags.difficulty();
                    }
                }
            }
        }

        assert!(
            !required_guessing,
            "Easy puzzle should not require guessing"
        );
        assert_eq!(
            max_difficulty,
            Difficulty::Easy,
            "Puzzle exceeded target difficulty"
        );
    }

    #[test]
    fn test_generate_by_difficulty_hard() {
        let board = generate_board_by_difficulty(Difficulty::Hard, 1000)
            .expect("Failed to generate a Hard puzzle within 1000 attempts");

        let mut rustoku = Rustoku::builder()
            .board(board)
            .techniques(TechniqueFlags::all())
            .build()
            .unwrap();

        let solutions = rustoku.solve_all();
        assert_eq!(solutions.len(), 1);

        let mut required_guessing = false;
        let mut max_difficulty = Difficulty::Easy;

        for step in &solutions[0].solve_path.steps {
            match step {
                crate::core::solution::SolveStep::Placement { flags, .. } => {
                    if flags.is_empty() {
                        required_guessing = true;
                    }
                    if flags.difficulty() > max_difficulty {
                        max_difficulty = flags.difficulty();
                    }
                }
                crate::core::solution::SolveStep::CandidateElimination { flags, .. } => {
                    if flags.difficulty() > max_difficulty {
                        max_difficulty = flags.difficulty();
                    }
                }
            }
        }

        assert!(
            !required_guessing,
            "Hard puzzle should not require guessing"
        );
        assert_eq!(
            max_difficulty,
            Difficulty::Hard,
            "Puzzle exceeded target difficulty"
        );
    }
}
