//! Formatting module for Rustoku data structures.
//!
//! This module provides functions to format the Sudoku board and its solve path
//! in various ways.

use crate::core::{Board, Solution, SolvePath, SolveStep, TechniqueFlags};
use std::fmt;

/// Formats the solution into a human-readable string representation.
impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.board)?;
        write!(f, "\n{}", self.solve_path)?;
        Ok(())
    }
}

/// Formats the board into a human-readable string representation.
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", format_grid(&self.cells).join("\n"))?;
        write!(f, "Line format: {}", format_line(&self.cells))?;
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

/// Formats the solve path into a human-readable string representation.
impl fmt::Display for SolvePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let path: Vec<(usize, usize, u8, TechniqueFlags, &str)> = self
            .steps
            .iter()
            .map(|step| match step {
                crate::core::SolveStep::Placement {
                    row,
                    col,
                    value,
                    flags,
                } => (*row, *col, *value, *flags, step.code()),
                crate::core::SolveStep::CandidateElimination {
                    row,
                    col,
                    value,
                    flags,
                } => (*row, *col, *value, *flags, step.code()),
            })
            .collect();

        let formatted_lines = format_solve_path(&path, 5);
        write!(f, "{}", formatted_lines.join("\n"))
    }
}

/// Formats the solve step into a human-readable string representation.
impl fmt::Display for SolveStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolveStep::Placement {
                row,
                col,
                value,
                flags,
            } => {
                write!(
                    f,
                    "Value {} is placed on R{}C{} by {}",
                    value, row, col, flags
                )
            }
            SolveStep::CandidateElimination {
                row,
                col,
                value,
                flags,
            } => {
                write!(
                    f,
                    "Value {} is eliminated from R{}C{} by {}",
                    value, row, col, flags
                )
            }
        }
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
        .map(|&n| (n + b'0') as char)
        .collect()
}

/// Formats a path of moves in the Sudoku solving process into a vector of strings.
///
/// This function takes a vector of tuples representing moves in the format `(row, column, value)`
/// and formats them into a human-readable string. Each move is represented as `(row, column, value)`,
/// where `row` and `column` are 1-based indices, and `value` is the number placed in that cell.
pub fn format_solve_path(
    path: &[(usize, usize, u8, TechniqueFlags, &str)],
    chunk_size: usize,
) -> Vec<String> {
    if path.is_empty() {
        return vec!["(No moves recorded)".to_string()];
    }

    let mut result = Vec::new();
    let mut current_technique = None;
    let mut current_moves = Vec::new();

    for (r, c, val, flags, action) in path {
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

        current_moves.push(format!("R{}C{}={},A={}", r + 1, c + 1, val, action));
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
    use crate::core::TechniqueFlags;

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
            "530678912672195348198342567859761423426853791713924856961537284287419635345286179";
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
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000";
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

    #[test]
    fn test_empty_path() {
        let path: Vec<(usize, usize, u8, TechniqueFlags, &str)> = Vec::new();
        let expected = vec!["(No moves recorded)".to_string()];
        assert_eq!(format_solve_path(&path, 5), expected);
    }

    #[test]
    fn test_single_technique_multiple_moves_with_chunking() {
        let path = vec![
            (0, 0, 1, TechniqueFlags::NAKED_SINGLES, "plac"),
            (0, 1, 2, TechniqueFlags::NAKED_SINGLES, "plac"),
            (0, 2, 3, TechniqueFlags::NAKED_SINGLES, "plac"),
            (0, 3, 4, TechniqueFlags::NAKED_SINGLES, "plac"),
            (0, 4, 5, TechniqueFlags::NAKED_SINGLES, "plac"),
            (0, 5, 6, TechniqueFlags::NAKED_SINGLES, "plac"),
        ];
        let chunk_size = 2; // Each line will have 2 moves

        let expected = vec![
            "Naked Singles:".to_string(),
            "  R1C1=1,A=plac R1C2=2,A=plac".to_string(),
            "  R1C3=3,A=plac R1C4=4,A=plac".to_string(),
            "  R1C5=5,A=plac R1C6=6,A=plac".to_string(),
        ];
        assert_eq!(format_solve_path(&path, chunk_size), expected);
    }

    #[test]
    fn test_multiple_techniques_and_mixed_chunking() {
        let path = vec![
            (0, 0, 1, TechniqueFlags::NAKED_SINGLES, "plac"),
            (0, 1, 2, TechniqueFlags::NAKED_SINGLES, "plac"),
            (1, 0, 3, TechniqueFlags::HIDDEN_SINGLES, "plac"),
            (1, 1, 4, TechniqueFlags::HIDDEN_SINGLES, "plac"),
            (1, 2, 5, TechniqueFlags::HIDDEN_SINGLES, "plac"),
            (2, 0, 6, TechniqueFlags::HIDDEN_PAIRS, "elim"),
        ];
        let chunk_size = 3; // Each line will have 3 moves

        let expected: Vec<String> = vec![
            "Naked Singles:".to_string(),
            "  R1C1=1,A=plac R1C2=2,A=plac".to_string(),
            "Hidden Singles:".to_string(),
            "  R2C1=3,A=plac R2C2=4,A=plac R2C3=5,A=plac".to_string(),
            "Hidden Pairs:".to_string(),
            "  R3C1=6,A=elim".to_string(),
        ];
        assert_eq!(format_solve_path(&path, chunk_size), expected);
    }
}
