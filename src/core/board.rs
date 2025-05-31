use crate::error::RustokuError;

/// A core Sudoku primitive that uses backtracking and bitmasking for constraints.
///
/// This struct supports the ability to:
/// - Initialize from a 2D array, a flat byte array, or a string representation
/// - Solve a Sudoku puzzle using backtracking with Minimum Remaining Values (MRV)
/// - Generate a Sudoku puzzle with a unique solution based on the number of clues specified
/// - Check if a Sudoku puzzle is solved correctly
///
/// # Examples
///
/// Solve a Sudoku puzzle:
/// ```
/// use rustoku::core::Rustoku;
/// let board = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
/// let mut rustoku = Rustoku::try_from(board).unwrap();
/// assert!(rustoku.solve_any().is_some());
/// ```
///
/// Generate a Sudoku puzzle:
/// ```
/// use rustoku::core::Rustoku;
/// let puzzle = Rustoku::generate(30).unwrap();
/// let solution = Rustoku::new(puzzle).unwrap().solve_all();
/// assert_eq!(solution.len(), 1);
/// ```
///
/// Check if a Sudoku puzzle is solved:
/// ```
/// use rustoku::core::Rustoku;
/// let board = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
/// let rustoku = Rustoku::try_from(board).unwrap();
/// assert!(rustoku.is_solved());
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Rustoku {
    /// The Sudoku board itself, where 0 represents an empty cell
    pub board: [[u8; 9]; 9],
    /// Bitmask to track used numbers in each row (1-9 mapped to bits 0-8)
    row_masks: [u16; 9],
    /// Bitmask to track used numbers in each column (1-9 mapped to bits 0-8)
    col_masks: [u16; 9],
    /// Bitmask to track used numbers in each 3x3 box (1-9 mapped to bits 0-8)
    box_masks: [u16; 9],
    /// Cache of possible candidates for each cell (0 if cell is filled).
    /// Indexed by [row][col].
    candidates_cache: [[u16; 9]; 9],
}

impl TryFrom<[u8; 81]> for Rustoku {
    type Error = RustokuError;

    fn try_from(bytes: [u8; 81]) -> Result<Self, Self::Error> {
        let mut board = [[0u8; 9]; 9];
        for i in 0..81 {
            board[i / 9][i % 9] = bytes[i];
        }
        Self::new(board)
    }
}

impl TryFrom<&str> for Rustoku {
    type Error = RustokuError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.len() != 81 {
            return Err(RustokuError::InvalidInputLength);
        }
        let mut bytes = [0u8; 81];
        for (i, ch) in s.bytes().enumerate() {
            match ch {
                b'0'..=b'9' => bytes[i] = ch - b'0',
                b'.' | b'_' => bytes[i] = 0, // Treat '.' and '_' as empty cells
                _ => return Err(RustokuError::InvalidInputCharacter),
            }
        }
        bytes.try_into()
    }
}

/// Initialization methods for Sudoku puzzles.
impl Rustoku {
    pub fn new(initial_board: [[u8; 9]; 9]) -> Result<Self, RustokuError> {
        let mut rustoku = Self {
            board: initial_board,
            row_masks: [0; 9],
            col_masks: [0; 9],
            box_masks: [0; 9],
            candidates_cache: [[0; 9]; 9], // Initialize with zeros
        };

        // Initialize the masks based on the given initial board
        for (r, row) in initial_board.iter().enumerate() {
            for (c, &num) in row.iter().enumerate() {
                if num != 0 {
                    if !rustoku.is_safe(r, c, num) {
                        return Err(RustokuError::DuplicateValues);
                    }
                    // Important: Use the new `place_number` which updates the cache
                    rustoku.place_number(r, c, num);
                }
            }
        }
        // After initial placement, fill the cache for empty cells
        for r in 0..9 {
            for c in 0..9 {
                if rustoku.board[r][c] == 0 {
                    rustoku.candidates_cache[r][c] =
                        rustoku.calculate_candidates_mask_for_cell(r, c);
                }
            }
        }

        Ok(rustoku)
    }

    /// Calculate the mask without using the cache.
    ///
    /// Computes the bitmask of legal candidates for a cell by combining the row, column,
    /// and box masks, inverting, and masking with `0x1FF` to keep only the lower 9 bits.
    fn calculate_candidates_mask_for_cell(&self, r: usize, c: usize) -> u16 {
        let row_mask = self.row_masks[r];
        let col_mask = self.col_masks[c];
        let box_mask = self.box_masks[Self::get_box_idx(r, c)];
        let used = row_mask | col_mask | box_mask;
        !used & 0x1FF
    }
}

/// Operation methods for Sudoku puzzles.
impl Rustoku {
    /// Returns the index of the 3x3 box for a given row and column.
    fn get_box_idx(r: usize, c: usize) -> usize {
        (r / 3) * 3 + (c / 3)
    }

    /// Returns a bitmask of possible candidates for a given empty cell.
    fn get_possible_candidates_mask(&self, r: usize, c: usize) -> u16 {
        self.candidates_cache[r][c]
    }

    /// Checks if placing a number in the given cell is safe according to Sudoku rules.
    pub(super) fn is_safe(&self, r: usize, c: usize, num: u8) -> bool {
        let bit_to_check = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);

        (self.row_masks[r] & bit_to_check == 0)
            && (self.col_masks[c] & bit_to_check == 0)
            && (self.box_masks[box_idx] & bit_to_check == 0)
    }

    /// Places a number in the Sudoku board and updates the corresponding masks.
    pub(super) fn place_number(&mut self, r: usize, c: usize, num: u8) {
        let bit_to_set = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);

        self.board[r][c] = num;
        self.row_masks[r] |= bit_to_set;
        self.col_masks[c] |= bit_to_set;
        self.box_masks[box_idx] |= bit_to_set;

        // Invalidate/update cache for the placed cell
        self.candidates_cache[r][c] = 0; // No candidates for a filled cell

        // Update cache for affected row, column, and box
        for i in 0..9 {
            if self.board[r][i] == 0 {
                // Only update if cell is empty
                self.candidates_cache[r][i] = self.calculate_candidates_mask_for_cell(r, i);
            }
            if self.board[i][c] == 0 {
                // Only update if cell is empty
                self.candidates_cache[i][c] = self.calculate_candidates_mask_for_cell(i, c);
            }
        }

        // Update box cells
        let start_row = (box_idx / 3) * 3;
        let start_col = (box_idx % 3) * 3;
        for r_offset in 0..3 {
            for c_offset in 0..3 {
                let cur_r = start_row + r_offset;
                let cur_c = start_col + c_offset;
                if self.board[cur_r][cur_c] == 0 {
                    // Only update if cell is empty
                    self.candidates_cache[cur_r][cur_c] =
                        self.calculate_candidates_mask_for_cell(cur_r, cur_c);
                }
            }
        }
    }

    /// Removes a number from the Sudoku board and updates the masks accordingly.
    pub(super) fn remove_number(&mut self, r: usize, c: usize, num: u8) {
        let bit_to_unset = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);

        self.board[r][c] = 0; // Set back to empty
        self.row_masks[r] &= !bit_to_unset;
        self.col_masks[c] &= !bit_to_unset;
        self.box_masks[box_idx] &= !bit_to_unset;

        // Re-calculate cache for the removed cell (it's now empty)
        self.candidates_cache[r][c] = self.calculate_candidates_mask_for_cell(r, c);

        // Update cache for affected row, column, and box
        for i in 0..9 {
            if self.board[r][i] == 0 {
                self.candidates_cache[r][i] = self.calculate_candidates_mask_for_cell(r, i);
            }
            if self.board[i][c] == 0 {
                self.candidates_cache[i][c] = self.calculate_candidates_mask_for_cell(i, c);
            }
        }

        // Update box cells
        let start_row = (box_idx / 3) * 3;
        let start_col = (box_idx % 3) * 3;
        for r_offset in 0..3 {
            for c_offset in 0..3 {
                let cur_r = start_row + r_offset;
                let cur_c = start_col + c_offset;
                if self.board[cur_r][cur_c] == 0 {
                    self.candidates_cache[cur_r][cur_c] =
                        self.calculate_candidates_mask_for_cell(cur_r, cur_c);
                }
            }
        }
    }

    /// Finds the next empty cell in the Sudoku board using MRV (Minimum Remaining Values).
    pub(super) fn find_next_empty_cell(&self) -> Option<(usize, usize)> {
        let mut min = (10, None);
        for r in 0..9 {
            for c in 0..9 {
                if self.board[r][c] == 0 {
                    let count = self.get_possible_candidates_mask(r, c).count_ones() as u8;
                    if count < min.0 {
                        min = (count, Some((r, c)));
                        if count == 1 {
                            return min.1;
                        }
                    }
                }
            }
        }
        min.1
    }
}

/// Propagation methods for Sudoku puzzles.
impl Rustoku {
    /// Applies the naked singles technique.
    ///
    /// A naked single occurs if an empty cell has only one possible candidate.
    /// This technique finds such cells and places the single candidate.
    fn naked_singles(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
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
    fn hidden_singles(&mut self, path: &mut Vec<(usize, usize, u8)>) -> bool {
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
