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
    pub(super) row_masks: [u16; 9],
    /// Bitmask to track used numbers in each column (1-9 mapped to bits 0-8)
    pub(super) col_masks: [u16; 9],
    /// Bitmask to track used numbers in each 3x3 box (1-9 mapped to bits 0-8)
    pub(super) box_masks: [u16; 9],
    /// Cache of possible candidates for each cell (0 if cell is filled).
    /// Indexed by [row][col].
    pub(super) candidates_cache: [[u16; 9]; 9],
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
        for r in 0..9 {
            for c in 0..9 {
                let num = initial_board[r][c];
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

/// Validation methods for Sudoku puzzles.
impl Rustoku {
    /// Checks if the Sudoku puzzle is solved correctly.
    ///
    /// A puzzle is considered solved if all cells are filled and the board does not
    /// contain duplicates across rows, columns, and 3x3 boxes.
    pub fn is_solved(&self) -> bool {
        if self.board.iter().flatten().any(|&val| val == 0) {
            return false;
        }
        Rustoku::new(self.board).is_ok()
    }
}

/// Board manipulation methods for Sudoku puzzles.
impl Rustoku {
    /// Returns the index of the 3x3 box for a given row and column.
    pub(super) fn get_box_idx(r: usize, c: usize) -> usize {
        (r / 3) * 3 + (c / 3)
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

    /// Returns a bitmask of possible candidates for a given empty cell.
    ///
    /// Now retrieves from the pre-computed cache.
    pub(super) fn get_possible_candidates_mask(&self, r: usize, c: usize) -> u16 {
        self.candidates_cache[r][c]
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
