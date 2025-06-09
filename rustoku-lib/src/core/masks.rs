/// Masks for Rustoku puzzle, representing the state of rows, columns, and boxes.
///
/// This struct holds bitmasks for each row, column, and 3x3 box in the Rustoku board.
/// Each bit in the masks corresponds to a number from 1 to 9, where a bit set to 1 indicates
/// that the corresponding number is present in that row, column, or box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Masks {
    row_masks: [u16; 9],
    col_masks: [u16; 9],
    box_masks: [u16; 9],
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
    pub fn is_safe(&self, r: usize, c: usize, num: u8) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper for creating bitmasks for expected values
    fn bit(n: u8) -> u16 {
        1u16 << (n - 1)
    }

    // Helper for combining bits for expected values
    fn bits(nums: &[u8]) -> u16 {
        nums.iter().map(|&n| bit(n)).fold(0, |acc, b| acc | b)
    }

    #[test]
    fn test_new_initializes_empty_masks() {
        let masks = Masks::new();
        assert_eq!(masks.row_masks, [0; 9]);
        assert_eq!(masks.col_masks, [0; 9]);
        assert_eq!(masks.box_masks, [0; 9]);
    }

    #[test]
    fn test_get_box_idx_top_left_box() {
        assert_eq!(Masks::get_box_idx(0, 0), 0);
        assert_eq!(Masks::get_box_idx(1, 1), 0);
        assert_eq!(Masks::get_box_idx(2, 2), 0);
    }

    #[test]
    fn test_get_box_idx_middle_box() {
        assert_eq!(Masks::get_box_idx(3, 3), 4);
        assert_eq!(Masks::get_box_idx(4, 4), 4);
        assert_eq!(Masks::get_box_idx(5, 5), 4);
    }

    #[test]
    fn test_get_box_idx_bottom_right_box() {
        assert_eq!(Masks::get_box_idx(6, 6), 8);
        assert_eq!(Masks::get_box_idx(7, 7), 8);
        assert_eq!(Masks::get_box_idx(8, 8), 8);
    }

    #[test]
    fn test_get_box_idx_various_boxes() {
        assert_eq!(Masks::get_box_idx(0, 3), 1); // Top-middle
        assert_eq!(Masks::get_box_idx(3, 0), 3); // Middle-left
        assert_eq!(Masks::get_box_idx(0, 8), 2); // Top-right
        assert_eq!(Masks::get_box_idx(8, 0), 6); // Bottom-left
    }

    #[test]
    fn test_add_number_single_cell() {
        let mut masks = Masks::new();
        masks.add_number(0, 0, 1);
        assert_eq!(masks.row_masks[0], bit(1));
        assert_eq!(masks.col_masks[0], bit(1));
        assert_eq!(masks.box_masks[0], bit(1));
    }

    #[test]
    fn test_add_number_multiple_in_same_row() {
        let mut masks = Masks::new();
        masks.add_number(0, 0, 1);
        masks.add_number(0, 1, 5);
        assert_eq!(masks.row_masks[0], bits(&[1, 5]));
        assert_eq!(masks.col_masks[0], bit(1));
        assert_eq!(masks.col_masks[1], bit(5));
        assert_eq!(masks.box_masks[0], bits(&[1, 5]));
    }

    #[test]
    fn test_add_number_to_different_box() {
        let mut masks = Masks::new();
        masks.add_number(8, 8, 9);
        assert_eq!(masks.row_masks[8], bit(9));
        assert_eq!(masks.col_masks[8], bit(9));
        assert_eq!(masks.box_masks[8], bit(9));
    }

    #[test]
    fn test_add_number_already_present_no_change() {
        let mut masks = Masks::new();
        masks.add_number(0, 0, 1);
        let initial_row_0 = masks.row_masks[0];
        masks.add_number(0, 0, 1); // Adding again
        assert_eq!(masks.row_masks[0], initial_row_0);
    }

    #[test]
    fn test_remove_number_single_value_from_cell() {
        let mut masks = Masks::new();
        masks.add_number(0, 0, 1); // Add 1
        masks.remove_number(0, 0, 1); // Remove 1
        assert_eq!(masks.row_masks[0], 0);
        assert_eq!(masks.col_masks[0], 0);
        assert_eq!(masks.box_masks[0], 0);
    }

    #[test]
    fn test_remove_number_from_shared_row() {
        let mut masks = Masks::new();
        masks.add_number(0, 0, 1);
        masks.add_number(0, 1, 5);
        masks.remove_number(0, 0, 1);
        assert_eq!(masks.row_masks[0], bit(5)); // Only 5 should remain in row 0
        assert_eq!(masks.col_masks[0], 0); // Col 0 should be clear
        assert_eq!(masks.col_masks[1], bit(5)); // Col 1 should still have 5
        assert_eq!(masks.box_masks[0], bit(5)); // Box 0 should only have 5
    }

    #[test]
    fn test_remove_number_not_present_no_change() {
        let mut masks = Masks::new();
        masks.add_number(0, 0, 1);
        let initial_row_0 = masks.row_masks[0];
        masks.remove_number(0, 0, 3); // Remove 3 (not present)
        assert_eq!(masks.row_masks[0], initial_row_0);
    }

    #[test]
    fn test_is_safe_on_empty_board_always_true() {
        let masks = Masks::new();
        assert!(masks.is_safe(0, 0, 1)); // 1 should be safe anywhere
        assert!(masks.is_safe(8, 8, 9)); // 9 should be safe anywhere
    }

    #[test]
    fn test_is_safe_conflict_in_row() {
        let mut masks = Masks::new();
        masks.add_number(0, 0, 1); // Place 1 at (0,0)
        assert!(!masks.is_safe(0, 1, 1)); // 1 should not be safe in same row
        assert!(masks.is_safe(0, 1, 2)); // 2 should be safe in same row
    }

    #[test]
    fn test_is_safe_conflict_in_column() {
        let mut masks = Masks::new();
        masks.add_number(0, 0, 1); // Place 1 at (0,0)
        assert!(!masks.is_safe(1, 0, 1)); // 1 should not be safe in same col
        assert!(masks.is_safe(1, 0, 2)); // 2 should be safe in same col
    }

    #[test]
    fn test_is_safe_conflict_in_box() {
        let mut masks = Masks::new();
        masks.add_number(1, 1, 1); // Place 1 at (1,1) in box 0
        assert!(!masks.is_safe(0, 0, 1)); // 1 should not be safe in same box
        assert!(masks.is_safe(0, 0, 2)); // 2 should be safe in same box
    }

    #[test]
    fn test_is_safe_conflict_with_current_cell_value() {
        let mut masks = Masks::new();
        masks.add_number(0, 0, 1);
        assert!(!masks.is_safe(0, 0, 1)); // Should not be safe to place 1 where 1 already is
        assert!(masks.is_safe(0, 0, 2)); // Should be safe for other numbers
    }

    #[test]
    fn test_compute_candidates_empty_cell_all_available() {
        let masks = Masks::new();
        let candidates = masks.compute_candidates_mask_for_cell(0, 0);
        assert_eq!(candidates, 0x1FF); // All 9 bits should be set
    }

    #[test]
    fn test_compute_candidates_row_has_1_to_8_only_9_available() {
        let mut masks = Masks::new();
        masks.row_masks[0] = bits(&[1, 2, 3, 4, 5, 6, 7, 8]); // Directly set mask
        let candidates = masks.compute_candidates_mask_for_cell(0, 0);
        assert_eq!(candidates, bit(9));
    }

    #[test]
    fn test_compute_candidates_col_has_1_to_8_only_9_available() {
        let mut masks = Masks::new();
        masks.col_masks[0] = bits(&[1, 2, 3, 4, 5, 6, 7, 8]); // Directly set mask
        let candidates = masks.compute_candidates_mask_for_cell(0, 0);
        assert_eq!(candidates, bit(9));
    }

    #[test]
    fn test_compute_candidates_box_has_1_to_8_only_9_available() {
        let mut masks = Masks::new();
        masks.box_masks[Masks::get_box_idx(1, 1)] = bits(&[1, 2, 3, 4, 5, 6, 7, 8]); // Directly set mask for box 0
        let candidates = masks.compute_candidates_mask_for_cell(1, 1);
        assert_eq!(candidates, bit(9));
    }

    #[test]
    fn test_compute_candidates_mixed_restrictions() {
        let mut masks = Masks::new();
        // Row 0 has 1, 2
        masks.row_masks[0] = bits(&[1, 2]);
        // Col 0 has 3, 4
        masks.col_masks[0] = bits(&[3, 4]);
        // Box 0 (for 0,0) has 5, 6
        masks.box_masks[0] = bits(&[5, 6]);

        let candidates = masks.compute_candidates_mask_for_cell(0, 0);
        assert_eq!(candidates, bits(&[7, 8, 9])); // Should be 7, 8, 9
    }

    #[test]
    fn test_compute_candidates_no_candidates_left() {
        let mut masks = Masks::new();
        masks.row_masks[0] = 0x1FF; // All 1-9 used in row
        let candidates = masks.compute_candidates_mask_for_cell(0, 0);
        assert_eq!(candidates, 0); // No candidates
    }
}
