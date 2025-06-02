/// Masks for Rustoku puzzle, representing the state of rows, columns, and boxes.
///
/// This struct holds bitmasks for each row, column, and 3x3 box in the Rustoku board.
/// Each bit in the masks corresponds to a number from 1 to 9, where a bit set to 1 indicates
/// that the corresponding number is present in that row, column, or box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Masks {
    pub(super) row_masks: [u16; 9],
    pub(super) col_masks: [u16; 9],
    pub(super) box_masks: [u16; 9],
}

impl Masks {
    pub(super) fn new() -> Self {
        Masks {
            row_masks: [0; 9],
            col_masks: [0; 9],
            box_masks: [0; 9],
        }
    }

    /// Computes the index of the 3x3 box based on the row and column indices.
    pub(super) fn get_box_idx(r: usize, c: usize) -> usize {
        (r / 3) * 3 + (c / 3)
    }

    /// Adds a number to the masks for the specified row, column, and box.
    pub(super) fn add_number(&mut self, r: usize, c: usize, num: u8) {
        let bit_to_set = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);
        self.row_masks[r] |= bit_to_set;
        self.col_masks[c] |= bit_to_set;
        self.box_masks[box_idx] |= bit_to_set;
    }

    /// Removes a number from the masks for the specified row, column, and box.
    pub(super) fn remove_number(&mut self, r: usize, c: usize, num: u8) {
        let bit_to_unset = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);
        self.row_masks[r] &= !bit_to_unset;
        self.col_masks[c] &= !bit_to_unset;
        self.box_masks[box_idx] &= !bit_to_unset;
    }

    /// Checks if a number can be safely placed in the specified cell.
    pub(super) fn is_safe(&self, r: usize, c: usize, num: u8) -> bool {
        let bit_to_check = 1 << (num - 1);
        let box_idx = Self::get_box_idx(r, c);

        (self.row_masks[r] & bit_to_check == 0)
            && (self.col_masks[c] & bit_to_check == 0)
            && (self.box_masks[box_idx] & bit_to_check == 0)
    }

    /// Computes the candidates mask for a specific cell based on the current masks.
    pub(super) fn compute_candidates_mask_for_cell(&self, r: usize, c: usize) -> u16 {
        let row_mask = self.row_masks[r];
        let col_mask = self.col_masks[c];
        let box_mask = self.box_masks[Self::get_box_idx(r, c)];
        let used = row_mask | col_mask | box_mask;
        !used & 0x1FF
    }
}
