use crate::core::{SolvePath, SolveStep};

use super::board::Board;
use super::candidates::Candidates;
use super::masks::Masks;

pub mod flags;
mod hidden_pairs;
mod hidden_singles;
mod locked_candidates;
mod naked_pairs;
mod naked_singles;
pub mod units;
mod x_wing;

use flags::TechniqueFlags;
use hidden_pairs::HiddenPairs;
use hidden_singles::HiddenSingles;
use locked_candidates::LockedCandidates;
use naked_pairs::NakedPairs;
use naked_singles::NakedSingles;
use x_wing::XWing;

/// Propagates constraints via zero or more techniques.
///
/// The techniques are toggled via bitflags. Most of the data in struct comes
/// from the Rustoku instance, which has a longer lifetime than this struct - since
/// it is only used at the start, before any backtracking occurs.
///
/// Some examples of techniques employed including Naked Singles and X-Wings.
/// If we want to add more techniques, extend the existing logic and bitflags
/// in this module.
///
/// This class acts as the Mediator object between `Rustoku` and the `TechniqueRule`
/// implementations out there. To learn about the Mediator design pattern, please
/// consult [this link](https://refactoring.guru/design-patterns/mediator)
/// for more details.
pub struct TechniquePropagator<'a> {
    board: &'a mut Board,
    masks: &'a mut Masks,
    candidates: &'a mut Candidates,
    techniques_enabled: TechniqueFlags,
}

impl<'a> TechniquePropagator<'a> {
    pub fn new(
        board: &'a mut Board,
        masks: &'a mut Masks,
        candidates: &'a mut Candidates,
        techniques_enabled: TechniqueFlags,
    ) -> Self {
        Self {
            board,
            masks,
            candidates,
            techniques_enabled,
        }
    }

    /// Helper to place a number and update caches.
    fn place_and_update(
        &mut self,
        r: usize,
        c: usize,
        num: u8,
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) {
        self.board.set(r, c, num);
        self.masks.add_number(r, c, num);

        // Count candidates eliminated by this placement
        let affected_cells_count = self.count_affected_cells(r, c, num);
        let candidates_eliminated_count = self.count_candidates_eliminated(r, c, num);

        self.candidates
            .update_affected_cells(r, c, self.masks, self.board);

        let step_number = path.steps.len() as u32;
        let difficulty_point = Self::difficulty_for_technique(flags);

        path.steps.push(SolveStep::Placement {
            row: r,
            col: c,
            value: num,
            flags,
            step_number,
            candidates_eliminated: candidates_eliminated_count,
            related_cell_count: affected_cells_count.min(255) as u8,
            difficulty_point,
        });
    }

    /// Helper to remove a number and update caches.
    fn remove_and_update(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, 0);
        self.masks.remove_number(r, c, num);
        self.candidates
            .update_affected_cells(r, c, self.masks, self.board);
        // Note: For propagation, `remove_number` is mostly for backtracking, not direct technique application.
        // The `update_affected_cells` on removal will recalculate candidates for the now-empty cell.
    }

    /// Helper to eliminate a candidate and update caches.
    fn eliminate_candidate(
        &mut self,
        r: usize,
        c: usize,
        candidate_bit: u16, // Assume only one candidate is being eliminated
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let initial_mask = self.candidates.get(r, c);
        let refined_mask = initial_mask & !candidate_bit;
        self.candidates.set(r, c, refined_mask);

        let num = candidate_bit.trailing_zeros() as u8 + 1; // Convert bit to number
        let step_number = path.steps.len() as u32;
        let difficulty_point = Self::difficulty_for_technique(flags);

        path.steps.push(SolveStep::CandidateElimination {
            row: r,
            col: c,
            value: num,
            flags,
            step_number,
            candidates_eliminated: 1, // Single candidate was eliminated
            related_cell_count: 1,    // At minimum, this cell is affected
            difficulty_point,
        });

        initial_mask != refined_mask // Return true if a candidate was eliminated
    }

    /// Helper to eliminate multiple candidates and update caches
    fn eliminate_multiple_candidates(
        &mut self,
        r: usize,
        c: usize,
        elimination_mask: u16, // bits to eliminate
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let initial_mask = self.candidates.get(r, c);
        let refined_mask = initial_mask & !elimination_mask;
        self.candidates.set(r, c, refined_mask);

        // Log each eliminated candidate
        let eliminated_mask = initial_mask & elimination_mask; // what was actually eliminated
        let eliminated_count = eliminated_mask.count_ones();
        let difficulty_point = Self::difficulty_for_technique(flags);

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);
            if (eliminated_mask & candidate_bit) != 0 {
                let step_number = path.steps.len() as u32;
                path.steps.push(SolveStep::CandidateElimination {
                    row: r,
                    col: c,
                    value: candidate,
                    flags,
                    step_number,
                    candidates_eliminated: eliminated_count,
                    related_cell_count: 1,
                    difficulty_point,
                });
            }
        }

        initial_mask != refined_mask // Return true if a candidate was eliminated
    }

    /// Counts affected cells when placing a number (cells in same row, column, or box).
    /// Deduplicates cells that appear in multiple units (row+box or col+box overlap).
    fn count_affected_cells(&self, r: usize, c: usize, _num: u8) -> u32 {
        let mut count = 0u32;
        let box_r = (r / 3) * 3;
        let box_c = (c / 3) * 3;

        // Count cells in the same row
        for col in 0..9 {
            if col != c && self.board.is_empty(r, col) {
                count += 1;
            }
        }

        // Count cells in the same column
        for row in 0..9 {
            if row != r && self.board.is_empty(row, c) {
                count += 1;
            }
        }

        // Count cells in the same 3x3 box, excluding those already counted in the row or column
        for br in box_r..box_r + 3 {
            for bc in box_c..box_c + 3 {
                if br != r && bc != c && self.board.is_empty(br, bc) {
                    count += 1;
                }
            }
        }

        count
    }

    /// Counts the number of candidates that would be eliminated by a placement.
    /// Deduplicates cells that appear in multiple units (row+box or col+box overlap).
    fn count_candidates_eliminated(&self, r: usize, c: usize, num: u8) -> u32 {
        let mut count = 0u32;
        let box_r = (r / 3) * 3;
        let box_c = (c / 3) * 3;
        let candidate_bit = 1u16 << (num - 1);

        // Count in the same row
        for col in 0..9 {
            if col != c && (self.candidates.get(r, col) & candidate_bit) != 0 {
                count += 1;
            }
        }

        // Count in the same column
        for row in 0..9 {
            if row != r && (self.candidates.get(row, c) & candidate_bit) != 0 {
                count += 1;
            }
        }

        // Count in the same 3x3 box, excluding those already counted in the row or column
        for br in box_r..box_r + 3 {
            for bc in box_c..box_c + 3 {
                if br != r && bc != c && (self.candidates.get(br, bc) & candidate_bit) != 0 {
                    count += 1;
                }
            }
        }

        count
    }

    /// Returns a difficulty metric for a given technique.
    fn difficulty_for_technique(flags: TechniqueFlags) -> u8 {
        if flags.is_empty() {
            0
        } else {
            flags.bits().trailing_zeros() as u8 + 1
        }
    }

    /// Applies deterministic constraint propagation techniques iteratively.
    pub fn propagate_constraints(&mut self, path: &mut SolvePath, initial_path_len: usize) -> bool {
        let techniques: Vec<&dyn TechniqueRule> = vec![
            &NakedSingles,
            &HiddenSingles,
            &NakedPairs,
            &HiddenPairs,
            &LockedCandidates,
            &XWing,
        ];

        loop {
            let mut changed_this_iter = false;

            for technique in &techniques {
                if self.techniques_enabled.contains(technique.flags()) {
                    // Pass the propagator itself and the current path to the technique. This
                    // is an example of the Mediator pattern, where the propagator
                    // mediates the interaction between the techniques and the board
                    changed_this_iter |= technique.apply(self, path);
                    if changed_this_iter {
                        break;
                    }
                }
            }

            if (0..9).any(|r| {
                (0..9).any(|c| self.board.is_empty(r, c) && self.candidates.get(r, c) == 0)
            }) {
                while path.steps.len() > initial_path_len {
                    if let Some(step) = path.steps.pop() {
                        match step {
                            SolveStep::Placement {
                                row,
                                col,
                                value,
                                flags: _,
                                ..
                            } => {
                                self.remove_and_update(row, col, value);
                            }
                            SolveStep::CandidateElimination {
                                row,
                                col,
                                value,
                                flags: _,
                                ..
                            } => {
                                // This is a candidate elimination step, we need to restore the candidate
                                // in the candidates cache
                                let initial_mask = self.candidates.get(row, col);
                                let refined_mask = initial_mask | (1 << (value - 1));
                                self.candidates.set(row, col, refined_mask);
                            }
                        }
                    }
                }
                return false;
            }

            if !changed_this_iter {
                break;
            }
        }
        true
    }
}

/// This is the contract for all human techniques.
///
/// All techniques are expected to have a way to apply themselves to a board
/// and modify the solve path with placements and eliminations. In addition, they
/// are expected to return one flag that helps with technique attribution when
/// people want to visualize the solve path.
pub trait TechniqueRule {
    /// Applies the technique to the given propagator.
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool;

    /// Returns the flags associated with this technique.
    fn flags(&self) -> TechniqueFlags;
}
