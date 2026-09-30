use crate::core::{SolvePath, TechniqueFlags};

use super::{TechniquePropagator, TechniqueRule, units};

/// Naked pairs technique implementation.
///
/// A naked pair occurs when two cells in the same unit (row, column, or box) contain
/// only the same two candidate numbers. Because those two numbers are locked into those
/// two cells, neither number can appear in any other cell in that unit.
///
/// ### Example
/// If cells `(0, 0)` and `(0, 1)` in row 0 both have candidate mask `{1, 2}`, then
/// candidates 1 and 2 are eliminated from all other empty cells in row 0.
///
/// See: <https://hodoku.sourceforge.net/en/tech_naked.php#n2>
pub struct NakedPairs;

impl NakedPairs {
    /// Processes a single unit (row, column, or box) for naked pairs.
    ///
    /// Identifies pairs of cells with identical 2-candidate masks and eliminates
    /// those candidates from all other cells in the unit.
    fn process_unit_for_naked_pairs(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;
        let mut two_cand_cells: Vec<(usize, usize, u16)> = Vec::new();

        // Step 1: Collect all empty cells in the unit that have exactly 2 candidates
        for &(r, c) in unit_cells {
            if prop.board.is_empty(r, c) {
                let cand_mask = prop.candidates.get(r, c);
                if cand_mask.count_ones() == 2 {
                    two_cand_cells.push((r, c, cand_mask));
                }
            }
        }

        if two_cand_cells.len() < 2 {
            return false;
        }

        // Step 2: Check each pairwise combination of 2-candidate cells
        for i in 0..two_cand_cells.len() {
            for j in (i + 1)..two_cand_cells.len() {
                let (r1, c1, mask1) = two_cand_cells[i];
                let (r2, c2, mask2) = two_cand_cells[j];

                // Step 3: When two cells share the exact same two candidates, eliminate from other cells
                if mask1 == mask2 {
                    let pair_cand_mask = mask1;

                    eliminations_made |= Self::eliminate_candidates_from_other_cells(
                        prop,
                        unit_cells,
                        &[(r1, c1), (r2, c2)],
                        pair_cand_mask,
                        flags,
                        path,
                    );
                }
            }
        }
        eliminations_made
    }

    /// Eliminates specific candidates from cells in a unit, excluding certain cells.
    fn eliminate_candidates_from_other_cells(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        exclude_cells: &[(usize, usize)],
        candidate_mask: u16,
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let mut eliminations_made = false;

        for &(other_r, other_c) in unit_cells {
            // Skip the cells that form the naked pair
            if exclude_cells.contains(&(other_r, other_c)) {
                continue;
            }

            if prop.board.is_empty(other_r, other_c) {
                let initial_mask = prop.candidates.get(other_r, other_c);

                if (initial_mask & candidate_mask) != 0 {
                    eliminations_made |= prop.eliminate_multiple_candidates(
                        other_r,
                        other_c,
                        candidate_mask,
                        flags,
                        path,
                    );
                }
            }
        }

        eliminations_made
    }
}

impl TechniqueRule for NakedPairs {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_eliminations_made = false;

        // Step 1: Process rows for naked pairs
        for i in 0..9 {
            let cells = units::row_cells(i);
            if Self::process_unit_for_naked_pairs(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        // Step 2: Process columns for naked pairs
        for i in 0..9 {
            let cells = units::col_cells(i);
            if Self::process_unit_for_naked_pairs(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        // Step 3: Process 3x3 boxes for naked pairs
        for i in 0..9 {
            let cells = units::box_cells(i);
            if Self::process_unit_for_naked_pairs(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }
        overall_eliminations_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::NAKED_PAIRS
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_naked_pairs_eliminates_candidates() {
        // Hodoku naked pair example
        // https://hodoku.sourceforge.net/en/show_example.php?file=n201&tech=Naked+Pair
        let s = "700009030000105006400260009002083951007000000005600000000000003100000060000004010";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::NAKED_PAIRS);
        let mut path = SolvePath::default();
        rustoku.techniques_make_valid_changes(&mut path);

        let eliminations: Vec<_> = path
            .steps
            .iter()
            .filter_map(|step| match step {
                SolveStep::CandidateElimination {
                    row,
                    col,
                    value,
                    flags,
                    ..
                } if flags.contains(TechniqueFlags::NAKED_PAIRS) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "Naked pairs should produce at least one candidate elimination"
        );

        // Verify eliminated candidates are no longer present
        for &(r, c, v) in &eliminations {
            let cand_bit = 1u16 << (v - 1);
            let remaining = rustoku.candidates.get(r, c);
            assert_eq!(
                remaining & cand_bit,
                0,
                "Candidate {v} should be eliminated from ({r},{c})"
            );
        }

        // Verify that initial clues were not altered
        let original = crate::core::Board::try_from(s).unwrap();
        for r in 0..9 {
            for c in 0..9 {
                let orig_val = original.get(r, c);
                if orig_val != 0 {
                    assert_eq!(
                        rustoku.board.get(r, c),
                        orig_val,
                        "Clue at ({r},{c}) was overwritten"
                    );
                }
            }
        }
    }
}
