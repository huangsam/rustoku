use super::board::RustokuBoard;
use super::candidates::RustokuCandidates;
use super::masks::RustokuMasks;
use bitflags::bitflags;

bitflags! {
    /// A bitmask to control which human techniques are applied.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RustokuTechniques: u16 {
        /// No specific techniques are applied.
        const NONE = 0;
        /// Apply the naked singles technique.
        const NAKED_SINGLES = 1 << 0;
        /// Apply the hidden singles technique.
        const HIDDEN_SINGLES = 1 << 1;
        /// Apply the naked pairs technique.
        const NAKED_PAIRS = 1 << 2;
        /// Apply the hidden pairs technique.
        const HIDDEN_PAIRS = 1 << 3;
        /// Apply the locked candidates technique.
        const LOCKED_CANDIDATES = 1 << 4;
        /// Apply the X-Wing technique.
        const X_WING = 1 << 5;

        /// Apply easy techniques like naked singles and hidden singles.
        const EASY = Self::NAKED_SINGLES.bits() | Self::HIDDEN_SINGLES.bits();
        /// Apply medium techniques like naked pairs and hidden pairs.
        const MEDIUM = Self::NAKED_PAIRS.bits() | Self::HIDDEN_PAIRS.bits();
        /// Apply hard techniques like locked candidates and X-Wings.
        const HARD = Self::LOCKED_CANDIDATES.bits() | Self::X_WING.bits();
        /// Apply all available human-like techniques
        const ALL = Self::EASY.bits() | Self::MEDIUM.bits() | Self::HARD.bits();
    }
}

// Now the actual implementation of the techniques, these would operate on
// references to Board, Masks, and CandidatesCache.
pub(super) struct TechniquePropagator<'a> {
    board: &'a mut RustokuBoard,
    masks: &'a mut RustokuMasks,
    candidates_cache: &'a mut RustokuCandidates,
    techniques_enabled: RustokuTechniques,
}

impl<'a> TechniquePropagator<'a> {
    pub(super) fn new(
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

    /// Helper to place a number and update caches.
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

    /// Helper to remove a number and update caches.
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

    /// Applies the locked candidates technique.
    fn locked_candidates(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut overall_placements_made = false;

        // Check rows for pointing pairs/triples
        for row in 0..9 {
            overall_placements_made |= self.process_row_for_locked_candidates(row, path);
        }

        // Check columns for pointing pairs/triples
        for col in 0..9 {
            overall_placements_made |= self.process_col_for_locked_candidates(col, path);
        }

        // Check boxes for box/line reduction
        for box_idx in 0..9 {
            overall_placements_made |= self.process_box_for_locked_candidates(box_idx, path);
        }

        overall_placements_made
    }

    /// Helper function to process a row for locked candidates (pointing pairs/triples).
    fn process_row_for_locked_candidates(
        &mut self,
        row: usize,
        path: &mut Vec<(usize, usize, u8)>,
    ) -> bool {
        let mut placements_made = false;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            // Find cells in the row that contain the candidate
            let candidate_cells: Vec<usize> = (0..9)
                .filter(|&col| {
                    self.board.is_empty(row, col)
                        && (self.candidates_cache.get(row, col) & candidate_bit) != 0
                })
                .collect();

            // If the candidate only appears in one box, it's a pointing pair/triple
            let boxes: std::collections::HashSet<usize> = candidate_cells
                .iter()
                .map(|&col| (row / 3) * 3 + (col / 3))
                .collect();

            if boxes.len() == 1 {
                let box_idx = *boxes.iter().next().unwrap();
                let start_row = (box_idx / 3) * 3;
                let start_col = (box_idx % 3) * 3;

                // Remove the candidate from other cells in the box, but not in the row
                for r in start_row..(start_row + 3) {
                    for c in start_col..(start_col + 3) {
                        if r != row && self.board.is_empty(r, c) {
                            let initial_mask = self.candidates_cache.get(r, c);
                            if (initial_mask & candidate_bit) != 0 {
                                let refined_mask = initial_mask & !candidate_bit;
                                self.candidates_cache.set(r, c, refined_mask);
                                placements_made = true;

                                if refined_mask.count_ones() == 1 {
                                    let num = refined_mask.trailing_zeros() as u8 + 1;
                                    if self.masks.is_safe(r, c, num) {
                                        self.place_and_update(r, c, num, path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        placements_made
    }

    /// Helper function to process a column for locked candidates (pointing pairs/triples).
    fn process_col_for_locked_candidates(
        &mut self,
        col: usize,
        path: &mut Vec<(usize, usize, u8)>,
    ) -> bool {
        let mut placements_made = false;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            // Find cells in the column that contain the candidate
            let candidate_cells: Vec<usize> = (0..9)
                .filter(|&row| {
                    self.board.is_empty(row, col)
                        && (self.candidates_cache.get(row, col) & candidate_bit) != 0
                })
                .collect();

            // If the candidate only appears in one box, it's a pointing pair/triple
            let boxes: std::collections::HashSet<usize> = candidate_cells
                .iter()
                .map(|&row| (row / 3) * 3 + (col / 3))
                .collect();

            if boxes.len() == 1 {
                let box_idx = *boxes.iter().next().unwrap();
                let start_row = (box_idx / 3) * 3;
                let start_col = (box_idx % 3) * 3;

                // Remove the candidate from other cells in the box, but not in the column
                for r in start_row..(start_row + 3) {
                    for c in start_col..(start_col + 3) {
                        if c != col && self.board.is_empty(r, c) {
                            let initial_mask = self.candidates_cache.get(r, c);
                            if (initial_mask & candidate_bit) != 0 {
                                let refined_mask = initial_mask & !candidate_bit;
                                self.candidates_cache.set(r, c, refined_mask);
                                placements_made = true;

                                if refined_mask.count_ones() == 1 {
                                    let num = refined_mask.trailing_zeros() as u8 + 1;
                                    if self.masks.is_safe(r, c, num) {
                                        self.place_and_update(r, c, num, path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        placements_made
    }

    /// Helper function to process a box for locked candidates (box/line reduction).
    fn process_box_for_locked_candidates(
        &mut self,
        box_idx: usize,
        path: &mut Vec<(usize, usize, u8)>,
    ) -> bool {
        let mut placements_made = false;
        let start_row = (box_idx / 3) * 3;
        let start_col = (box_idx % 3) * 3;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            // Find cells in the box that contain the candidate
            let mut candidate_cells: Vec<(usize, usize)> = Vec::new();
            for r_offset in 0..3 {
                for c_offset in 0..3 {
                    let r = start_row + r_offset;
                    let c = start_col + c_offset;
                    if self.board.is_empty(r, c)
                        && (self.candidates_cache.get(r, c) & candidate_bit) != 0
                    {
                        candidate_cells.push((r, c));
                    }
                }
            }

            // Check if all candidates are in the same row
            let rows: std::collections::HashSet<usize> =
                candidate_cells.iter().map(|&(r, _)| r).collect();
            if rows.len() == 1 {
                let row = *rows.iter().next().unwrap();

                // Remove candidate from other cells in the row, but not in the box
                for c in 0..9 {
                    if (c < start_col || c >= start_col + 3) && self.board.is_empty(row, c) {
                        let initial_mask = self.candidates_cache.get(row, c);
                        if (initial_mask & candidate_bit) != 0 {
                            let refined_mask = initial_mask & !candidate_bit;
                            self.candidates_cache.set(row, c, refined_mask);
                            placements_made = true;

                            if refined_mask.count_ones() == 1 {
                                let num = refined_mask.trailing_zeros() as u8 + 1;
                                if self.masks.is_safe(row, c, num) {
                                    self.place_and_update(row, c, num, path);
                                }
                            }
                        }
                    }
                }
            }

            // Check if all candidates are in the same column
            let cols: std::collections::HashSet<usize> =
                candidate_cells.iter().map(|&(_, c)| c).collect();
            if cols.len() == 1 {
                let col = *cols.iter().next().unwrap();

                // Remove candidate from other cells in the column, but not in the box
                for r in 0..9 {
                    if (r < start_row || r >= start_row + 3) && self.board.is_empty(r, col) {
                        let initial_mask = self.candidates_cache.get(r, col);
                        if (initial_mask & candidate_bit) != 0 {
                            let refined_mask = initial_mask & !candidate_bit;
                            self.candidates_cache.set(r, col, refined_mask);
                            placements_made = true;

                            if refined_mask.count_ones() == 1 {
                                let num = refined_mask.trailing_zeros() as u8 + 1;
                                if self.masks.is_safe(r, col, num) {
                                    self.place_and_update(r, col, num, path);
                                }
                            }
                        }
                    }
                }
            }
        }

        placements_made
    }

    /// Applies the X-Wing technique.
    fn x_wing(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut placements_made = false;

        for candidate_val in 1..=9 {
            let candidate_bit = 1 << (candidate_val - 1);

            // Check for row-based X-Wings
            let mut rows_with_two_candidates: Vec<usize> = Vec::new();
            let mut candidate_cols_in_rows: Vec<Vec<usize>> = Vec::new();

            for r in 0..9 {
                let mut cols_for_candidate_in_row: Vec<usize> = Vec::new();
                for c in 0..9 {
                    if self.board.is_empty(r, c)
                        && (self.candidates_cache.get(r, c) & candidate_bit) != 0
                    {
                        cols_for_candidate_in_row.push(c);
                    }
                }
                if cols_for_candidate_in_row.len() == 2 {
                    rows_with_two_candidates.push(r);
                    candidate_cols_in_rows.push(cols_for_candidate_in_row);
                }
            }

            for i in 0..rows_with_two_candidates.len() {
                for j in (i + 1)..rows_with_two_candidates.len() {
                    let r1 = rows_with_two_candidates[i];
                    let r2 = rows_with_two_candidates[j];
                    let cols1 = &candidate_cols_in_rows[i];
                    let cols2 = &candidate_cols_in_rows[j];

                    if cols1[0] == cols2[0] && cols1[1] == cols2[1] {
                        let c1 = cols1[0];
                        let c2 = cols1[1];

                        // Found an X-Wing in columns c1 and c2 across rows r1 and r2
                        // Remove candidate from other cells in column c1 (excluding r1, r2)
                        for r_other in 0..9 {
                            if r_other != r1 && r_other != r2 && self.board.is_empty(r_other, c1) {
                                let initial_mask = self.candidates_cache.get(r_other, c1);
                                if (initial_mask & candidate_bit) != 0 {
                                    let refined_mask = initial_mask & !candidate_bit;
                                    self.candidates_cache.set(r_other, c1, refined_mask);
                                    placements_made = true;
                                    if refined_mask.count_ones() == 1 {
                                        let num = refined_mask.trailing_zeros() as u8 + 1;
                                        if self.masks.is_safe(r_other, c1, num) {
                                            self.place_and_update(r_other, c1, num, path);
                                        }
                                    }
                                }
                            }
                        }

                        // Remove candidate from other cells in column c2 (excluding r1, r2)
                        for r_other in 0..9 {
                            if r_other != r1 && r_other != r2 && self.board.is_empty(r_other, c2) {
                                let initial_mask = self.candidates_cache.get(r_other, c2);
                                if (initial_mask & candidate_bit) != 0 {
                                    let refined_mask = initial_mask & !candidate_bit;
                                    self.candidates_cache.set(r_other, c2, refined_mask);
                                    placements_made = true;
                                    if refined_mask.count_ones() == 1 {
                                        let num = refined_mask.trailing_zeros() as u8 + 1;
                                        if self.masks.is_safe(r_other, c2, num) {
                                            self.place_and_update(r_other, c2, num, path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Check for column-based X-Wings (symmetric to row-based)
            let mut cols_with_two_candidates: Vec<usize> = Vec::new();
            let mut candidate_rows_in_cols: Vec<Vec<usize>> = Vec::new();

            for c in 0..9 {
                let mut rows_for_candidate_in_col: Vec<usize> = Vec::new();
                for r in 0..9 {
                    if self.board.is_empty(r, c)
                        && (self.candidates_cache.get(r, c) & candidate_bit) != 0
                    {
                        rows_for_candidate_in_col.push(r);
                    }
                }
                if rows_for_candidate_in_col.len() == 2 {
                    cols_with_two_candidates.push(c);
                    candidate_rows_in_cols.push(rows_for_candidate_in_col);
                }
            }

            for i in 0..cols_with_two_candidates.len() {
                for j in (i + 1)..cols_with_two_candidates.len() {
                    let c1 = cols_with_two_candidates[i];
                    let c2 = cols_with_two_candidates[j];
                    let rows1 = &candidate_rows_in_cols[i];
                    let rows2 = &candidate_rows_in_cols[j];

                    if rows1[0] == rows2[0] && rows1[1] == rows2[1] {
                        let r1 = rows1[0];
                        let r2 = rows1[1];

                        // Found an X-Wing in rows r1 and r2 across columns c1 and c2
                        // Remove candidate from other cells in row r1 (excluding c1, c2)
                        for c_other in 0..9 {
                            if c_other != c1 && c_other != c2 && self.board.is_empty(r1, c_other) {
                                let initial_mask = self.candidates_cache.get(r1, c_other);
                                if (initial_mask & candidate_bit) != 0 {
                                    let refined_mask = initial_mask & !candidate_bit;
                                    self.candidates_cache.set(r1, c_other, refined_mask);
                                    placements_made = true;
                                    if refined_mask.count_ones() == 1 {
                                        let num = refined_mask.trailing_zeros() as u8 + 1;
                                        if self.masks.is_safe(r1, c_other, num) {
                                            self.place_and_update(r1, c_other, num, path);
                                        }
                                    }
                                }
                            }
                        }

                        // Remove candidate from other cells in row r2 (excluding c1, c2)
                        for c_other in 0..9 {
                            if c_other != c1 && c_other != c2 && self.board.is_empty(r2, c_other) {
                                let initial_mask = self.candidates_cache.get(r2, c_other);
                                if (initial_mask & candidate_bit) != 0 {
                                    let refined_mask = initial_mask & !candidate_bit;
                                    self.candidates_cache.set(r2, c_other, refined_mask);
                                    placements_made = true;
                                    if refined_mask.count_ones() == 1 {
                                        let num = refined_mask.trailing_zeros() as u8 + 1;
                                        if self.masks.is_safe(r2, c_other, num) {
                                            self.place_and_update(r2, c_other, num, path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        placements_made
    }

    /// Applies deterministic constraint propagation techniques iteratively.
    pub(super) fn propagate_constraints(
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
            if !changed_this_iter
                && self
                    .techniques_enabled
                    .contains(RustokuTechniques::HIDDEN_SINGLES)
            {
                changed_this_iter |= self.hidden_singles(path);
            }
            if !changed_this_iter
                && self
                    .techniques_enabled
                    .contains(RustokuTechniques::NAKED_PAIRS)
            {
                changed_this_iter |= self.naked_pairs(path);
            }
            if !changed_this_iter
                && self
                    .techniques_enabled
                    .contains(RustokuTechniques::HIDDEN_PAIRS)
            {
                changed_this_iter |= self.hidden_pairs(path);
            }
            if !changed_this_iter
                && self
                    .techniques_enabled
                    .contains(RustokuTechniques::LOCKED_CANDIDATES)
            {
                changed_this_iter |= self.locked_candidates(path);
            }
            if !changed_this_iter && self.techniques_enabled.contains(RustokuTechniques::X_WING) {
                changed_this_iter |= self.x_wing(path);
            }

            // Contradiction check
            if (0..9).any(|r| {
                (0..9).any(|c| self.board.is_empty(r, c) && self.candidates_cache.get(r, c) == 0)
            }) {
                // Contradiction: Roll back placements from this propagation call
                while path.len() > initial_path_len {
                    if let Some((r, c, num)) = path.pop() {
                        self.remove_and_update(r, c, num);
                    }
                }
                return false; // Propagation failed
            }

            if !changed_this_iter {
                break; // Stable state
            }
        }
        true // Propagation successful
    }
}
