use super::board::Board;
use super::masks::Masks;

/// Represents the candidates cache for a Rustoku puzzle.
///
/// This struct holds a 9x9 grid of candidate masks for each cell in the Rustoku board.
/// Each cell's candidates are represented as a bitmask, where each bit corresponds to a number
/// from 1 to 9. A bit set to 1 indicates that the corresponding number is a candidate for that cell.
/// This struct provides methods to get and set candidate masks for specific cells, as well as to
/// update the candidates based on the current state of the board and masks.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Candidates {
    cache: [[u16; 9]; 9],
}

impl Candidates {
    pub(super) fn new() -> Self {
        Candidates { cache: [[0; 9]; 9] }
    }

    /// Returns the candidate mask for a specific cell in the cache.
    pub(super) fn get(&self, r: usize, c: usize) -> u16 {
        self.cache[r][c]
    }

    /// Returns the actual candidate numbers (1-9) for a specific cell.
    pub fn get_candidates(&self, r: usize, c: usize) -> Vec<u8> {
        let mask = self.get(r, c);
        let mut candidates = Vec::new();
        for i in 0..9 {
            // Check if the i-th bit is set (representing number i+1)
            if (mask >> i) & 1 == 1 {
                candidates.push((i + 1) as u8);
            }
        }
        candidates
    }

    /// Sets the candidate mask for a specific cell in the cache.
    pub(super) fn set(&mut self, r: usize, c: usize, mask: u16) {
        self.cache[r][c] = mask;
    }

    /// Update affected cells in the cache based on the current state of the board and masks.
    pub(super) fn update_affected_cells(
        &mut self,
        r: usize,
        c: usize,
        masks: &Masks,
        board: &Board,
    ) {
        // Invalidate/update cache for the placed cell
        self.cache[r][c] = 0; // No candidates for a filled cell

        // Update cache for affected row, column, and box
        for i in 0..9 {
            if board.is_empty(r, i) {
                self.cache[r][i] = masks.compute_candidates_mask_for_cell(r, i);
            }
            if board.is_empty(i, c) {
                self.cache[i][c] = masks.compute_candidates_mask_for_cell(i, c);
            }
        }

        // Update box cells
        let box_idx = Masks::get_box_idx(r, c);
        let start_row = (box_idx / 3) * 3;
        let start_col = (box_idx % 3) * 3;
        for r_offset in 0..3 {
            for c_offset in 0..3 {
                let cur_r = start_row + r_offset;
                let cur_c = start_col + c_offset;
                if board.is_empty(cur_r, cur_c) {
                    self.cache[cur_r][cur_c] = masks.compute_candidates_mask_for_cell(cur_r, cur_c);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_candidates_empty_mask() {
        let candidates = Candidates::new(); // Starts with all 0s
        let r = 0;
        let c = 0;
        let cands = candidates.get_candidates(r, c);
        assert_eq!(cands, vec![]);
    }

    #[test]
    fn test_get_candidates_full_mask() {
        let mut candidates = Candidates::new();
        let r = 0;
        let c = 0;
        // All bits from 0 to 8 set (representing numbers 1 to 9)
        // 0b00000001_11111111 = 511 (binary)
        let full_mask = (1 << 9) - 1; // Or 0b111111111
        candidates.set(r, c, full_mask);
        let cands = candidates.get_candidates(r, c);
        assert_eq!(cands, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_get_candidates_single_candidate() {
        let mut candidates = Candidates::new();
        let r = 1;
        let c = 2;

        // Test for candidate 1 (bit 0)
        candidates.set(r, c, 1 << 0); // Mask: 0b000000001
        let cands_1 = candidates.get_candidates(r, c);
        assert_eq!(cands_1, vec![1]);

        // Test for candidate 5 (bit 4)
        candidates.set(r, c, 1 << 4); // Mask: 0b000010000
        let cands_5 = candidates.get_candidates(r, c);
        assert_eq!(cands_5, vec![5]);

        // Test for candidate 9 (bit 8)
        candidates.set(r, c, 1 << 8); // Mask: 0b100000000
        let cands_9 = candidates.get_candidates(r, c);
        assert_eq!(cands_9, vec![9]);
    }

    #[test]
    fn test_get_candidates_multiple_candidates() {
        let mut candidates = Candidates::new();
        let r = 3;
        let c = 4;

        // Candidates: 2, 4, 7
        // Bit positions: 1, 3, 6
        // Mask: (1 << 1) | (1 << 3) | (1 << 6)
        // Mask: 0b01001010 = 2 + 8 + 64 = 74
        let mask = (1 << 1) | (1 << 3) | (1 << 6); // 0b01001010
        candidates.set(r, c, mask);
        let cands = candidates.get_candidates(r, c);
        assert_eq!(cands, vec![2, 4, 7]);

        // Candidates: 1, 9
        // Bit positions: 0, 8
        let mask_1_9 = (1 << 0) | (1 << 8); // 0b100000001
        candidates.set(r, c, mask_1_9);
        let cands_1_9 = candidates.get_candidates(r, c);
        assert_eq!(cands_1_9, vec![1, 9]);
    }

    #[test]
    fn test_get_candidates_different_cells() {
        let mut candidates = Candidates::new();

        // Set candidates for (0,0)
        candidates.set(0, 0, (1 << 0) | (1 << 2)); // Candidates 1, 3
        // Set candidates for (8,8)
        candidates.set(8, 8, (1 << 5) | (1 << 7)); // Candidates 6, 8

        assert_eq!(candidates.get_candidates(0, 0), vec![1, 3]);
        assert_eq!(candidates.get_candidates(8, 8), vec![6, 8]);
        assert_eq!(candidates.get_candidates(0, 1), vec![]); // Unset cell
    }
}
