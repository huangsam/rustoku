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
