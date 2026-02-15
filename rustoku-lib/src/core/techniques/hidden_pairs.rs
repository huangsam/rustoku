use super::TechniqueFlags;
use super::{TechniquePropagator, TechniqueRule, units};
use crate::core::SolvePath;

/// Hidden pairs technique implementation.
///
/// A hidden pair occurs when two numbers in a unit (row, column, or box) can only appear
/// in exactly two cells. Even if those cells contain other candidates, the fact that only
/// those two cells can contain the pair means we can eliminate all other candidates from
/// those cells.
///
/// If two numbers can only appear in two specific cells within a unit, then those cells
/// must contain those two numbers (in some order). Therefore, any other candidates in
/// those cells can be eliminated.
///
/// Consider this row: [1,2,3] [1,2,4] [5,6] [5,6] [5,6] [7,8,9] [7,8,9] [7,8,9] [7,8,9]
///
/// Numbers 1 and 2 only appear in the first two cells. Even though those cells have
/// other candidates (3 and 4), we know they must contain 1 and 2 between them.
/// We can eliminate 3 from the first cell and 4 from the second cell, leaving:
/// [1,2] [1,2] [5,6] [5,6] [5,6] [7,8,9] [7,8,9] [7,8,9] [7,8,9]
///
/// 1. For each pair of numbers (n1, n2), find cells in the unit that contain n1 and n2
/// 2. If exactly 2 cells contain n1 AND exactly 2 cells contain n2 AND they are the same cells,
///    then we have a hidden pair
/// 3. Eliminate all other candidates from those two cells
/// 4. Repeat for rows, columns, and boxes
pub struct HiddenPairs;

impl HiddenPairs {
    /// Process a single unit (row, column, or box) for hidden pairs.
    ///
    /// This function implements the core hidden pairs algorithm for one unit:
    /// - For each pair of numbers, check if they appear in exactly 2 cells
    /// - If those cells are the same for both numbers, we have a hidden pair
    /// - Eliminate other candidates from those cells
    ///
    /// Returns true if any eliminations were made.
    fn process_unit_for_hidden_pairs(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

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
                        eliminations_made |= prop.eliminate_multiple_candidates(
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
                        eliminations_made |= prop.eliminate_multiple_candidates(
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
        eliminations_made
    }
}

impl TechniqueRule for HiddenPairs {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_placements_made = false;

        // Process rows
        for i in 0..9 {
            let cells = units::row_cells(i);
            if Self::process_unit_for_hidden_pairs(prop, &cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }

        // Process columns
        for i in 0..9 {
            let cells = units::col_cells(i);
            if Self::process_unit_for_hidden_pairs(prop, &cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }

        // Process 3x3 boxes
        for i in 0..9 {
            let cells = units::box_cells(i);
            if Self::process_unit_for_hidden_pairs(prop, &cells, path, self.flags()) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::HIDDEN_PAIRS
    }
}
