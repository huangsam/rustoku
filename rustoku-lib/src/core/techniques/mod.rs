use crate::core::{SolvePath, SolveStep};

use super::board::Board;
use super::candidates::Candidates;
use super::masks::Masks;

mod aic;
pub mod flags;
mod hidden_pairs;
mod hidden_quads;
mod hidden_singles;
mod hidden_triples;
mod jellyfish;
mod locked_candidates;
mod naked_pairs;
mod naked_quads;
mod naked_singles;
mod naked_triples;
mod skyscraper;
mod swordfish;
pub mod units;
mod w_wing;
mod x_wing;
mod xy_wing;
mod xyz_wing;

use aic::AlternatingInferenceChain;
use flags::TechniqueFlags;
use hidden_pairs::HiddenPairs;
use hidden_quads::HiddenQuads;
use hidden_singles::HiddenSingles;
use hidden_triples::HiddenTriples;
use jellyfish::Jellyfish;
use locked_candidates::LockedCandidates;
use naked_pairs::NakedPairs;
use naked_quads::NakedQuads;
use naked_singles::NakedSingles;
use naked_triples::NakedTriples;
use skyscraper::Skyscraper;
use swordfish::Swordfish;
use w_wing::WWing;
use x_wing::XWing;
use xy_wing::XYWing;
use xyz_wing::XyzWing;

/// Propagates constraints via zero or more techniques.
///
/// Acts as the Mediator between [`Rustoku`] and individual [`TechniqueRule`] implementations,
/// coordinating updates across [`Board`], [`Masks`], and [`Candidates`] while ensuring individual
/// techniques do not mutate board state directly (see [Mediator pattern](https://refactoring.guru/design-patterns/mediator)).
///
/// ### Mediator Architecture & Invariants
/// - **Synchronized State**: Placing digits or eliminating candidates updates [`Board`],
///   [`Masks`], and [`Candidates`] in lockstep.
/// - **Telemetry**: Records step metadata ([`SolveStep`]) with candidate and peer counts.
/// - **Transactional Rollback**: If a contradiction occurs (an empty cell with zero candidates),
///   all changes applied during the pass are unwound to restore the previous state.
pub struct TechniquePropagator<'a> {
    board: &'a mut Board,
    masks: &'a mut Masks,
    candidates: &'a mut Candidates,
    techniques_enabled: TechniqueFlags,
}

impl<'a> TechniquePropagator<'a> {
    /// Creates a new `TechniquePropagator` mediator over the solver's working state.
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

    /// Places a number on the board, updates masks and peer candidates, and records the step.
    fn place_and_update(
        &mut self,
        r: usize,
        c: usize,
        num: u8,
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) {
        // Step 1 & 2: Update board cell and constraint masks
        self.board.set(r, c, num);
        self.masks.add_number(r, c, num);

        // Step 3: Compute telemetry for step logging before candidate cache is updated
        let affected_cells_count = self.count_affected_cells(r, c, num);
        let candidates_eliminated_count = self.count_candidates_eliminated(r, c, num);

        // Step 4: Propagate constraint to peer cells (removing `num` from their candidate bitmasks)
        self.candidates
            .update_affected_cells(r, c, self.masks, self.board);

        // Step 5: Record placement in the solve path
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

    /// Reverses a number placement and restores peer candidates during propagation rollback.
    fn remove_and_update(&mut self, r: usize, c: usize, num: u8) {
        // Reset board cell to empty
        self.board.set(r, c, 0);
        // Clear digit bit from row, column, and box constraint masks
        self.masks.remove_number(r, c, num);
        // Recalculate candidates for peer cells as well as the newly-emptied cell (r, c)
        self.candidates
            .update_affected_cells(r, c, self.masks, self.board);
    }

    /// Eliminates a single candidate digit from cell `(r, c)` and records the step.
    fn eliminate_candidate(
        &mut self,
        r: usize,
        c: usize,
        candidate_bit: u16, // Assume only one candidate is being eliminated (e.g. 1 << (v - 1))
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let initial_mask = self.candidates.get(r, c);
        // Clear the specific candidate bit using bitwise AND-NOT
        let refined_mask = initial_mask & !candidate_bit;
        self.candidates.set(r, c, refined_mask);

        // Convert the candidate bit back to the digit (1-9) via 0-indexed trailing zeros + 1
        let num = candidate_bit.trailing_zeros() as u8 + 1;
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

        // Return true if the mask actually changed
        initial_mask != refined_mask
    }

    /// Eliminates multiple candidate digits simultaneously from cell `(r, c)` and records each step.
    fn eliminate_multiple_candidates(
        &mut self,
        r: usize,
        c: usize,
        elimination_mask: u16, // bits to eliminate
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let initial_mask = self.candidates.get(r, c);
        // Clear all bits present in elimination_mask
        let refined_mask = initial_mask & !elimination_mask;
        self.candidates.set(r, c, refined_mask);

        // Determine which candidate bits were actually present before elimination
        let eliminated_mask = initial_mask & elimination_mask;
        let eliminated_count = eliminated_mask.count_ones();
        let difficulty_point = Self::difficulty_for_technique(flags);

        // Log each individual eliminated candidate digit to the solve path
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

        // Return true if any candidate was eliminated
        initial_mask != refined_mask
    }

    /// Counts empty peer cells sharing a row, column, or 3x3 box with `(r, c)`.
    fn count_affected_cells(&self, r: usize, c: usize, _num: u8) -> u32 {
        let mut count = 0u32;
        let box_r = (r / 3) * 3;
        let box_c = (c / 3) * 3;

        // 1. Count empty cells in the same row (excluding the target cell itself)
        for col in 0..9 {
            if col != c && self.board.is_empty(r, col) {
                count += 1;
            }
        }

        // 2. Count empty cells in the same column (excluding the target cell itself)
        for row in 0..9 {
            if row != r && self.board.is_empty(row, c) {
                count += 1;
            }
        }

        // 3. Count empty cells in the same 3x3 box, deduplicating cells already counted above
        for br in box_r..box_r + 3 {
            for bc in box_c..box_c + 3 {
                if br != r && bc != c && self.board.is_empty(br, bc) {
                    count += 1;
                }
            }
        }

        count
    }

    /// Counts peer cells containing candidate `num` that will be eliminated by placing `num` at `(r, c)`.
    fn count_candidates_eliminated(&self, r: usize, c: usize, num: u8) -> u32 {
        let mut count = 0u32;
        let box_r = (r / 3) * 3;
        let box_c = (c / 3) * 3;
        let candidate_bit = 1u16 << (num - 1);

        // 1. Count peer cells in the same row containing candidate `num`
        for col in 0..9 {
            if col != c && (self.candidates.get(r, col) & candidate_bit) != 0 {
                count += 1;
            }
        }

        // 2. Count peer cells in the same column containing candidate `num`
        for row in 0..9 {
            if row != r && (self.candidates.get(row, c) & candidate_bit) != 0 {
                count += 1;
            }
        }

        // 3. Count peer cells in the 3x3 box containing candidate `num`, excluding row/col peers
        for br in box_r..box_r + 3 {
            for bc in box_c..box_c + 3 {
                if br != r && bc != c && (self.candidates.get(br, bc) & candidate_bit) != 0 {
                    count += 1;
                }
            }
        }

        count
    }

    /// Calculates the difficulty point score for a technique based on its lowest flag bit index.
    fn difficulty_for_technique(flags: TechniqueFlags) -> u8 {
        if flags.is_empty() {
            0
        } else {
            flags.bits().trailing_zeros() as u8 + 1
        }
    }

    /// Iteratively applies enabled techniques in priority order until reaching a fixpoint or contradiction.
    ///
    /// ### Algorithmic Behavior
    /// - **Priority & Greedy Restart**: Evaluates techniques in ascending difficulty. Whenever a technique
    ///   makes progress (places a digit or eliminates candidates), iteration breaks and restarts from the
    ///   simplest techniques to capitalize on newly unlocked deductions.
    /// - **Contradiction & Rollback**: If an empty cell is reduced to zero candidates, returns `false`
    ///   and unwinds all steps applied during this run back to `initial_path_len`.
    /// - **Fixpoint**: Terminates with `true` when a complete pass produces no further changes.
    pub fn propagate_constraints(&mut self, path: &mut SolvePath, initial_path_len: usize) -> bool {
        // Techniques registered in ascending order of complexity/difficulty
        let techniques: Vec<&dyn TechniqueRule> = vec![
            &NakedSingles,
            &HiddenSingles,
            &NakedPairs,
            &HiddenPairs,
            &LockedCandidates,
            &NakedTriples,
            &HiddenTriples,
            &XWing,
            &NakedQuads,
            &HiddenQuads,
            &Swordfish,
            &Jellyfish,
            &Skyscraper,
            &WWing,
            &XYWing,
            &XyzWing,
            &AlternatingInferenceChain,
        ];

        // Fixpoint iteration loop: repeat until no technique can make further progress
        loop {
            let mut changed_this_iter = false;

            // Iterate through registered techniques in priority order
            for technique in &techniques {
                if self.techniques_enabled.contains(technique.flags()) {
                    // Apply technique via Mediator pattern: technique inspects candidates/board
                    // and calls propagator helper methods to register placements or eliminations.
                    changed_this_iter |= technique.apply(self, path);
                    if changed_this_iter {
                        // Greedy restart: restart from simplest technique to capitalize on newly unlocked moves
                        break;
                    }
                }
            }

            // Contradiction Check: If any empty cell has 0 available candidates,
            // the puzzle has reached an invalid/unsolvable state.
            if (0..9).any(|r| {
                (0..9).any(|c| self.board.is_empty(r, c) && self.candidates.get(r, c) == 0)
            }) {
                // Transactional Rollback: unwind all changes made since entering this propagation call
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
                                // Undo board placement and restore constraint masks & peer candidates
                                self.remove_and_update(row, col, value);
                            }
                            SolveStep::CandidateElimination {
                                row,
                                col,
                                value,
                                flags: _,
                                ..
                            } => {
                                // Restore eliminated candidate bit in cell's candidate mask
                                let initial_mask = self.candidates.get(row, col);
                                let refined_mask = initial_mask | (1 << (value - 1));
                                self.candidates.set(row, col, refined_mask);
                            }
                        }
                    }
                }
                return false;
            }

            // Fixpoint Check: If no technique made any changes across the entire pass, stop.
            if !changed_this_iter {
                break;
            }
        }
        true
    }
}

/// Contract for deterministic Sudoku solving strategies.
///
/// Each technique inspects the board/candidate state and applies deductions (placements
/// or candidate eliminations) via [`TechniquePropagator`].
///
/// For intuition and background on these solving patterns, consult
/// [SudokuWiki](https://www.sudokuwiki.org/Introduction) and
/// [HoDoKu](https://hodoku.sourceforge.net/en/tech_intro.php).
pub trait TechniqueRule {
    /// Applies the technique to the given propagator.
    ///
    /// Returns `true` if any candidate was eliminated or cell value was placed.
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool;

    /// Returns the bitflag associated with this technique for attribution and difficulty scoring.
    fn flags(&self) -> TechniqueFlags;
}
