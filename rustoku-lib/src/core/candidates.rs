use super::board::Board;
use super::masks::Masks;

/// Cache of available candidate digits (1–9) for each cell on the board.
///
/// Maintains a 9x9 matrix of 9-bit bitmasks (`0x01FF`), where bit index `(num - 1)`
/// indicates that digit `num` remains valid. Filled cells always store `0`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Candidates {
    cache: [[u16; 9]; 9],
}

/// Compact stack-allocated candidate list (up to 9 digits).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CandidateList {
    /// Number of valid candidate digits stored in `digits`.
    pub len: u8,
    /// Candidate digits 1..=9.
    pub digits: [u8; 9],
}

impl CandidateList {
    /// Extracts candidate digits from a 9-bit bitmask in $O(k)$ where $k$ is candidate count.
    #[inline]
    pub fn from_mask(mask: u16) -> Self {
        let mut digits = [0u8; 9];
        let mut len = 0;
        let mut m = mask & 0x01FF;
        while m != 0 {
            let trailing = m.trailing_zeros() as u8;
            digits[len as usize] = trailing + 1;
            len += 1;
            m &= m - 1;
        }
        Self { len, digits }
    }

    /// Returns a slice of candidate digits.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.digits[..self.len as usize]
    }

    /// Returns a mutable slice of candidate digits (e.g. for shuffling).
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.digits[..self.len as usize]
    }
}

impl Candidates {
    pub(super) fn new() -> Self {
        Candidates { cache: [[0; 9]; 9] }
    }

    /// Returns the raw 9-bit candidate bitmask for cell `(r, c)`.
    #[inline]
    pub(super) fn get(&self, r: usize, c: usize) -> u16 {
        self.cache[r][c]
    }

    /// Unpacks the candidate bitmask for `(r, c)` into a sorted vector of digits `1..=9`.
    pub fn get_candidates(&self, r: usize, c: usize) -> Vec<u8> {
        let mask = self.get(r, c);
        CandidateList::from_mask(mask).as_slice().to_vec()
    }

    /// Sets the 9-bit candidate bitmask for cell `(r, c)`.
    #[inline]
    pub(super) fn set(&mut self, r: usize, c: usize, mask: u16) {
        self.cache[r][c] = mask;
    }

    /// Recomputes candidates for all peers sharing a row, column, or 3x3 box with `(r, c)`.
    ///
    /// Used during candidate removal or backtracking undo where all peer constraints must be
    /// unconditionally refreshed (`placed_num: None`).
    pub(super) fn update_affected_cells(
        &mut self,
        r: usize,
        c: usize,
        masks: &Masks,
        board: &Board,
    ) {
        self.update_affected_cells_for(r, c, masks, board, None);
    }

    /// Recomputes candidates for affected peers, optionally filtering by a placed digit.
    ///
    /// - **Placement (`placed_num: Some(n)`)**: Fast path that only recomputes peer cells
    ///   currently holding `n` as a candidate, skipping cells unaffected by the new constraint.
    /// - **Removal / Undo (`placed_num: None`)**: Unconditional path that recomputes all peer cells,
    ///   restoring candidate possibilities that were previously suppressed by the placed digit.
    pub(super) fn update_affected_cells_for(
        &mut self,
        r: usize,
        c: usize,
        masks: &Masks,
        board: &Board,
        placed_num: Option<u8>,
    ) {
        // 1. Invalidate/update cache for the target cell (r, c)
        if board.is_empty(r, c) {
            // Removal case: recompute candidates for the now-empty cell
            self.cache[r][c] = masks.compute_candidates_mask_for_cell(r, c);
        } else {
            // Placement case: filled cells have 0 available candidates
            self.cache[r][c] = 0;
        }

        // 2. Precompute the placed digit's bitmask for fast-path peer filtering
        let num_bit = placed_num.map(|n| 1u16 << (n - 1));

        // 3. Update peer cells in the same row and column
        for i in 0..9 {
            if board.is_empty(r, i) && i != c {
                // Fast path: skip recomputing if cell did not contain the placed digit
                if let Some(bit) = num_bit
                    && self.cache[r][i] & bit == 0
                {
                    continue;
                }
                self.cache[r][i] = masks.compute_candidates_mask_for_cell(r, i);
            }
            if board.is_empty(i, c) && i != r {
                // Fast path: skip recomputing if cell did not contain the placed digit
                if let Some(bit) = num_bit
                    && self.cache[i][c] & bit == 0
                {
                    continue;
                }
                self.cache[i][c] = masks.compute_candidates_mask_for_cell(i, c);
            }
        }

        // 4. Update remaining peer cells in the 3x3 box
        // Skips (cur_r == r) and (cur_c == c) because the 4 peer cells sharing both box and row/column
        // were already updated in the row/column loop above, avoiding redundant recalculations.
        let box_idx = Masks::get_box_idx(r, c);
        let start_row = (box_idx / 3) * 3;
        let start_col = (box_idx % 3) * 3;
        for r_offset in 0..3 {
            for c_offset in 0..3 {
                let cur_r = start_row + r_offset;
                let cur_c = start_col + c_offset;
                // Skip the target cell and peers already updated in the row/column loop
                if (cur_r == r) || (cur_c == c) {
                    continue;
                }
                if board.is_empty(cur_r, cur_c) {
                    // Fast path: skip recomputing if cell did not contain the placed digit
                    if let Some(bit) = num_bit
                        && self.cache[cur_r][cur_c] & bit == 0
                    {
                        continue;
                    }
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
        let cands: Vec<u8> = candidates.get_candidates(r, c);
        assert_eq!(cands, Vec::<u8>::new());
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
        assert_eq!(candidates.get_candidates(0, 1), Vec::<u8>::new()); // Unset cell
    }
}
