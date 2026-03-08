use super::TechniqueFlags;
use super::{TechniquePropagator, TechniqueRule, units};
use crate::core::SolvePath;

/// Hidden triples technique implementation.
///
/// A hidden triple occurs when three numbers in a unit (row, column, or box) can only appear
/// in exactly three cells. Even if those cells contain other candidates, the fact that only
/// those three cells can contain the triple means we can eliminate all other candidates from
/// those cells.
pub struct HiddenTriples;

impl HiddenTriples {
    fn process_unit_for_hidden_triples(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        for n1_val in 1..=7 {
            for n2_val in (n1_val + 1)..=8 {
                for n3_val in (n2_val + 1)..=9 {
                    let n1_bit = 1 << (n1_val - 1);
                    let n2_bit = 1 << (n2_val - 1);
                    let n3_bit = 1 << (n3_val - 1);
                    let triple_mask = n1_bit | n2_bit | n3_bit;

                    let cells1 = Self::find_cells_with_candidate(unit_cells, n1_bit, prop);
                    let cells2 = Self::find_cells_with_candidate(unit_cells, n2_bit, prop);
                    let cells3 = Self::find_cells_with_candidate(unit_cells, n3_bit, prop);

                    // If any candidate has 0 or >3 positions, it can't be part of a hidden triple
                    if cells1.is_empty()
                        || cells1.len() > 3
                        || cells2.is_empty()
                        || cells2.len() > 3
                        || cells3.is_empty()
                        || cells3.len() > 3
                    {
                        continue;
                    }

                    // Compute union of cells
                    let mut all_cells = cells1.clone();
                    all_cells.extend(cells2.iter());
                    all_cells.extend(cells3.iter());
                    all_cells.sort_unstable();
                    all_cells.dedup();

                    if all_cells.len() == 3 {
                        // We found 3 cells that contain all instances of n1, n2, n3.
                        // Eliminate all other candidates from these 3 cells.
                        eliminations_made |= Self::eliminate_other_candidates_from_cells(
                            prop,
                            &all_cells,
                            triple_mask,
                            flags,
                            path,
                        );
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

impl TechniqueRule for HiddenTriples {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_placements_made = false;

        for i in 0..9 {
            let cells = units::row_cells(i);
            if Self::process_unit_for_hidden_triples(prop, &cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }

        for i in 0..9 {
            let cells = units::col_cells(i);
            if Self::process_unit_for_hidden_triples(prop, &cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }

        for i in 0..9 {
            let cells = units::box_cells(i);
            if Self::process_unit_for_hidden_triples(prop, &cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::HIDDEN_TRIPLES
    }
}
