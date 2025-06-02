use crate::error::RustokuError;

/// Raw board, which is a 9x9 grid of cells.
///
/// Each cell can contain a number from 1 to 9, or be empty (is 0).
///
/// There are multiple ways to create a `Board`:
/// - Using a 2D array of `u8` with dimensions 9x9
/// - Using a 1D array of `u8` with length 81
/// - Using a string representation with length 81
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Board {
    pub cells: [[u8; 9]; 9],
}

impl Board {
    pub fn new(initial_board: [[u8; 9]; 9]) -> Self {
        Board {
            cells: initial_board,
        }
    }

    /// Creates an empty Rustoku board, where all cells are initialized to 0.
    pub fn empty() -> Self {
        Board {
            cells: [[0u8; 9]; 9],
        }
    }

    /// Gets a value from the board at the specified row and column.
    pub fn get(&self, r: usize, c: usize) -> u8 {
        self.cells[r][c]
    }

    /// Sets a value in the board at the specified row and column.
    pub fn set(&mut self, r: usize, c: usize, value: u8) {
        self.cells[r][c] = value;
    }

    /// Checks if a cell at the specified row and column is empty (contains 0).
    pub fn is_empty(&self, r: usize, c: usize) -> bool {
        self.cells[r][c] == 0
    }

    /// Iterates over all cells in the board, yielding their row and column indices.
    pub fn iter_cells(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..9).flat_map(move |r| (0..9).map(move |c| (r, c)))
    }

    /// Iterates over empty cells in the board, yielding their row and column indices.
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
}

impl TryFrom<[u8; 81]> for Board {
    type Error = RustokuError;

    fn try_from(bytes: [u8; 81]) -> Result<Self, Self::Error> {
        let mut board = [[0u8; 9]; 9];
        for i in 0..81 {
            // Validate that numbers are within 0-9 if you want strictness here,
            // though Rustoku::new will validate initial state safety.
            if bytes[i] > 9 {
                return Err(RustokuError::InvalidInputCharacter); // Or a more specific error
            }
            board[i / 9][i % 9] = bytes[i];
        }
        Ok(Board::new(board)) // Construct the board
    }
}

impl TryFrom<&str> for Board {
    type Error = RustokuError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.len() != 81 {
            return Err(RustokuError::InvalidInputLength);
        }
        let mut bytes = [0u8; 81];
        for (i, ch) in s.bytes().enumerate() {
            match ch {
                b'0'..=b'9' => bytes[i] = ch - b'0',
                b'.' | b'_' => bytes[i] = 0, // Treat '.' and '_' as empty cells
                _ => return Err(RustokuError::InvalidInputCharacter),
            }
        }
        // Now use the TryFrom<[u8; 81]> for board
        bytes.try_into()
    }
}
