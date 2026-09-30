use super::TechniqueFlags;
use super::{TechniquePropagator, TechniqueRule, units};
use crate::core::SolvePath;

/// Hidden pairs technique implementation.
///
/// A hidden pair occurs when two candidate digits in a unit (row, column, or box) appear
/// only within the exact same two cells. Even if those two cells contain additional candidates,
/// those cells must contain the two pair numbers between them. Therefore, all other candidates
/// in those two cells can be safely eliminated.
///
/// ### Example
/// If candidates 1 and 2 appear only in cells `(0, 0)` and `(0, 1)` of row 0, any other candidates
/// in `(0, 0)` and `(0, 1)` (e.g. 3 or 4) are eliminated, leaving only `{1, 2}` in both cells.
///
/// See: <https://hodoku.sourceforge.net/en/tech_hidden.php#h2>
pub struct HiddenPairs;

impl HiddenPairs {
    /// Processes a single unit (row, column, or box) for hidden pairs.
    ///
    /// Identifies pairs of digits confined to the exact same two cells in the unit
    /// and eliminates all other candidates from those two cells.
    fn process_unit_for_hidden_pairs(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        // Step 1: Check each pair of candidate digits (n1, n2)
        for n1_val in 1..=9 {
            for n2_val in (n1_val + 1)..=9 {
                let n1_bit = 1 << (n1_val - 1);
                let n2_bit = 1 << (n2_val - 1);
                let pair_mask = n1_bit | n2_bit; // The candidates we want to KEEP

                // Step 2: Find cells in the unit containing each candidate
                let cells_with_n1 = Self::find_cells_with_candidate(unit_cells, n1_bit, prop);
                let cells_with_n2 = Self::find_cells_with_candidate(unit_cells, n2_bit, prop);

                // Step 3: Check if both candidates appear in exactly 2 cells and those cells match
                if cells_with_n1.len() == 2
                    && cells_with_n2.len() == 2
                    && cells_with_n1 == cells_with_n2
                {
                    let (r1, c1) = cells_with_n1[0];
                    let (r2, c2) = cells_with_n1[1];

                    // Eliminate all other candidates from these two cells
                    eliminations_made |= Self::eliminate_other_candidates_from_cells(
                        prop,
                        &[(r1, c1), (r2, c2)],
                        pair_mask,
                        flags,
                        path,
                    );
                }
            }
        }
        eliminations_made
    }

    /// Finds all cells in a unit that contain a specific candidate.
    fn find_cells_with_candidate(
        unit_cells: &[(usize, usize)],
        candidate_bit: u16,
        prop: &TechniquePropagator,
    ) -> Vec<(usize, usize)> {
        unit_cells
            .iter()
            .filter(|&&(r, c)| {
                prop.board.is_empty(r, c) && (prop.candidates.get(r, c) & candidate_bit) != 0
            })
            .cloned()
            .collect()
    }

    /// Eliminates candidates from specific cells, keeping only the specified mask.
    fn eliminate_other_candidates_from_cells(
        prop: &mut TechniquePropagator,
        cells: &[(usize, usize)],
        keep_mask: u16,
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let mut eliminations_made = false;

        for &(r, c) in cells {
            let current_mask = prop.candidates.get(r, c);
            // The candidates to eliminate are all candidates EXCEPT for the keep_mask
            let elimination_mask = current_mask & !keep_mask;

            if elimination_mask != 0 {
                eliminations_made |=
                    prop.eliminate_multiple_candidates(r, c, elimination_mask, flags, path);
            }
        }

        eliminations_made
    }
}

impl TechniqueRule for HiddenPairs {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_eliminations_made = false;

        // Step 1: Scan all rows for hidden pairs
        for i in 0..9 {
            let cells = units::row_cells(i);
            if Self::process_unit_for_hidden_pairs(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        // Step 2: Scan all columns for hidden pairs
        for i in 0..9 {
            let cells = units::col_cells(i);
            if Self::process_unit_for_hidden_pairs(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        // Step 3: Scan all 3x3 boxes for hidden pairs
        for i in 0..9 {
            let cells = units::box_cells(i);
            if Self::process_unit_for_hidden_pairs(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }
        overall_eliminations_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::HIDDEN_PAIRS
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_hidden_pairs_eliminates_non_pair_candidates() {
        // Hodoku hidden pair example – needs EASY techniques to simplify first
        // https://hodoku.sourceforge.net/en/show_example.php?file=h201&tech=Hidden+Pair
        let s = "000032000000000000007600914096000800005008000030040005050200000700000560904010000";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::EASY | TechniqueFlags::HIDDEN_PAIRS);
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
                } if flags.contains(TechniqueFlags::HIDDEN_PAIRS) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "Hidden pairs should produce at least one candidate elimination"
        );

        // Verify eliminated candidates are no longer present
        for &(r, c, v) in &eliminations {
            let cand_bit = 1u16 << (v - 1);
            let remaining = rustoku.candidates.get(r, c);
            assert_eq!(
                remaining & cand_bit,
                0,
                "Candidate {v} should be eliminated from ({r},{c}) by hidden pair"
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
