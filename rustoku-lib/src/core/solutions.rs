//! Lazy streaming solution iterator for Sudoku puzzles.
//!
//! Provides [`Solutions`], which uses an explicit depth-first search stack
//! to yield solutions one by one on demand without computing them all up front.

use rand::prelude::SliceRandom;
use rand::rng;

use super::solution::{Solution, SolvePath, SolveStep};
use super::solver::Rustoku;
use super::techniques::flags::TechniqueFlags;

/// Lazy iterator wrapper for solutions. Uses an explicit DFS stack and yields
/// solutions one-by-one without computing them all up-front.
///
/// Unlike recursive search, `Solutions` maintains its state in an explicit heap-allocated
/// stack of `Frame` structures. This allows caller-driven, memory-bounded solution streaming.
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

            // 2. Undo any previous placement at this frame before proceeding
            if let Some(num) = frame.placed.take() {
                self.solver.remove_number(frame.r, frame.c, num);
                self.path.steps.pop();
            }

            // 3. Candidate Exhaustion & Backtracking
            if frame.idx >= frame.nums.len() {
                self.stack.pop();
                if self.stack.is_empty() {
                    self.finished = true;
                    return None;
                }
                continue;
            }

            // 4. Evaluate Next Candidate Digit
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

                // 5. Branch Descent or Goal Reached
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
                    return Some(solution);
                }
            }
            // else candidate was not safe; try next candidate in this frame
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::board::Board;
    use crate::format::format_line;

    const UNIQUE_PUZZLE: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const UNIQUE_SOLUTION: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
    const TWO_PUZZLE: &str =
        "295743861431865900876192543387459216612387495549216738763504189928671354154938600";

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
    fn test_solutions_iterator_two_solutions() {
        let solver = Rustoku::new_from_str(TWO_PUZZLE).expect("valid puzzle");
        let mut sols = Solutions::from_solver(solver);

        let first = sols.next();
        assert!(first.is_some());
        let second = sols.next();
        assert!(second.is_some());
        assert_ne!(first.unwrap().board, second.unwrap().board);
        assert!(sols.next().is_none());
    }

    #[test]
    fn test_solutions_iterator_unsolvable() {
        let s = "078002609030008020002000083000000040043090000007300090200001036001840902050003007";
        let solver = Rustoku::new_from_str(s).expect("valid board format");
        let mut sols = Solutions::from_solver(solver);
        assert!(sols.next().is_none());
    }

    #[test]
    fn test_solutions_iterator_already_solved() {
        let solver = Rustoku::new_from_str(UNIQUE_SOLUTION).expect("valid puzzle");
        let mut sols = Solutions::from_solver(solver);

        let sol = sols.next();
        assert!(sol.is_some());
        assert_eq!(format_line(&sol.unwrap().board), UNIQUE_SOLUTION);
        assert!(sols.next().is_none());
    }
}
