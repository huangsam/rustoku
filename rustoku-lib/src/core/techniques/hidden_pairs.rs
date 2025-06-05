use super::TechniqueFlags;
use super::{TechniquePropagator, TechniqueRule};
use crate::core::SolvePath;

/// Hidden pairs technique implementation.
pub struct HiddenPairs;

impl HiddenPairs {
    // Helper function for Hidden Pairs, made private to this impl block
    fn process_unit_for_hidden_pairs(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut unit_placements_made = false;

        for n1_val in 1..=9 {
            for n2_val in (n1_val + 1)..=9 {
                let n1_bit = 1 << (n1_val - 1);
                let n2_bit = 1 << (n2_val - 1);
                let pair_mask = n1_bit | n2_bit; // The candidates we want to KEEP

                let mut cells_containing_n1: Vec<(usize, usize)> = Vec::new();
                let mut cells_containing_n2: Vec<(usize, usize)> = Vec::new();

                for &(r, c) in unit_cells {
                    if prop.board.is_empty(r, c) {
                        let cell_cand_mask = prop.candidates.get(r, c);
                        if (cell_cand_mask & n1_bit) != 0 {
                            cells_containing_n1.push((r, c));
                        }
                        if (cell_cand_mask & n2_bit) != 0 {
                            cells_containing_n2.push((r, c));
                        }
                    }
                }

                if cells_containing_n1.len() == 2
                    && cells_containing_n2.len() == 2
                    && cells_containing_n1[0] == cells_containing_n2[0]
                    && cells_containing_n1[1] == cells_containing_n2[1]
                {
                    let (r1, c1) = cells_containing_n1[0];
                    let (r2, c2) = cells_containing_n1[1];

                    // For the first cell in the pair
                    let current_mask1 = prop.candidates.get(r1, c1);
                    // The candidates to eliminate are all candidates EXCEPT for the pair_mask
                    let elimination_mask1 = current_mask1 & !pair_mask;

                    if elimination_mask1 != 0 {
                        unit_placements_made |= prop.eliminate_multiple_candidates(
                            r1,
                            c1,
                            elimination_mask1,
                            flags,
                            path,
                        );
                    }

                    // For the second cell in the pair
                    let current_mask2 = prop.candidates.get(r2, c2);
                    let elimination_mask2 = current_mask2 & !pair_mask;

                    if elimination_mask2 != 0 {
                        unit_placements_made |= prop.eliminate_multiple_candidates(
                            r2,
                            c2,
                            elimination_mask2,
                            flags,
                            path,
                        )
                    }
                }
            }
        }
        unit_placements_made
    }
}

impl TechniqueRule for HiddenPairs {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_placements_made = false;

        // Process rows
        for i in 0..9 {
            let row_cells: Vec<(usize, usize)> = (0..9).map(|col| (i, col)).collect();
            if Self::process_unit_for_hidden_pairs(prop, &row_cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }

        // Process columns
        for i in 0..9 {
            let col_cells: Vec<(usize, usize)> = (0..9).map(|row| (row, i)).collect();
            if Self::process_unit_for_hidden_pairs(prop, &col_cells, path, self.flags()) {
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
            if Self::process_unit_for_hidden_pairs(prop, &box_cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::HIDDEN_PAIRS
    }
}
