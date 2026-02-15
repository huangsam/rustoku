use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule, units};

/// Hidden singles technique implementation.
///
/// A hidden single occurs when a number can only appear in one cell within a unit
/// (row, column, or box), even though that cell may contain other candidates.
/// Since the number must go somewhere in the unit and only one cell can take it,
/// we can place it in that cell.
///
/// Each number 1-9 must appear exactly once in each row, column, and box.
/// If a number has only one possible cell left in a unit, that cell must contain it,
/// regardless of what other candidates the cell might have.
///
/// Consider this row: [1,2,3] [1,4] [2,5] [2,5] [2,5] [6,7] [6,7] [6,7] [8,9]
///
/// Number 1 appears only in the first cell. Even though that cell has other candidates
/// (2 and 3), we know it must contain 1, so we can place 1 there and eliminate 2 and 3.
///
/// 1. For each unit (row, column, box) and each number 1-9:
/// 2. Count how many cells in the unit can contain that number
/// 3. If exactly one cell can contain it, place the number in that cell
/// 4. Update constraints and candidates for the entire board
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
            let cells = units::row_cells(r);
            if check_unit_hidden_singles(&cells, prop, path) {
                overall_placements_made = true;
            }
        }

        for c in 0..9 {
            let cells = units::col_cells(c);
            if check_unit_hidden_singles(&cells, prop, path) {
                overall_placements_made = true;
            }
        }

        for box_idx in 0..9 {
            let cells = units::box_cells(box_idx);
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
