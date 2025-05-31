use crate::solution::RustokuSolution;
use rand::rng;
use rand::seq::SliceRandom;

use super::Rustoku;

/// Solver methods for Sudoku puzzles.
impl Rustoku {
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
}

/// Validation methods for Sudoku puzzles.
impl Rustoku {
    /// Checks if the Sudoku puzzle is solved correctly.
    ///
    /// A puzzle is solved if all cells are filled and the board does not
    /// contain duplicates across rows, columns, and 3x3 boxes.
    pub fn is_solved(&self) -> bool {
        if self.board.iter().flatten().any(|&val| val == 0) {
            return false;
        }
        Rustoku::new(self.board).is_ok()
    }
}
