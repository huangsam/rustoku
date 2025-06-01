use super::board::RustokuBoard;
use super::candidates::RustokuCandidates;
use super::masks::RustokuMasks;
use super::solution::RustokuSolution;
use super::techniques::{RustokuTechniques, TechniquePropagator};
use crate::error::RustokuError;
use rand::prelude::SliceRandom;
use rand::rng;

/// A Sudoku primitive that uses backtracking and bitmasking for constraints.
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
/// use rustoku::core::{Rustoku, RustokuBoard};
/// let s = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
/// let board = RustokuBoard::try_from(s).unwrap();
/// let mut rustoku = Rustoku::new(board).unwrap();
/// assert!(rustoku.solve_any().is_some());
/// ```
///
/// Generate a Sudoku puzzle:
/// ```
/// use rustoku::core::{Rustoku, generate_board};
/// let board = generate_board(30).unwrap();
/// let solution = Rustoku::new(board).unwrap().solve_all();
/// assert_eq!(solution.len(), 1);
/// ```
///
/// Check if a Sudoku puzzle is solved:
/// ```
/// use rustoku::core::{Rustoku, RustokuBoard};
/// let s = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
/// let board = RustokuBoard::try_from(s).unwrap();
/// let rustoku = Rustoku::new(board).unwrap();
/// assert!(rustoku.is_solved());
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Rustoku {
    pub board: RustokuBoard,
    masks: RustokuMasks,
    candidates_cache: RustokuCandidates,
    techniques: RustokuTechniques,
}

impl Rustoku {
    /// Constructs a new `Rustoku` instance from an initial `Board`.
    pub fn new(initial_board: RustokuBoard) -> Result<Self, RustokuError> {
        let board = initial_board; // Now takes a Board directly
        let mut masks = RustokuMasks::new();
        let mut candidates_cache = RustokuCandidates::new();

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
                    candidates_cache.set(r, c, masks.compute_candidates_mask_for_cell(r, c));
                }
            }
        }

        Ok(Self {
            board,
            masks,
            candidates_cache,
            techniques: RustokuTechniques::SINGLES, // Default
        })
    }

    /// Constructs a new `Rustoku` instance from a string representation of the board.
    pub fn new_from_str(s: &str) -> Result<Self, RustokuError> {
        let board = RustokuBoard::try_from(s)?;
        Self::new(board)
    }

    /// Returns the existing Rustoku instance, with modified techniques.
    pub fn with_techniques(mut self, techniques: RustokuTechniques) -> Self {
        self.techniques = techniques;
        self
    }

    /// Helper for solver to find the next empty cell (MRV).
    fn find_next_empty_cell(&self) -> Option<(usize, usize)> {
        let mut min = (10, None); // Min candidates, (r, c)
        for (r, c) in self.board.iter_empty_cells() {
            let count = self.candidates_cache.get(r, c).count_ones() as u8;
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
        self.candidates_cache
            .update_affected_cells(r, c, &self.masks, &self.board);
    }

    /// Remove a number from the board and update masks and candidates.
    fn remove_number(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, 0); // Set back to empty
        self.masks.remove_number(r, c, num);
        self.candidates_cache
            .update_affected_cells(r, c, &self.masks, &self.board);
        // Note: `update_affected_cells` will recalculate candidates for the removed cell.
    }

    /// Recursive function to solve the Sudoku puzzle with backtracking.
    fn solve_until_recursive(
        &mut self,
        solutions: &mut Vec<RustokuSolution>,
        path: &mut Vec<(usize, usize, u8)>,
        bound: usize,
    ) -> usize {
        if let Some((r, c)) = self.find_next_empty_cell() {
            let mut count = 0;
            let mut nums: Vec<u8> = (1..=9).collect();
            nums.shuffle(&mut rng());

            for &num in &nums {
                if self.masks.is_safe(r, c, num) {
                    self.place_number(r, c, num);
                    path.push((r, c, num));
                    count += self.solve_until_recursive(solutions, path, bound);
                    path.pop();
                    self.remove_number(r, c, num);

                    if bound > 0 && solutions.len() >= bound {
                        return count;
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
        }
    }

    /// Solves the Sudoku puzzle up to a certain bound, returning solutions with their solve paths.
    pub fn solve_until(&mut self, bound: usize) -> Vec<RustokuSolution> {
        let mut solutions = Vec::new();
        let mut path = Vec::new();

        let mut propagator = TechniquePropagator::new(
            &mut self.board,
            &mut self.masks,
            &mut self.candidates_cache,
            self.techniques,
        );

        if !propagator.propagate_constraints(&mut path, 0) {
            return solutions; // Early exit if initial constraints are inconsistent
        }

        self.solve_until_recursive(&mut solutions, &mut path, bound);
        solutions
    }

    /// Attempts to solve the Sudoku puzzle using backtracking with MRV (Minimum Remaining Values).
    pub fn solve_any(&mut self) -> Option<RustokuSolution> {
        self.solve_until(1).into_iter().next()
    }

    /// Finds all possible solutions for the Sudoku puzzle.
    pub fn solve_all(&mut self) -> Vec<RustokuSolution> {
        self.solve_until(0)
    }

    /// Checks if the Sudoku puzzle is solved correctly.
    pub fn is_solved(&self) -> bool {
        self.board.cells.iter().flatten().all(|&val| val != 0) && Rustoku::new(self.board).is_ok()
    }
}
