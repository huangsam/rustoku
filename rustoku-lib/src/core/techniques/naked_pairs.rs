use crate::core::{SolvePath, TechniqueFlags};

use super::{TechniquePropagator, TechniqueRule, units};

/// Naked pairs technique implementation.
pub struct NakedPairs;

impl NakedPairs {
    // Helper function for Naked Pairs, made private to this impl block
    fn process_unit_for_naked_pairs(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;
        let mut two_cand_cells: Vec<(usize, usize, u16)> = Vec::new();

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

        for i in 0..two_cand_cells.len() {
            for j in (i + 1)..two_cand_cells.len() {
                let (r1, c1, mask1) = two_cand_cells[i];
                let (r2, c2, mask2) = two_cand_cells[j];

                if mask1 == mask2 {
                    let pair_cand_mask = mask1;

                    for &(other_r, other_c) in unit_cells {
                        if (other_r == r1 && other_c == c1) || (other_r == r2 && other_c == c2) {
                            continue;
                        }

                        if prop.board.is_empty(other_r, other_c) {
                            let initial_mask = prop.candidates.get(other_r, other_c);

                            if (initial_mask & pair_cand_mask) != 0 {
                                eliminations_made |= prop.eliminate_multiple_candidates(
                                    other_r,
                                    other_c,
                                    pair_cand_mask,
                                    flags,
                                    path,
                                );
                            }
                        }
                    }
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
