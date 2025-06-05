use crate::core::{SolvePath, TechniqueFlags};

use super::{TechniquePropagator, TechniqueRule};

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
        let mut unit_placements_made = false;
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
                                unit_placements_made |= prop.eliminate_multiple_candidates(
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
        unit_placements_made
    }
}

impl TechniqueRule for NakedPairs {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_placements_made = false;

        // Process rows
        for i in 0..9 {
            let row_cells: Vec<(usize, usize)> = (0..9).map(|col| (i, col)).collect();
            if Self::process_unit_for_naked_pairs(prop, &row_cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }

        // Process columns
        for i in 0..9 {
            let col_cells: Vec<(usize, usize)> = (0..9).map(|row| (row, i)).collect();
            if Self::process_unit_for_naked_pairs(prop, &col_cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }

        // Process 3x3 boxes
        for i in 0..9 {
            let mut box_cells: Vec<(usize, usize)> = Vec::with_capacity(9);
            let start_row = (i / 3) * 3;
            let start_col = (i % 3) * 3;
            for r_offset in 0..3 {
                for c_offset in 0..3 {
                    box_cells.push((start_row + r_offset, start_col + c_offset));
                }
            }
            if Self::process_unit_for_naked_pairs(prop, &box_cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::NAKED_PAIRS
    }
}
