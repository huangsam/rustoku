//! Formatting module for Rustoku data structures.
//!
//! This module provides functions to format the Sudoku board and its solve path
//! in various ways.

use crate::core::{Board, Solution, SolvePath, TechniqueFlags};
use std::fmt;

/// Formats the solution into a human-readable string representation.
impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.board)?;
        writeln!(f, "{}", self.solve_path)?;
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

/// Formats the technique mask into a human-readable string representation.
impl fmt::Display for TechniqueFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "None");
        }
        if self.is_all() {
            return write!(f, "All Techniques");
        }

        let mut techniques = Vec::new();

        if self.contains(TechniqueFlags::NAKED_SINGLES) {
            techniques.push("Naked Singles");
        }
        if self.contains(TechniqueFlags::HIDDEN_SINGLES) {
            techniques.push("Hidden Singles");
        }
        if self.contains(TechniqueFlags::NAKED_PAIRS) {
            techniques.push("Naked Pairs");
        }
        if self.contains(TechniqueFlags::HIDDEN_PAIRS) {
            techniques.push("Hidden Pairs");
        }
        if self.contains(TechniqueFlags::LOCKED_CANDIDATES) {
            techniques.push("Locked Candidates");
        }
        if self.contains(TechniqueFlags::XWING) {
            techniques.push("X-Wing");
        }

        write!(f, "{}", techniques.join(", "))
    }
}

impl fmt::Display for SolvePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let path: Vec<(usize, usize, u8, TechniqueFlags)> = self
            .steps
            .iter()
            .map(|step| (step.row, step.col, step.value, step.flags))
            .collect();

        let formatted_lines = format_solve_path(&path, 10);
        write!(f, "{}", formatted_lines.join("\n"))
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
pub fn format_solve_path(
    path: &[(usize, usize, u8, TechniqueFlags)],
    chunk_size: usize,
) -> Vec<String> {
    if path.is_empty() {
        return vec!["(No moves recorded)".to_string()];
    }

    let mut result = Vec::new();
    let mut current_technique = None;
    let mut current_moves = Vec::new();

    for (r, c, val, flags) in path {
        let technique_name = format!("{}", flags);

        if current_technique.as_ref() != Some(&technique_name) {
            // Flush previous technique's moves
            if let Some(tech) = current_technique {
                result.push(format!("{}:", tech));
                // Break moves into chunks of 5 per line
                for chunk in current_moves.chunks(chunk_size) {
                    result.push(format!("  {}", chunk.join(" ")));
                }
                current_moves.clear();
            }
            current_technique = Some(technique_name);
        }

        current_moves.push(format!("R{}C{}={}", r + 1, c + 1, val));
    }

    // Flush final technique
    if let Some(tech) = current_technique {
        result.push(format!("{}:", tech));
        for chunk in current_moves.chunks(chunk_size) {
            result.push(format!("  {}", chunk.join(" ")));
        }
    }

    result
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
    fn test_display_empty_mask() {
        let mask = TechniqueFlags::empty();
        assert_eq!(format!("{}", mask), "None");
    }

    #[test]
    fn test_display_single_technique() {
        let mask = TechniqueFlags::NAKED_SINGLES;
        assert_eq!(format!("{}", mask), "Naked Singles");

        let mask = TechniqueFlags::XWING;
        assert_eq!(format!("{}", mask), "X-Wing");
    }

    #[test]
    fn test_display_multiple_techniques() {
        let mask = TechniqueFlags::EASY;
        assert_eq!(format!("{}", mask), "Naked Singles, Hidden Singles");

        let mask = TechniqueFlags::NAKED_SINGLES
            | TechniqueFlags::XWING
            | TechniqueFlags::LOCKED_CANDIDATES;
        assert_eq!(
            format!("{}", mask),
            "Naked Singles, Locked Candidates, X-Wing"
        );
    }
}
