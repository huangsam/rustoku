use super::Rustoku;

/// Propagation methods for Sudoku puzzles.
impl Rustoku {
    /// Applies the naked singles technique.
    ///
    /// A naked single occurs if an empty cell has only one possible candidate.
    /// This technique finds such cells and places the single candidate.
    pub(super) fn naked_singles(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut placements_made = false;
        // Temporarily store placements made in this pass
        let mut pass_placements: Vec<(usize, usize, u8)> = Vec::new();

        for r in 0..9 {
            for c in 0..9 {
                if self.board[r][c] == 0 {
                    // Only consider empty cells
                    let cand_mask = self.get_possible_candidates_mask(r, c);
                    // Check if the cell has exactly one possible candidate
                    if cand_mask.count_ones() == 1 {
                        // The single candidate number is derived from the position of the single set bit
                        let num = cand_mask.trailing_zeros() as u8 + 1;

                        self.place_number(r, c, num);
                        pass_placements.push((r, c, num));
                        placements_made = true;
                    }
                }
            }
        }

        // Add all placements made in this pass to the main solve path.
        for (r, c, num) in pass_placements {
            path.push((r, c, num));
        }
        placements_made
    }

    /// Applies the hidden singles technique.
    ///
    /// A hidden single occurs when a specific number can only be placed in one cell
    /// within a unit (row, column, or 3x3 box), even if that cell itself has other candidates.
    pub(super) fn hidden_singles(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut overall_placements_made = false;

        // Helper closure to find hidden singles in a given unit (row, column, or box)
        let mut check_unit_hidden_singles = |unit_cells: &[(usize, usize)]| {
            let mut unit_placement_made = false;
            // For each possible number (1 through 9)
            for cand_val in 1..=9 {
                let cand_bit = 1 << (cand_val - 1); // Bitmask for the current number

                let mut potential_cell: Option<(usize, usize)> = None;
                let mut cand_occurrences = 0;

                // Iterate through all cells in the current unit
                for &(r, c) in unit_cells.iter() {
                    if self.board[r][c] == 0 {
                        // Only consider empty cells
                        // Check if 'cand_val' is a possible candidate for this empty cell
                        let cell_cand_mask = self.get_possible_candidates_mask(r, c);
                        if (cell_cand_mask & cand_bit) != 0 {
                            cand_occurrences += 1;
                            potential_cell = Some((r, c));
                        }
                    }
                }

                // If 'cand_val' was found as a candidate in *exactly one* empty cell
                // within this unit, then it's a hidden single
                if cand_occurrences == 1 {
                    if let Some((r, c)) = potential_cell {
                        // Ensure the cell is still empty before placing.
                        if self.board[r][c] == 0 {
                            self.place_number(r, c, cand_val);
                            path.push((r, c, cand_val));
                            unit_placement_made = true;
                        }
                    }
                }
            }
            unit_placement_made
        };

        // Check rows for hidden singles
        for r in 0..9 {
            let row_cells: Vec<(usize, usize)> = (0..9).map(|c| (r, c)).collect();
            if check_unit_hidden_singles(&row_cells) {
                overall_placements_made = true;
            }
        }

        // Check columns for hidden singles
        for c in 0..9 {
            let col_cells: Vec<(usize, usize)> = (0..9).map(|r| (r, c)).collect();
            if check_unit_hidden_singles(&col_cells) {
                overall_placements_made = true;
            }
        }

        // Check 3x3 boxes for hidden singles
        for box_idx in 0..9 {
            let mut box_cells: Vec<(usize, usize)> = Vec::with_capacity(9);
            let start_row = (box_idx / 3) * 3;
            let start_col = (box_idx % 3) * 3;
            for r_offset in 0..3 {
                for c_offset in 0..3 {
                    box_cells.push((start_row + r_offset, start_col + c_offset));
                }
            }
            if check_unit_hidden_singles(&box_cells) {
                overall_placements_made = true;
            }
        }

        overall_placements_made
    }

    /// Applies the naked pairs technique.
    pub(super) fn naked_pairs(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut overall_placements_made = false;

        // Process rows
        for i in 0..9 {
            let row_cells: Vec<(usize, usize)> = (0..9).map(|col| (i, col)).collect();
            if self.process_unit_for_naked_pairs(&row_cells, path) {
                overall_placements_made = true;
            }
        }

        // Process columns
        for i in 0..9 {
            let col_cells: Vec<(usize, usize)> = (0..9).map(|row| (row, i)).collect();
            if self.process_unit_for_naked_pairs(&col_cells, path) {
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
            if self.process_unit_for_naked_pairs(&box_cells, path) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    /// Helper function to process a single unit (row, column, or box) for naked pairs.
    pub(super) fn process_unit_for_naked_pairs(
        &mut self,
        unit_cells: &[(usize, usize)], // Cells in the current row, column, or box
        path: &mut Vec<(usize, usize, u8)>,
    ) -> bool {
        let mut unit_placements_made = false;
        // Stores (row, col, candidate_mask) for cells in this unit that are empty
        // and have exactly two possible candidates
        let mut two_cand_cells: Vec<(usize, usize, u16)> = Vec::new();

        // Step 1: Identify empty cells with two candidates
        for &(r, c) in unit_cells {
            if self.board[r][c] == 0 {
                let cand_mask = self.get_possible_candidates_mask(r, c);
                if cand_mask.count_ones() == 2 {
                    two_cand_cells.push((r, c, cand_mask));
                }
            }
        }

        if two_cand_cells.len() < 2 {
            return false;
        }

        // Step 2: Find naked pairs
        for i in 0..two_cand_cells.len() {
            for j in (i + 1)..two_cand_cells.len() {
                let (r1, c1, mask1) = two_cand_cells[i];
                let (r2, c2, mask2) = two_cand_cells[j];

                if mask1 == mask2 {
                    // Found a naked pair
                    let pair_cand_mask = mask1;

                    // Step 3: Remove pair candidates from other cells in the unit
                    for &(other_r, other_c) in unit_cells {
                        if (other_r == r1 && other_c == c1) || (other_r == r2 && other_c == c2) {
                            continue;
                        }

                        if self.board[other_r][other_c] == 0 {
                            let initial_mask = self.get_possible_candidates_mask(other_r, other_c);

                            if (initial_mask & pair_cand_mask) != 0 {
                                let refined_mask = initial_mask & !pair_cand_mask;

                                // Step 4: Place number if refined mask has one candidate
                                if refined_mask.count_ones() == 1 {
                                    let num = refined_mask.trailing_zeros() as u8 + 1;

                                    if self.is_safe(other_r, other_c, num) {
                                        self.place_number(other_r, other_c, num);
                                        path.push((other_r, other_c, num));
                                        unit_placements_made = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        unit_placements_made
    }

    /// Applies deterministic constraint propagation techniques iteratively.
    pub(super) fn propagate_constraints(
        &mut self,
        path: &mut Vec<(usize, usize, u8)>,
        initial_path_len: usize,
    ) -> bool {
        loop {
            let mut changed_this_iter = false;
            changed_this_iter |= self.naked_singles(path);
            changed_this_iter |= self.hidden_singles(path);
            changed_this_iter |= self.naked_pairs(path);

            // Contradiction Check
            for r in 0..9 {
                for c in 0..9 {
                    if self.board[r][c] == 0 && self.get_possible_candidates_mask(r, c) == 0 {
                        // Contradiction: Roll back placements from this propagation call
                        while path.len() > initial_path_len {
                            let (r, c, num) = path.pop().unwrap();
                            self.remove_number(r, c, num);
                        }
                        return false; // Propagation failed
                    }
                }
            }

            if !changed_this_iter {
                break; // Stable state
            }
        }
        true // Propagation successful
    }
}
