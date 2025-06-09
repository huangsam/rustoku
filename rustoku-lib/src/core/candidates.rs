use super::board::Board;
use super::masks::Masks;

/// Represents the candidates cache for a Rustoku puzzle.
///
/// This struct holds a 9x9 grid of candidate masks for each cell in the Rustoku board.
/// Each cell's candidates are represented as a bitmask, where each bit corresponds to a number
/// from 1 to 9. A bit set to 1 indicates that the corresponding number is a candidate for that cell.
/// This struct provides methods to get and set candidate masks for specific cells, as well as to
/// update the candidates based on the current state of the board and masks.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Candidates {
    cache: [[u16; 9]; 9],
}

impl Candidates {
    pub fn new() -> Self {
        Candidates { cache: [[0; 9]; 9] }
    }

    /// Returns the candidate mask for a specific cell in the cache.
    pub fn get(&self, r: usize, c: usize) -> u16 {
        self.cache[r][c]
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
