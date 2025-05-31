use super::board::RustokuBoard;
use super::candidates::RustokuCandidates;
use super::masks::RustokuMasks;
use super::solution::RustokuSolution;
use super::techniques::{RustokuTechniques, TechniquePropagator};
use crate::error::RustokuError;
use rand::prelude::SliceRandom;
use rand::rng;

#[derive(Debug, Copy, Clone)]
pub struct Rustoku {
    pub board: RustokuBoard,
    masks: RustokuMasks,
    candidates_cache: RustokuCandidates,
    techniques: RustokuTechniques,
}

impl TryFrom<[u8; 81]> for Rustoku {
    type Error = RustokuError;

    fn try_from(bytes: [u8; 81]) -> Result<Self, Self::Error> {
        let mut board_array = [[0u8; 9]; 9];
        for i in 0..81 {
            board_array[i / 9][i % 9] = bytes[i];
        }
        Self::new(board_array)
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
    pub fn new(initial_board_array: [[u8; 9]; 9]) -> Result<Self, RustokuError> {
        let board = RustokuBoard::new(initial_board_array);
        let mut masks = RustokuMasks::new();
        let mut candidates_cache = RustokuCandidates::new();

        // Initialize masks and check for duplicates
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

        // Initialize the candidates cache for empty cells based on initial masks
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
            techniques: RustokuTechniques::SIMPLE, // Default
        })
    }

    pub fn with_techniques(mut self, techniques: RustokuTechniques) -> Self {
        self.techniques = techniques;
        self
    }

    // Helper for solver to find the next empty cell (MRV)
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

    // Place and remove operations for the solver, updated to use the new structs
    fn place_number(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, num);
        self.masks.add_number(r, c, num);
        self.candidates_cache
            .update_affected_cells(r, c, &self.masks, &self.board);
    }

    fn remove_number(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, 0); // Set back to empty
        self.masks.remove_number(r, c, num);
        self.candidates_cache
            .update_affected_cells(r, c, &self.masks, &self.board);
        // Note: `update_affected_cells` will recalculate candidates for the removed cell.
    }

    // Solver logic
    fn solve_until_recursive(
        &mut self,
        solutions: &mut Vec<RustokuSolution>,
        path: &mut Vec<(usize, usize, u8)>,
        bound: usize,
    ) -> usize {
        let path_len_before = path.len();

        let mut propagator = TechniquePropagator::new(
            &mut self.board,
            &mut self.masks,
            &mut self.candidates_cache,
            self.techniques,
        );

        if !propagator.propagate_constraints(path, path_len_before) {
            // Contradiction detected during propagation, no solution down this path
            return 0;
        }

        let result = if let Some((r, c)) = self.find_next_empty_cell() {
            let mut count = 0;
            let mut nums: Vec<u8> = (1..=9).collect();
            // Use rand::thread_rng() for shuffling
            nums.shuffle(&mut rng());

            for &num in &nums {
                if self.masks.is_safe(r, c, num) {
                    self.place_number(r, c, num);
                    path.push((r, c, num)); // Add to path for this branch
                    count += self.solve_until_recursive(solutions, path, bound);
                    path.pop(); // Remove from path on backtrack
                    self.remove_number(r, c, num);

                    if bound > 0 && solutions.len() >= bound {
                        break;
                    }
                }
            }
            count
        } else {
            // Board is filled and consistent (due to propagation and `is_safe` checks)
            solutions.push(RustokuSolution {
                board: self.board, // Access the internal array
                solve_path: path.clone(),
            });
            1
        };

        // Backtrack placements made by propagation in this call frame
        while path.len() > path_len_before {
            let (r, c, num) = path.pop().unwrap();
            self.remove_number(r, c, num);
        }

        result
    }

    pub fn solve_until(&mut self, bound: usize) -> Vec<RustokuSolution> {
        let mut solutions = Vec::new();
        let mut path = Vec::new();
        self.solve_until_recursive(&mut solutions, &mut path, bound);
        solutions
    }

    pub fn solve_any(&mut self) -> Option<RustokuSolution> {
        self.solve_until(1).into_iter().next()
    }

    pub fn solve_all(&mut self) -> Vec<RustokuSolution> {
        self.solve_until(0)
    }

    pub fn is_solved(&self) -> bool {
        self.board.cells.iter().flatten().all(|&val| val != 0)
            && Rustoku::new(self.board.cells).is_ok()
    }
}
