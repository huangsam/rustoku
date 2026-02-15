use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule};

/// Hidden singles technique implementation.
pub struct HiddenSingles;

/// Helper to build unit cell arrays without heap allocation.
fn row_cells(r: usize) -> [(usize, usize); 9] {
    core::array::from_fn(|c| (r, c))
}

fn col_cells(c: usize) -> [(usize, usize); 9] {
    core::array::from_fn(|r| (r, c))
}

fn box_cells(box_idx: usize) -> [(usize, usize); 9] {
    let start_row = (box_idx / 3) * 3;
    let start_col = (box_idx % 3) * 3;
    core::array::from_fn(|i| (start_row + i / 3, start_col + i % 3))
}

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
            let cells = row_cells(r);
            if check_unit_hidden_singles(&cells, prop, path) {
                overall_placements_made = true;
            }
        }

        for c in 0..9 {
            let cells = col_cells(c);
            if check_unit_hidden_singles(&cells, prop, path) {
                overall_placements_made = true;
            }
        }

        for box_idx in 0..9 {
            let cells = box_cells(box_idx);
            if check_unit_hidden_singles(&cells, prop, path) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::HIDDEN_SINGLES
    }
}
