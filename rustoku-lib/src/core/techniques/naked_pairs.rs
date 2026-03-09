use crate::core::{SolvePath, TechniqueFlags};

use super::{TechniquePropagator, TechniqueRule, units};

/// Naked pairs technique implementation.
///
/// A naked pair occurs when two cells in the same unit (row, column, or box) both contain
/// exactly the same two candidate numbers. Since these two cells must contain these two
/// numbers between them, no other cell in the same unit can contain either of these numbers.
///
/// If two cells in a unit are the only possible locations for two specific numbers, then
/// those numbers cannot appear anywhere else in that unit. This allows us to eliminate
/// those candidates from all other cells in the unit.
///
/// Consider this row: [1,2] [1,2] [3,4,5] [3,4,5] [3,4,5] [6,7,8] [6,7,8] [6,7,8] [9]
///
/// The first two cells both have candidates {1,2}. Since these two cells must take 1 and 2,
/// we can eliminate 1 and 2 from all other cells in the row, leaving:
/// [1,2] [1,2] [3,4,5] [3,4,5] [3,4,5] [6,7,8] [6,7,8] [6,7,8] [9]
///
/// 1. Find all cells in a unit that have exactly 2 candidates
/// 2. For each pair of such cells, check if they have identical candidate sets
/// 3. If they do, eliminate those candidates from all other cells in the unit
/// 4. Repeat for rows, columns, and boxes
pub struct NakedPairs;

impl NakedPairs {
    /// Process a single unit (row, column, or box) for naked pairs.
    ///
    /// This function implements the core naked pairs algorithm for one unit:
    /// - Find cells with exactly 2 candidates
    /// - Identify pairs of cells with identical candidate sets
    /// - Eliminate those candidates from other cells in the unit
    ///
    /// Returns true if any eliminations were made.
    fn process_unit_for_naked_pairs(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;
        let mut two_cand_cells: Vec<(usize, usize, u16)> = Vec::new();

        // Find all cells in the unit with exactly 2 candidates
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

        // Check each pair of cells with 2 candidates
        for i in 0..two_cand_cells.len() {
            for j in (i + 1)..two_cand_cells.len() {
                let (r1, c1, mask1) = two_cand_cells[i];
                let (r2, c2, mask2) = two_cand_cells[j];

                if mask1 == mask2 {
                    let pair_cand_mask = mask1;

                    // Eliminate these candidates from other cells in the unit
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

        // Process rows
        for i in 0..9 {
            let cells = units::row_cells(i);
            if Self::process_unit_for_naked_pairs(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        // Process columns
        for i in 0..9 {
            let cells = units::col_cells(i);
            if Self::process_unit_for_naked_pairs(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        // Process 3x3 boxes
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
