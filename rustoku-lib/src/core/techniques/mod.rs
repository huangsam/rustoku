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
mod x_wing;

use flags::TechniqueFlags;
use hidden_pairs::HiddenPairs;
use hidden_singles::HiddenSingles;
use locked_candidates::LockedCandidates;
use naked_pairs::NakedPairs;
use naked_singles::NakedSingles;
use x_wing::XWing;

// Now the actual implementation of the techniques, these would operate on
// references to Board, Masks, and CandidatesCache.
pub struct TechniquePropagator<'a> {
    board: &'a mut Board,
    masks: &'a mut Masks,
    candidates_cache: &'a mut Candidates,
    techniques_enabled: TechniqueFlags,
}

impl<'a> TechniquePropagator<'a> {
    pub fn new(
        board: &'a mut Board,
        masks: &'a mut Masks,
        candidates_cache: &'a mut Candidates,
        techniques_enabled: TechniqueFlags,
    ) -> Self {
        Self {
            board,
            masks,
            candidates_cache,
            techniques_enabled,
        }
    }

    /// Helper to place a number and update caches.
    pub(super) fn place_and_update(
        &mut self,
        r: usize,
        c: usize,
        num: u8,
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) {
        self.board.set(r, c, num);
        self.masks.add_number(r, c, num);
        self.candidates_cache
            .update_affected_cells(r, c, self.masks, self.board);
        path.steps
            .push(SolveStep::new(r, c, num).with_flags(flags));
    }

    /// Helper to remove a number and update caches.
    fn remove_and_update(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, 0);
        self.masks.remove_number(r, c, num);
        self.candidates_cache
            .update_affected_cells(r, c, self.masks, self.board);
        // Note: For propagation, `remove_number` is mostly for backtracking, not direct technique application.
        // The `update_affected_cells` on removal will recalculate candidates for the now-empty cell.
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
                (0..9).any(|c| self.board.is_empty(r, c) && self.candidates_cache.get(r, c) == 0)
            }) {
                while path.steps.len() > initial_path_len {
                    if let Some(step) = path.steps.pop() {
                        self.remove_and_update(step.row, step.col, step.value);
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

pub trait TechniqueRule {
    /// Applies the technique to the given propagator.
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool;

    /// Returns the flags associated with this technique.
    fn flags(&self) -> TechniqueFlags;
}
