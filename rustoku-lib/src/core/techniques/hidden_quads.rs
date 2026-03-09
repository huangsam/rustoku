use super::TechniqueFlags;
use super::{TechniquePropagator, TechniqueRule, units};
use crate::core::SolvePath;

/// Hidden quads technique implementation.
///
/// A hidden quad occurs when four candidate numbers in a unit (row, column, or box)
/// appear in exactly four cells, and nowhere else in that unit.
///
/// Even if those four cells contain other "extraneous" candidates, we know that
/// those four specific cells *must* contain the four quad numbers between them.
/// Therefore, we can safely eliminate all other candidates from those four cells.
///
/// This is the "hidden" counterpart to Naked Quads. While Naked Quads are found by
/// looking at cell candidate counts, Hidden Quads are found by looking at the
/// distribution of candidate positions across a unit.
///
/// Example:
/// If in a row, the numbers {2, 4, 7, 8} appear only in cells C2, C5, C6, and C9,
/// then any other numbers in those four cells (like a 3 or a 5) can be removed.
pub struct HiddenQuads;

impl HiddenQuads {
    fn process_unit_for_hidden_quads(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        for n1_val in 1..=6 {
            for n2_val in (n1_val + 1)..=7 {
                for n3_val in (n2_val + 1)..=8 {
                    for n4_val in (n3_val + 1)..=9 {
                        let n1_bit = 1 << (n1_val - 1);
                        let n2_bit = 1 << (n2_val - 1);
                        let n3_bit = 1 << (n3_val - 1);
                        let n4_bit = 1 << (n4_val - 1);
                        let quad_mask = n1_bit | n2_bit | n3_bit | n4_bit;

                        let cells1 = Self::find_cells_with_candidate(unit_cells, n1_bit, prop);
                        let cells2 = Self::find_cells_with_candidate(unit_cells, n2_bit, prop);
                        let cells3 = Self::find_cells_with_candidate(unit_cells, n3_bit, prop);
                        let cells4 = Self::find_cells_with_candidate(unit_cells, n4_bit, prop);

                        // If any candidate has 0 or >4 positions, it can't be part of a hidden quad
                        if cells1.is_empty()
                            || cells1.len() > 4
                            || cells2.is_empty()
                            || cells2.len() > 4
                            || cells3.is_empty()
                            || cells3.len() > 4
                            || cells4.is_empty()
                            || cells4.len() > 4
                        {
                            continue;
                        }

                        // Compute union of cells
                        let mut all_cells = cells1.clone();
                        all_cells.extend(cells2.iter());
                        all_cells.extend(cells3.iter());
                        all_cells.extend(cells4.iter());
                        all_cells.sort_unstable();
                        all_cells.dedup();

                        if all_cells.len() == 4 {
                            // We found 4 cells that contain all instances of n1, n2, n3, n4.
                            // Eliminate all other candidates from these 4 cells.
                            eliminations_made |= Self::eliminate_other_candidates_from_cells(
                                prop, &all_cells, quad_mask, flags, path,
                            );
                        }
                    }
                }
            }
        }
        eliminations_made
    }

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
            .copied()
            .collect()
    }

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
            let elimination_mask = current_mask & !keep_mask;

            if elimination_mask != 0 {
                eliminations_made |=
                    prop.eliminate_multiple_candidates(r, c, elimination_mask, flags, path);
            }
        }

        eliminations_made
    }
}

impl TechniqueRule for HiddenQuads {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_placements_made = false;

        for i in 0..9 {
            let cells = units::row_cells(i);
            if Self::process_unit_for_hidden_quads(prop, &cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }

        for i in 0..9 {
            let cells = units::col_cells(i);
            if Self::process_unit_for_hidden_quads(prop, &cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }

        for i in 0..9 {
            let cells = units::box_cells(i);
            if Self::process_unit_for_hidden_quads(prop, &cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::HIDDEN_QUADS
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, TechniqueFlags};

    #[test]
    fn test_hidden_quads_eliminates_candidates() {
        // Hodoku Hidden Quads example
        // https://hodoku.sourceforge.net/en/show_example.php?file=h401&tech=Hidden+Quad
        let s = "800570290390000000000200000001000508000496000000800000209000001008000070560000082";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::HIDDEN_QUADS | TechniqueFlags::EASY);
        let mut path = crate::core::SolvePath::default();
        rustoku.techniques_make_valid_changes(&mut path);

        let has_hidden_quad = path.steps.iter().any(|step| match step {
            crate::core::SolveStep::CandidateElimination { flags, .. } => {
                flags.contains(TechniqueFlags::HIDDEN_QUADS)
            }
            _ => false,
        });

        assert!(
            has_hidden_quad,
            "Expected HIDDEN_QUADS technique to be used in solution path"
        );

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
