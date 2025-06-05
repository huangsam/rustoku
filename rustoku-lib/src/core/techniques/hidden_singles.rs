use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule};

/// Hidden singles technique implementation.
pub struct HiddenSingles;

impl TechniqueRule for HiddenSingles {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_placements_made = false;

        // Helper closure (can be moved to a private helper function if desired)
        let check_unit_hidden_singles =
            |unit_cells: &[(usize, usize)],
             prop: &mut TechniquePropagator,
             path: &mut SolvePath| {
                let mut unit_placement_made = false;
                for cand_val in 1..=9 {
                    let cand_bit = 1 << (cand_val - 1);
                    let mut potential_cell: Option<(usize, usize)> = None;
                    let mut cand_occurrences = 0;

                    for &(r, c) in unit_cells.iter() {
                        if prop.board.is_empty(r, c) {
                            let cell_cand_mask = prop.candidates.get(r, c);
                            if (cell_cand_mask & cand_bit) != 0 {
                                cand_occurrences += 1;
                                potential_cell = Some((r, c));
                            }
                        }
                    }

                    if cand_occurrences == 1 {
                        if let Some((r, c)) = potential_cell {
                            if prop.board.is_empty(r, c) {
                                prop.place_and_update(r, c, cand_val, self.flags(), path);
                                unit_placement_made = true;
                            }
                        }
                    }
                }
                unit_placement_made
            };

        for r in 0..9 {
            let row_cells: Vec<(usize, usize)> = (0..9).map(|c| (r, c)).collect();
            if check_unit_hidden_singles(&row_cells, prop, path) {
                overall_placements_made = true;
            }
        }

        for c in 0..9 {
            let col_cells: Vec<(usize, usize)> = (0..9).map(|r| (r, c)).collect();
            if check_unit_hidden_singles(&col_cells, prop, path) {
                overall_placements_made = true;
            }
        }

        for box_idx in 0..9 {
            let mut box_cells: Vec<(usize, usize)> = Vec::with_capacity(9);
            let start_row = (box_idx / 3) * 3;
            let start_col = (box_idx % 3) * 3;
            for r_offset in 0..3 {
                for c_offset in 0..3 {
                    box_cells.push((start_row + r_offset, start_col + c_offset));
                }
            }
            if check_unit_hidden_singles(&box_cells, prop, path) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::HIDDEN_SINGLES
    }
}
