//! Formatting module for Rustoku data structures.
//!
//! This module provides functions to format the Sudoku board and its solve path
//! in various ways. It also includes a simple utility to print the board to
//! the console.

use crate::core::{Board, Solution};
use std::fmt;

/// Formats the solution into a human-readable string representation.
impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.board)?;
        writeln!(
            f,
            "Solve path:\n{}",
            format_solve_path(&self.solve_path).join("\n")
        )?;
        Ok(())
    }
}

/// Formats the board into a human-readable string representation.
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", format_grid(&self.cells).join("\n"))?;
        writeln!(f, "Line format: {}", format_line(&self.cells))?;
        Ok(())
    }
}

/// Formats the Sudoku board into a grid representation.
///
/// This function takes a 9x9 Sudoku board and formats it into a grid with
/// horizontal and vertical separators to visually distinguish the 3x3 boxes.
/// Each cell is represented by its number, with empty cells shown as a dot (`.`).
pub fn format_grid(board: &[[u8; 9]; 9]) -> Vec<String> {
    let mut grid = Vec::new();
    let horizontal_line = "+-------+-------+-------+";

    grid.push(horizontal_line.to_string()); // Top line

    for (r, row) in board.iter().enumerate().take(9) {
        let mut line = String::from("|"); // Start of the row
        for (c, &cell) in row.iter().enumerate().take(9) {
            match cell {
                0 => line.push_str(" ."), // Empty cell, two spaces for alignment
                n => line.push_str(&format!(" {}", n)), // Number, two spaces for alignment
            }
            if (c + 1) % 3 == 0 {
                line.push_str(" |"); // Vertical separator after every 3rd column
            }
        }
        grid.push(line); // Add the row to the grid

        if (r + 1) % 3 == 0 {
            grid.push(horizontal_line.to_string()); // Horizontal separator after every 3rd row
        }
    }

    grid
}

/// Formats the Sudoku board into a single line string representation.
///
/// This function converts the board into a single string where each number is
/// represented by its digit, and empty cells are represented by a dot (`.`).
pub fn format_line(board: &[[u8; 9]; 9]) -> String {
    board
        .iter()
        .flatten()
        .map(|&n| match n {
            0 => '.',
            n => (n + b'0') as char,
        })
        .collect()
}

/// Formats a path of moves in the Sudoku solving process into a vector of strings.
///
/// This function takes a vector of tuples representing moves in the format `(row, column, value)`
/// and formats them into a human-readable string. Each move is represented as `(row, column, value)`,
/// where `row` and `column` are 1-based indices, and `value` is the number placed in that cell.
pub fn format_solve_path(path: &[(usize, usize, u8)]) -> Vec<String> {
    if path.is_empty() {
        vec!["(No moves recorded)".to_string()]
    } else {
        path.iter()
            .map(|(r, c, val)| format!("({}, {}, {})", r + 1, c + 1, val))
            .collect::<Vec<String>>()
            .chunks(5) // Break into chunks of 5 moves
            .map(|chunk| chunk.join(" -> "))
            .collect::<Vec<String>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_grid() {
        let board = [
            [5, 3, 0, 6, 7, 8, 9, 1, 2],
            [6, 7, 2, 1, 9, 5, 3, 4, 8],
            [1, 9, 8, 3, 4, 2, 5, 6, 7],
            [8, 5, 9, 7, 6, 1, 4, 2, 3],
            [4, 2, 6, 8, 5, 3, 7, 9, 1],
            [7, 1, 3, 9, 2, 4, 8, 5, 6],
            [9, 6, 1, 5, 3, 7, 2, 8, 4],
            [2, 8, 7, 4, 1, 9, 6, 3, 5],
            [3, 4, 5, 2, 8, 6, 1, 7, 9],
        ];

        let expected = vec![
            "+-------+-------+-------+",
            "| 5 3 . | 6 7 8 | 9 1 2 |",
            "| 6 7 2 | 1 9 5 | 3 4 8 |",
            "| 1 9 8 | 3 4 2 | 5 6 7 |",
            "+-------+-------+-------+",
            "| 8 5 9 | 7 6 1 | 4 2 3 |",
            "| 4 2 6 | 8 5 3 | 7 9 1 |",
            "| 7 1 3 | 9 2 4 | 8 5 6 |",
            "+-------+-------+-------+",
            "| 9 6 1 | 5 3 7 | 2 8 4 |",
            "| 2 8 7 | 4 1 9 | 6 3 5 |",
            "| 3 4 5 | 2 8 6 | 1 7 9 |",
            "+-------+-------+-------+",
        ];

        assert_eq!(expected, format_grid(&board));
    }

    #[test]
    fn test_format_line() {
        let board = [
            [5, 3, 0, 6, 7, 8, 9, 1, 2],
            [6, 7, 2, 1, 9, 5, 3, 4, 8],
            [1, 9, 8, 3, 4, 2, 5, 6, 7],
            [8, 5, 9, 7, 6, 1, 4, 2, 3],
            [4, 2, 6, 8, 5, 3, 7, 9, 1],
            [7, 1, 3, 9, 2, 4, 8, 5, 6],
            [9, 6, 1, 5, 3, 7, 2, 8, 4],
            [2, 8, 7, 4, 1, 9, 6, 3, 5],
            [3, 4, 5, 2, 8, 6, 1, 7, 9],
        ];

        let expected =
            "53.678912672195348198342567859761423426853791713924856961537284287419635345286179";
        assert_eq!(expected, format_line(&board));
    }

    #[test]
    fn test_format_grid_empty_board() {
        let board = [[0; 9]; 9];

        let expected = vec![
            "+-------+-------+-------+",
            "| . . . | . . . | . . . |",
            "| . . . | . . . | . . . |",
            "| . . . | . . . | . . . |",
            "+-------+-------+-------+",
            "| . . . | . . . | . . . |",
            "| . . . | . . . | . . . |",
            "| . . . | . . . | . . . |",
            "+-------+-------+-------+",
            "| . . . | . . . | . . . |",
            "| . . . | . . . | . . . |",
            "| . . . | . . . | . . . |",
            "+-------+-------+-------+",
        ];

        assert_eq!(expected, format_grid(&board));
    }

    #[test]
    fn test_format_line_empty_board() {
        let board = [[0; 9]; 9];
        let expected =
            ".................................................................................";
        assert_eq!(expected, format_line(&board));
    }

    #[test]
    fn test_format_solve_path_one_line() {
        let path = vec![(0, 0, 5), (1, 1, 3), (2, 2, 4), (3, 3, 6), (4, 4, 7)];
        let expected = vec!["(1, 1, 5) -> (2, 2, 3) -> (3, 3, 4) -> (4, 4, 6) -> (5, 5, 7)"];
        assert_eq!(expected, format_solve_path(&path));
    }

    #[test]
    fn test_format_solve_path_no_moves() {
        let path: Vec<(usize, usize, u8)> = vec![];
        let expected = vec!["(No moves recorded)".to_string()];
        assert_eq!(expected, format_solve_path(&path));
    }

    #[test]
    fn test_format_solve_path_multiple_lines() {
        let path = vec![
            (0, 0, 5),
            (1, 1, 3),
            (2, 2, 4),
            (3, 3, 6),
            (4, 4, 7),
            (5, 5, 8),
        ];
        let expected = vec![
            "(1, 1, 5) -> (2, 2, 3) -> (3, 3, 4) -> (4, 4, 6) -> (5, 5, 7)",
            "(6, 6, 8)",
        ];
        assert_eq!(expected, format_solve_path(&path));
    }
}
