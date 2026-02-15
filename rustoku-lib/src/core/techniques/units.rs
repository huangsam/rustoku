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
