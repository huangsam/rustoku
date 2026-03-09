use crate::error::RustokuError;
use serde::{Serialize, Deserialize};

/// Raw 9x9 board with some useful helpers.
///
/// There are multiple ways to create a `Board`:
/// - Using a 2D array of `u8` with dimensions 9x9
/// - Using a 1D array of `u8` with length 81
/// - Using a string representation with length 81
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Board {
    /// Each cell can contain a number from 1 to 9, or be empty (is 0).
    pub(crate) cells: [[u8; 9]; 9],
}

impl Board {
    pub fn new(initial_board: [[u8; 9]; 9]) -> Self {
        Board {
            cells: initial_board,
        }
    }

    /// Gets a value from the board at the specified row and column.
    #[inline]
    pub fn get(&self, r: usize, c: usize) -> u8 {
        self.cells[r][c]
    }

    /// Sets a value in the board at the specified row and column.
    #[inline]
    pub(super) fn set(&mut self, r: usize, c: usize, value: u8) {
        self.cells[r][c] = value;
    }

    /// Checks if a cell at the specified row and column is empty (contains 0).
    #[inline]
    pub fn is_empty(&self, r: usize, c: usize) -> bool {
        self.cells[r][c] == 0
    }

    /// Iterates over all cells in the board, yielding their row and column indices.
    #[inline]
    pub fn iter_cells(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..9).flat_map(move |r| (0..9).map(move |c| (r, c)))
    }

    /// Iterates over empty cells in the board, yielding their row and column indices.
    #[inline]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RustokuError;

    const UNIQUE_PUZZLE: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    #[test]
    fn test_new_with_bytes_and_str() {
        let board = [
            [5, 3, 0, 0, 7, 0, 0, 0, 0],
            [6, 0, 0, 1, 9, 5, 0, 0, 0],
            [0, 9, 8, 0, 0, 0, 0, 6, 0],
            [8, 0, 0, 0, 6, 0, 0, 0, 3],
            [4, 0, 0, 8, 0, 3, 0, 0, 1],
            [7, 0, 0, 0, 2, 0, 0, 0, 6],
            [0, 6, 0, 0, 0, 0, 2, 8, 0],
            [0, 0, 0, 4, 1, 9, 0, 0, 5],
            [0, 0, 0, 0, 8, 0, 0, 7, 9],
        ];

        let flat_bytes: [u8; 81] = board
            .concat()
            .try_into()
            .expect("Concat board to bytes failed");
        let board_str: String = flat_bytes.iter().map(|&b| (b + b'0') as char).collect();

        let board_from_new = Board::new(board);
        let board_from_bytes = Board::try_from(flat_bytes).expect("Board from flat bytes failed");
        let board_from_str = Board::try_from(board_str.as_str()).expect("Board from string failed");

        assert_eq!(board_from_new, board_from_bytes);
        assert_eq!(board_from_new, board_from_str);
        assert_eq!(board_from_bytes, board_from_str);
    }

    #[test]
    fn test_try_from_with_valid_input() {
        let rustoku = Board::try_from(UNIQUE_PUZZLE);
        assert!(rustoku.is_ok());
    }

    #[test]
    fn test_try_from_with_invalid_length() {
        let s = "530070000"; // Too short
        let rustoku = Board::try_from(s);
        assert!(matches!(rustoku, Err(RustokuError::InvalidInputLength)));
    }

    #[test]
    fn test_try_from_with_invalid_character() {
        let s = "53007000060019500009800006080006000340080300170002000606000028000041900500008007X"; // 'X'
        let rustoku = Board::try_from(s);
        assert!(matches!(rustoku, Err(RustokuError::InvalidInputCharacter)));
    }
}

#[cfg(test)]
mod serde_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serde_board_roundtrip() {
        let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let board = Board::try_from(puzzle).unwrap();
        let json = serde_json::to_string(&board).expect("Failed to serialize board");
        let deserialized: Board = serde_json::from_str(&json).expect("Failed to deserialize board");
        assert_eq!(board, deserialized);
    }
}
