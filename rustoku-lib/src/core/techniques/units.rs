/// Returns an array of all cell coordinates in the specified row.
pub fn row_cells(r: usize) -> [(usize, usize); 9] {
    core::array::from_fn(|c| (r, c))
}

/// Returns an array of all cell coordinates in the specified column.
pub fn col_cells(c: usize) -> [(usize, usize); 9] {
    core::array::from_fn(|r| (r, c))
}

/// Returns an array of all cell coordinates in the specified 3x3 box.
/// Box indices are numbered 0-8, left-to-right, top-to-bottom.
pub fn box_cells(box_idx: usize) -> [(usize, usize); 9] {
    let start_row = (box_idx / 3) * 3;
    let start_col = (box_idx % 3) * 3;
    core::array::from_fn(|i| (start_row + i / 3, start_col + i % 3))
}

/// Finds units (rows/columns/boxes) that have exactly N cells containing a specific candidate.
/// Returns a vector of (unit_index, positions) tuples.
pub fn find_units_with_n_candidates(
    candidate_bit: u16,
    n: usize,
    candidates: &super::super::Candidates,
    board: &super::super::Board,
    unit_type: UnitType,
) -> Vec<(usize, Vec<usize>)> {
    let mut result = Vec::new();

    for unit_idx in 0..9 {
        let unit_cells = match unit_type {
            UnitType::Row => row_cells(unit_idx),
            UnitType::Column => col_cells(unit_idx),
        };

        let positions: Vec<usize> = unit_cells
            .iter()
            .enumerate()
            .filter(|&(_, &(r, c))| {
                board.is_empty(r, c) && (candidates.get(r, c) & candidate_bit) != 0
            })
            .map(|(pos, _)| pos)
            .collect();

        if positions.len() == n {
            result.push((unit_idx, positions));
        }
    }

    result
}

/// Represents the type of unit (row or column).
#[derive(Clone, Copy)]
pub enum UnitType {
    Row,
    Column,
}

#[cfg(test)]
mod tests {
    use super::super::super::board::Board;
    use super::super::super::candidates::Candidates;
    use super::*;

    #[test]
    fn test_row_cells() {
        let cells = row_cells(5);
        for c in 0..9 {
            assert_eq!(cells[c], (5, c));
        }
    }

    #[test]
    fn test_col_cells() {
        let cells = col_cells(3);
        for r in 0..9 {
            assert_eq!(cells[r], (r, 3));
        }
    }

    #[test]
    fn test_box_cells() {
        // Box 0
        let box0 = box_cells(0);
        assert_eq!(box0[0], (0, 0));
        assert_eq!(box0[4], (1, 1));
        assert_eq!(box0[8], (2, 2));

        // Box 4 (middle)
        let box4 = box_cells(4);
        assert_eq!(box4[0], (3, 3));
        assert_eq!(box4[4], (4, 4));
        assert_eq!(box4[8], (5, 5));

        // Box 8 (bottom-right)
        let box8 = box_cells(8);
        assert_eq!(box8[0], (6, 6));
        assert_eq!(box8[4], (7, 7));
        assert_eq!(box8[8], (8, 8));
    }

    #[test]
    fn test_find_units_with_n_candidates() {
        let mut board = Board::default();
        let mut candidates = Candidates::new();

        // Let's set candidate 1 at (0,0) and (0,1)
        // That's row 0, positions 0 and 1
        let candidate_bit = 1 << 0;
        candidates.set(0, 0, candidate_bit);
        candidates.set(0, 1, candidate_bit);

        // find row with 2 candidates
        let rows =
            find_units_with_n_candidates(candidate_bit, 2, &candidates, &board, UnitType::Row);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, 0); // row 0
        assert_eq!(rows[0].1, vec![0, 1]); // positions 0 and 1

        // find columns with 1 candidate
        // Col 0 and Col 1 should each have 1
        let cols =
            find_units_with_n_candidates(candidate_bit, 1, &candidates, &board, UnitType::Column);
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].0, 0); // col 0
        assert_eq!(cols[1].0, 1); // col 1

        // find rows with 3 candidates -> should be none
        let empty_rows =
            find_units_with_n_candidates(candidate_bit, 3, &candidates, &board, UnitType::Row);
        assert!(empty_rows.is_empty());

        // Now place a number in (0,0) so it's not empty anymore
        board.set(0, 0, 1);
        // Now row 0 only has 1 candidate (at (0,1))
        let rows_after_placement =
            find_units_with_n_candidates(candidate_bit, 1, &candidates, &board, UnitType::Row);
        assert_eq!(rows_after_placement.len(), 1);
        assert_eq!(rows_after_placement[0].0, 0);
        assert_eq!(rows_after_placement[0].1, vec![1]);
    }
}
