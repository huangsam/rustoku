use super::board::RustokuBoard;
use super::masks::RustokuMasks;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RustokuCandidates {
    cache: [[u16; 9]; 9],
}

impl RustokuCandidates {
    pub fn new() -> Self {
        RustokuCandidates { cache: [[0; 9]; 9] }
    }

    pub fn get(&self, r: usize, c: usize) -> u16 {
        self.cache[r][c]
    }

    pub fn set(&mut self, r: usize, c: usize, mask: u16) {
        self.cache[r][c] = mask;
    }

    // Invalidation logic would live here, but would need to know about the `Masks` struct
    // or receive the calculated mask.
    pub fn update_affected_cells(
        &mut self,
        r: usize,
        c: usize,
        masks: &RustokuMasks,
        board: &RustokuBoard,
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
        let box_idx = RustokuMasks::get_box_idx(r, c);
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
