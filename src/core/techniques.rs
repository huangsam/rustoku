use super::board::RustokuBoard;
use super::candidates::RustokuCandidates;
use super::masks::RustokuMasks;
use bitflags::bitflags;

bitflags! {
    /// A bitmask to control which human-like solving techniques are applied during propagation.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RustokuTechniques: u16 {
        const NONE = 0b0000_0000;

        const NAKED_SINGLES = 0b0000_0001;
        const HIDDEN_SINGLES = 0b0000_0010;

        const NAKED_PAIRS = 0b0000_0100;
        const HIDDEN_PAIRS = 0b0000_1000;

        const SIMPLE = Self::NAKED_SINGLES.bits() | Self::HIDDEN_SINGLES.bits();
        const COMPLEX = Self::NAKED_PAIRS.bits() | Self::HIDDEN_PAIRS.bits();

        const ALL = Self::SIMPLE.bits() | Self::COMPLEX.bits();
    }
}

// Now the actual implementation of the techniques, these would operate on
// references to Board, Masks, and CandidatesCache.
pub struct TechniquePropagator<'a> {
    board: &'a mut RustokuBoard,
    masks: &'a mut RustokuMasks,
    candidates_cache: &'a mut RustokuCandidates,
    techniques_enabled: RustokuTechniques,
}

impl<'a> TechniquePropagator<'a> {
    pub fn new(
        board: &'a mut RustokuBoard,
        masks: &'a mut RustokuMasks,
        candidates_cache: &'a mut RustokuCandidates,
        techniques_enabled: RustokuTechniques,
    ) -> Self {
        Self {
            board,
            masks,
            candidates_cache,
            techniques_enabled,
        }
    }

    // Helper to place a number and update caches
    fn place_and_update(
        &mut self,
        r: usize,
        c: usize,
        num: u8,
        path: &mut Vec<(usize, usize, u8)>,
    ) {
        self.board.set(r, c, num);
        self.masks.add_number(r, c, num);
        self.candidates_cache
            .update_affected_cells(r, c, self.masks, self.board);
        path.push((r, c, num));
    }

    // Helper to remove a number and update caches
    fn remove_and_update(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, 0);
        self.masks.remove_number(r, c, num);
        self.candidates_cache
            .update_affected_cells(r, c, self.masks, self.board);
        // Note: For propagation, `remove_number` is mostly for backtracking, not direct technique application.
        // The `update_affected_cells` on removal will recalculate candidates for the now-empty cell.
    }

    /// Applies the naked singles technique.
    fn naked_singles(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut placements_made = false;
        let mut pass_placements: Vec<(usize, usize, u8)> = Vec::new();

        for r in 0..9 {
            for c in 0..9 {
                if self.board.is_empty(r, c) {
                    let cand_mask = self.candidates_cache.get(r, c);
                    if cand_mask.count_ones() == 1 {
                        let num = cand_mask.trailing_zeros() as u8 + 1;
                        self.place_and_update(r, c, num, path);
                        pass_placements.push((r, c, num)); // Store for propagation
                        placements_made = true;
                    }
                }
            }
        }
        placements_made
    }

    /// Applies the hidden singles technique.
    fn hidden_singles(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut overall_placements_made = false;

        let mut check_unit_hidden_singles = |unit_cells: &[(usize, usize)]| {
            let mut unit_placement_made = false;
            for cand_val in 1..=9 {
                let cand_bit = 1 << (cand_val - 1);
                let mut potential_cell: Option<(usize, usize)> = None;
                let mut cand_occurrences = 0;

                for &(r, c) in unit_cells.iter() {
                    if self.board.is_empty(r, c) {
                        let cell_cand_mask = self.candidates_cache.get(r, c);
                        if (cell_cand_mask & cand_bit) != 0 {
                            cand_occurrences += 1;
                            potential_cell = Some((r, c));
                        }
                    }
                }

                if cand_occurrences == 1 {
                    if let Some((r, c)) = potential_cell {
                        if self.board.is_empty(r, c) {
                            self.place_and_update(r, c, cand_val, path);
                            unit_placement_made = true;
                        }
                    }
                }
            }
            unit_placement_made
        };

        for r in 0..9 {
            let row_cells: Vec<(usize, usize)> = (0..9).map(|c| (r, c)).collect();
            if check_unit_hidden_singles(&row_cells) {
                overall_placements_made = true;
            }
        }

        for c in 0..9 {
            let col_cells: Vec<(usize, usize)> = (0..9).map(|r| (r, c)).collect();
            if check_unit_hidden_singles(&col_cells) {
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
            if check_unit_hidden_singles(&box_cells) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    /// Applies the naked pairs technique.
    fn naked_pairs(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
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
    fn process_unit_for_naked_pairs(
        &mut self,
        unit_cells: &[(usize, usize)],
        path: &mut Vec<(usize, usize, u8)>,
    ) -> bool {
        let mut unit_placements_made = false;
        let mut two_cand_cells: Vec<(usize, usize, u16)> = Vec::new();

        for &(r, c) in unit_cells {
            if self.board.is_empty(r, c) {
                let cand_mask = self.candidates_cache.get(r, c);
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

                        if self.board.is_empty(other_r, other_c) {
                            let initial_mask = self.candidates_cache.get(other_r, other_c);

                            if (initial_mask & pair_cand_mask) != 0 {
                                let refined_mask = initial_mask & !pair_cand_mask;

                                self.candidates_cache.set(other_r, other_c, refined_mask);
                                unit_placements_made = true;

                                if refined_mask.count_ones() == 1 {
                                    let num = refined_mask.trailing_zeros() as u8 + 1;

                                    if self.masks.is_safe(other_r, other_c, num) {
                                        self.place_and_update(other_r, other_c, num, path);
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

    /// Applies the hidden pairs technique.
    fn hidden_pairs(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut overall_placements_made = false;

        // Process rows
        for i in 0..9 {
            let row_cells: Vec<(usize, usize)> = (0..9).map(|col| (i, col)).collect();
            if self.process_unit_for_hidden_pairs(&row_cells, path) {
                overall_placements_made = true;
            }
        }

        // Process columns
        for i in 0..9 {
            let col_cells: Vec<(usize, usize)> = (0..9).map(|row| (row, i)).collect();
            if self.process_unit_for_hidden_pairs(&col_cells, path) {
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
            if self.process_unit_for_hidden_pairs(&box_cells, path) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    /// Helper function to process a single unit (row, column, or box) for hidden pairs.
    fn process_unit_for_hidden_pairs(
        &mut self,
        unit_cells: &[(usize, usize)],
        path: &mut Vec<(usize, usize, u8)>,
    ) -> bool {
        let mut unit_placements_made = false;

        for n1_val in 1..=9 {
            for n2_val in (n1_val + 1)..=9 {
                let n1_bit = 1 << (n1_val - 1);
                let n2_bit = 1 << (n2_val - 1);
                let pair_mask = n1_bit | n2_bit;

                let mut cells_containing_n1: Vec<(usize, usize)> = Vec::new();
                let mut cells_containing_n2: Vec<(usize, usize)> = Vec::new();

                for &(r, c) in unit_cells {
                    if self.board.is_empty(r, c) {
                        let cell_cand_mask = self.candidates_cache.get(r, c);
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

                    let current_mask1 = self.candidates_cache.get(r1, c1);
                    let new_mask1 = pair_mask;

                    if new_mask1 != current_mask1 {
                        self.candidates_cache.set(r1, c1, new_mask1);
                        unit_placements_made = true;
                        if new_mask1.count_ones() == 1 {
                            let num = new_mask1.trailing_zeros() as u8 + 1;
                            if self.masks.is_safe(r1, c1, num) {
                                self.place_and_update(r1, c1, num, path);
                            }
                        }
                    }

                    let current_mask2 = self.candidates_cache.get(r2, c2);
                    let new_mask2 = pair_mask;

                    if new_mask2 != current_mask2 {
                        self.candidates_cache.set(r2, c2, new_mask2);
                        unit_placements_made = true;
                        if new_mask2.count_ones() == 1 {
                            let num = new_mask2.trailing_zeros() as u8 + 1;
                            if self.masks.is_safe(r2, c2, num) {
                                self.place_and_update(r2, c2, num, path);
                            }
                        }
                    }
                }
            }
        }
        unit_placements_made
    }

    /// Applies deterministic constraint propagation techniques iteratively.
    pub fn propagate_constraints(
        &mut self,
        path: &mut Vec<(usize, usize, u8)>,
        initial_path_len: usize,
    ) -> bool {
        loop {
            let mut changed_this_iter = false;

            if self
                .techniques_enabled
                .contains(RustokuTechniques::NAKED_SINGLES)
            {
                changed_this_iter |= self.naked_singles(path);
            }
            if self
                .techniques_enabled
                .contains(RustokuTechniques::HIDDEN_SINGLES)
            {
                changed_this_iter |= self.hidden_singles(path);
            }
            if self
                .techniques_enabled
                .contains(RustokuTechniques::NAKED_PAIRS)
            {
                changed_this_iter |= self.naked_pairs(path);
            }
            if self
                .techniques_enabled
                .contains(RustokuTechniques::HIDDEN_PAIRS)
            {
                changed_this_iter |= self.hidden_pairs(path);
            }

            // Contradiction check
            for r in 0..9 {
                for c in 0..9 {
                    if self.board.is_empty(r, c) && self.candidates_cache.get(r, c) == 0 {
                        // Contradiction: Roll back placements from this propagation call
                        while path.len() > initial_path_len {
                            let (r, c, num) = path.pop().unwrap();
                            // Need to remove from masks and recalculate candidates on removal
                            self.remove_and_update(r, c, num);
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
