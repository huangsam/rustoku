use crate::error::RustokuError;

/// Represents a Rustoku board, which is a 9x9 grid of cells.
///
/// Each cell can contain a number from 1 to 9, or be empty (represented by 0).
/// There are a few methods to manipulate and query the board state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// Move TryFrom<[u8; 81]> here
impl TryFrom<[u8; 81]> for RustokuBoard {
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
        Ok(RustokuBoard::new(board)) // Construct the RustokuBoard
    }
}

// Move TryFrom<&str> here
impl TryFrom<&str> for RustokuBoard {
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
        // Now use the TryFrom<[u8; 81]> for RustokuBoard to construct it
        bytes.try_into()
    }
}
