#[derive(Debug, Copy, Clone)]
pub struct RustokuBoard {
    pub cells: [[u8; 9]; 9],
}

impl RustokuBoard {
    pub fn new(initial_board: [[u8; 9]; 9]) -> Self {
        RustokuBoard {
            cells: initial_board,
        }
    }

    pub fn get(&self, r: usize, c: usize) -> u8 {
        self.cells[r][c]
    }

    pub fn set(&mut self, r: usize, c: usize, value: u8) {
        self.cells[r][c] = value;
    }

    pub fn is_empty(&self, r: usize, c: usize) -> bool {
        self.cells[r][c] == 0
    }

    pub fn iter_empty_cells(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..9).flat_map(move |r| {
            (0..9).filter_map(move |c| {
                if self.is_empty(r, c) {
                    Some((r, c))
                } else {
                    None
                }
            })
        })
    }

    // You might also add methods to get an iterator over a row, column, or box's cells.
}
