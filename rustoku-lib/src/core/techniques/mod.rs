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
///
/// ### Mediator Architecture & Invariants
/// Individual [`TechniqueRule`] implementations do not mutate the board or candidates
/// directly. Instead, they interact solely through `TechniquePropagator`:
/// 1. **Synchronized State Updates**: Placing a number or eliminating candidates automatically
///    synchronizes [`Board`], [`Masks`], and [`Candidates`] caches.
/// 2. **Telemetry & Audit Trail**: Every deduction is logged to the [`SolvePath`] with
///    metrics like candidates eliminated, peer cells affected, and difficulty points.
/// 3. **Transactional Rollback**: If propagation uncovers an inconsistency (an empty cell with
///    no valid candidates left), all steps executed during the propagation pass are cleanly
///    unwound in reverse order to restore the board and candidate state.
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

    /// Places a number on the board and propagates the new constraint across all peer cells.
    ///
    /// This method performs the following coordinated updates:
    /// 1. Updates the cell value on the [`Board`].
    /// 2. Records the number in the row, column, and 3x3 box in [`Masks`].
    /// 3. Computes telemetry: counts affected peer cells and candidate eliminations caused by the placement.
    /// 4. Recomputes candidate bitmasks for all peer cells (same row, column, box) via [`Candidates::update_affected_cells`].
    /// 5. Appends a [`SolveStep::Placement`] to `path` with difficulty rating and telemetry.
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

    /// Reverses a number placement, restoring constraint masks and recalculating candidates.
    ///
    /// Used during contradiction rollback to undo tentative placements made by techniques.
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
    ///
    /// `candidate_bit` is a one-hot bitmask representing digit `v` as `1 << (v - 1)`.
    ///
    /// Returns `true` if the candidate was present and successfully eliminated, `false` otherwise.
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

    /// Eliminates multiple candidate digits simultaneously from cell `(r, c)` and logs each elimination.
    ///
    /// `elimination_mask` is a bitmask where set bits correspond to digits to eliminate.
    /// Used by techniques such as Naked/Hidden Subsets and Locked Candidates.
    ///
    /// Returns `true` if one or more candidates were actually eliminated, `false` otherwise.
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

    /// Counts empty peer cells in the same row, column, or 3x3 box as `(r, c)`.
    ///
    /// Deduplicates cells that appear in multiple units (e.g. intersection of row and box).
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

    /// Counts how many peer cells currently contain `num` as a candidate.
    ///
    /// Placing `num` at `(r, c)` will eliminate `num` from all these cells.
    /// Deduplicates peer cells across row, column, and 3x3 box boundaries.
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

    /// Iteratively applies enabled solving techniques until reaching a fixpoint or detecting a contradiction.
    ///
    /// ### Algorithmic Workflow
    /// 1. **Prioritized Ordering**: Techniques are evaluated in order of human difficulty and
    ///    computational complexity, starting from fast direct deductions ([`NakedSingles`],
    ///    [`HiddenSingles`]) through intermediate patterns ([`LockedCandidates`], [`XWing`])
    ///    up to advanced search chains ([`AlternatingInferenceChain`]).
    /// 2. **Greedy Restart**: Whenever a technique makes any deduction (placing a number or
    ///    eliminating candidates), the loop breaks immediately and restarts from the beginning.
    ///    This ensures simpler techniques are always prioritized (e.g. an advanced elimination
    ///    that exposes a new Naked Single will immediately trigger the Naked Single rather than
    ///    another advanced technique).
    /// 3. **Contradiction Detection & Rollback**: If an empty cell is reduced to zero candidates,
    ///    the puzzle state is invalid. The method unwinds all steps applied during this run
    ///    (back to `initial_path_len`), restoring candidate masks and removing placements,
    ///    and returns `false`.
    /// 4. **Fixpoint Termination**: When a complete iteration pass over all enabled techniques
    ///    yields no changes, the puzzle cannot be simplified further with the current techniques.
    ///    The method terminates cleanly and returns `true`.
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

/// This is the contract for all human techniques.
///
/// All techniques are expected to have a way to apply themselves to a board
/// and modify the solve path with placements and eliminations. In addition, they
/// are expected to return one flag that helps with technique attribution when
/// people want to visualize the solve path.
///
/// To get started on the intuition behind the techniques, check out
/// [SudokuWiki](https://www.sudokuwiki.org/Introduction) and
/// [HoDoKu](https://hodoku.sourceforge.net/en/tech_intro.php)
/// to understand the basic strategies and techniques used in Sudoku solving.
pub trait TechniqueRule {
    /// Applies the technique to the given propagator.
    ///
    /// Returns `true` if any candidate was eliminated or cell value was placed,
    /// signalling the propagator to record progress and restart the technique pass.
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool;

    /// Returns the bitflag associated with this technique for attribution and difficulty scoring.
    fn flags(&self) -> TechniqueFlags;
}
