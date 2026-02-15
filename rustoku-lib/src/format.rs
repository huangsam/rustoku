//! Formatting module for Rustoku data structures.
//!
//! This module provides functions to format the Sudoku board and its solve path
//! in a way that is suitable for terminals.

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
        writeln!(f, "{}", format_grid(self).join("\n"))?;
        write!(f, "Line format: {}", format_line(self))?;
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
        let formatted_lines = format_solve_path(self, 5);
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
                step_number,
                candidates_eliminated,
                related_cell_count,
                difficulty_point,
            } => {
                write!(
                    f,
                    "#{:3} | Value {value} is placed on R{row}C{col} by {flags} | elim:{} related:{} diff:{}",
                    step_number + 1,
                    bin(*candidates_eliminated).count_ones(),
                    related_cell_count,
                    difficulty_point
                )
            }
            SolveStep::CandidateElimination {
                row,
                col,
                value,
                flags,
                step_number,
                candidates_eliminated,
                related_cell_count,
                difficulty_point,
            } => {
                write!(
                    f,
                    "#{:3} | Value {value} is eliminated from R{row}C{col} by {flags} | elim:{} related:{} diff:{}",
                    step_number + 1,
                    bin(*candidates_eliminated).count_ones() + 1, // +1 for the main elimination
                    related_cell_count,
                    difficulty_point
                )
            }
        }
    }
}

/// Formats a u32 bitmask showing binary representation (helper for diagnostics).
fn bin(x: u32) -> u32 {
    x
}

/// Formats the Sudoku board into a grid representation.
///
/// This function takes a 9x9 Sudoku board and formats it into a grid with
/// horizontal and vertical separators to visually distinguish the 3x3 boxes.
/// Each cell is represented by its number, with empty cells shown as a dot (`.`).
pub(crate) fn format_grid(board: &Board) -> Vec<String> {
    let mut grid = Vec::new();
    let horizontal_line = "+-------+-------+-------+";

    grid.push(horizontal_line.to_string()); // Top line

    for (r, row) in board.cells.iter().enumerate().take(9) {
        let mut line = String::from("|"); // Start of the row
        for (c, &cell) in row.iter().enumerate().take(9) {
            match cell {
                0 => line.push_str(" ."), // Empty cell, two spaces for alignment
                n => line.push_str(&format!(" {n}")), // Number, two spaces for alignment
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
pub(crate) fn format_line(board: &Board) -> String {
    board
        .cells
        .iter()
        .flatten()
        .map(|&n| (n + b'0') as char)
        .collect()
}

/// Formats a path of moves in the Sudoku solving process into a vector of strings.
///
/// This function takes a `SolvePath` struct and formats its moves into a compact multi-step format.
/// Each line shows exactly 3 steps with diagnostic metadata for efficient overview.
pub(crate) fn format_solve_path(solve_path: &SolvePath, _chunk_size: usize) -> Vec<String> {
    if solve_path.steps.is_empty() {
        return vec!["(No moves recorded)".to_string()];
    }

    let mut result = Vec::new();
    let mut current_technique = None;
    let mut current_moves = Vec::new();

    for step in &solve_path.steps {
        let flags = match step {
            SolveStep::Placement { flags, .. } | SolveStep::CandidateElimination { flags, .. } => {
                *flags
            }
        };

        let technique_name = format!("{flags}");

        if current_technique.as_ref() != Some(&technique_name) {
            // Flush previous technique's moves
            if let Some(tech) = current_technique {
                result.push(format!("{tech}:"));
                // Use 1 step per line for maximum clarity and learning
                for chunk in current_moves.chunks(1) {
                    // Format with padding: each step gets 5 chars width for neat alignment
                    let formatted_chunk: Vec<String> =
                        chunk.iter().map(|s| format!("{:<5}", s)).collect();
                    result.push(format!("  {}", formatted_chunk.join("")));
                }
                current_moves.clear();
            }
            current_technique = Some(technique_name);
        }

        // Format as compact step with readable labels
        let step_str = match step {
            SolveStep::Placement {
                row,
                col,
                value,
                step_number,
                candidates_eliminated,
                related_cell_count,
                difficulty_point,
                ..
            } => {
                format!(
                    "#{} R{}C{}={} [E:{} R:{} D:{}]",
                    step_number + 1,
                    row + 1,
                    col + 1,
                    value,
                    candidates_eliminated,
                    related_cell_count,
                    difficulty_point
                )
            }
            SolveStep::CandidateElimination {
                row,
                col,
                value,
                step_number,
                candidates_eliminated,
                related_cell_count,
                difficulty_point,
                ..
            } => {
                let total_elim = *candidates_eliminated + 1;
                format!(
                    "#{} -{}@R{}C{} [E:{} R:{} D:{}]",
                    step_number + 1,
                    value,
                    row + 1,
                    col + 1,
                    total_elim,
                    related_cell_count,
                    difficulty_point
                )
            }
        };

        current_moves.push(step_str);
    }

    // Flush final technique
    if let Some(tech) = current_technique {
        result.push(format!("{tech}:"));
        for chunk in current_moves.chunks(1) {
            let formatted_chunk: Vec<String> = chunk.iter().map(|s| format!("{:<5}", s)).collect();
            result.push(format!("  {}", formatted_chunk.join("")));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_format_grid() {
        let board = Board::new([
            [5, 3, 0, 6, 7, 8, 9, 1, 2],
            [6, 7, 2, 1, 9, 5, 3, 4, 8],
            [1, 9, 8, 3, 4, 2, 5, 6, 7],
            [8, 5, 9, 7, 6, 1, 4, 2, 3],
            [4, 2, 6, 8, 5, 3, 7, 9, 1],
            [7, 1, 3, 9, 2, 4, 8, 5, 6],
            [9, 6, 1, 5, 3, 7, 2, 8, 4],
            [2, 8, 7, 4, 1, 9, 6, 3, 5],
            [3, 4, 5, 2, 8, 6, 1, 7, 9],
        ]);

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
        let board = Board::new([
            [5, 3, 0, 6, 7, 8, 9, 1, 2],
            [6, 7, 2, 1, 9, 5, 3, 4, 8],
            [1, 9, 8, 3, 4, 2, 5, 6, 7],
            [8, 5, 9, 7, 6, 1, 4, 2, 3],
            [4, 2, 6, 8, 5, 3, 7, 9, 1],
            [7, 1, 3, 9, 2, 4, 8, 5, 6],
            [9, 6, 1, 5, 3, 7, 2, 8, 4],
            [2, 8, 7, 4, 1, 9, 6, 3, 5],
            [3, 4, 5, 2, 8, 6, 1, 7, 9],
        ]);

        let expected =
            "530678912672195348198342567859761423426853791713924856961537284287419635345286179";
        assert_eq!(expected, format_line(&board));
    }

    #[test]
    fn test_format_grid_empty_board() {
        let board = Board::default();

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
        let board = Board::default();
        let expected =
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(expected, format_line(&board));
    }

    #[test]
    fn test_display_empty_mask() {
        let mask = TechniqueFlags::empty();
        assert_eq!(format!("{mask}"), "None");
    }

    #[test]
    fn test_display_single_technique() {
        let mask = TechniqueFlags::NAKED_SINGLES;
        assert_eq!(format!("{mask}"), "Naked Singles");

        let mask = TechniqueFlags::XWING;
        assert_eq!(format!("{mask}"), "X-Wing");
    }

    #[test]
    fn test_display_multiple_techniques() {
        let mask = TechniqueFlags::EASY;
        assert_eq!(format!("{mask}"), "Naked Singles, Hidden Singles");

        let mask = TechniqueFlags::NAKED_SINGLES
            | TechniqueFlags::XWING
            | TechniqueFlags::LOCKED_CANDIDATES;
        assert_eq!(
            format!("{mask}"),
            "Naked Singles, Locked Candidates, X-Wing"
        );
    }

    #[test]
    fn test_empty_path() {
        let solve_path = SolvePath { steps: Vec::new() }; // Create an empty SolvePath
        let expected = vec!["(No moves recorded)"];
        assert_eq!(format_solve_path(&solve_path, 5), expected);
    }

    #[test]
    fn test_single_technique_multiple_moves_with_chunking() {
        let steps = vec![
            SolveStep::Placement {
                row: 0,
                col: 0,
                value: 1,
                flags: TechniqueFlags::NAKED_SINGLES,
                step_number: 0,
                candidates_eliminated: 9,
                related_cell_count: 6,
                difficulty_point: 1,
            },
            SolveStep::Placement {
                row: 0,
                col: 1,
                value: 2,
                flags: TechniqueFlags::NAKED_SINGLES,
                step_number: 1,
                candidates_eliminated: 8,
                related_cell_count: 6,
                difficulty_point: 1,
            },
            SolveStep::Placement {
                row: 0,
                col: 2,
                value: 3,
                flags: TechniqueFlags::NAKED_SINGLES,
                step_number: 2,
                candidates_eliminated: 7,
                related_cell_count: 6,
                difficulty_point: 1,
            },
            SolveStep::Placement {
                row: 0,
                col: 3,
                value: 4,
                flags: TechniqueFlags::NAKED_SINGLES,
                step_number: 3,
                candidates_eliminated: 6,
                related_cell_count: 6,
                difficulty_point: 1,
            },
        ];
        let solve_path = SolvePath { steps };

        let formatted = format_solve_path(&solve_path, 3);
        assert_eq!(formatted[0], "Naked Singles:");
        // Each step should be on its own line
        assert!(formatted[1].contains("#1 R1C1=1"));
        assert!(formatted[2].contains("#2 R1C2=2"));
        assert!(formatted[3].contains("#3 R1C3=3"));
        assert!(formatted[4].contains("#4 R1C4=4"));
    }

    #[test]
    fn test_multiple_techniques_and_mixed_chunking() {
        let steps = vec![
            SolveStep::Placement {
                row: 0,
                col: 0,
                value: 1,
                flags: TechniqueFlags::NAKED_SINGLES,
                step_number: 0,
                candidates_eliminated: 9,
                related_cell_count: 6,
                difficulty_point: 1,
            },
            SolveStep::Placement {
                row: 1,
                col: 0,
                value: 3,
                flags: TechniqueFlags::HIDDEN_SINGLES,
                step_number: 1,
                candidates_eliminated: 8,
                related_cell_count: 9,
                difficulty_point: 2,
            },
            SolveStep::CandidateElimination {
                row: 2,
                col: 0,
                value: 6,
                flags: TechniqueFlags::HIDDEN_PAIRS,
                step_number: 2,
                candidates_eliminated: 3,
                related_cell_count: 4,
                difficulty_point: 3,
            },
        ];
        let solve_path = SolvePath { steps };

        let formatted = format_solve_path(&solve_path, 3);
        assert_eq!(formatted[0], "Naked Singles:");
        assert!(formatted[1].contains("#1 R1C1=1"));
        assert_eq!(formatted[2], "Hidden Singles:");
        assert!(formatted[3].contains("#2 R2C1=3"));
        assert_eq!(formatted[4], "Hidden Pairs:");
        assert!(formatted[5].contains("#3 -6@R3C1"));
    }
}
