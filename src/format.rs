/// Prints the current state of the Sudoku board to the console.
///
/// The board is formatted with horizontal and vertical separators to visually
/// distinguish the 3x3 boxes. This function is useful for displaying the board
/// in a human-readable format during debugging or while solving a puzzle.
///
/// It also prints a line representation of the board at the end, which is a single string
/// containing all numbers in row-major order, with empty cells represented by dots.
///
/// # Example
///
/// Prints a Sudoku board in a formatted way:
/// ```
/// use rustoku::format::print_board;
/// let board = [
///     [5, 3, 4, 6, 7, 8, 9, 1, 2],
///     [6, 7, 2, 1, 9, 5, 3, 4, 8],
///     [1, 9, 8, 3, 4, 2, 5, 6, 7],
///     [8, 5, 9, 7, 6, 1, 4, 2, 3],
///     [4, 2, 6, 8, 5, 3, 7, 9, 1],
///     [7, 1, 3, 9, 2, 4, 8, 5, 6],
///     [9, 6, 1, 5, 3, 7, 2, 8, 4],
///     [2, 8, 7, 4, 1, 9, 6, 3, 5],
///     [3, 4, 5, 2, 8, 6, 1, 7, 9],
/// ];
/// print_board(&board);
/// ```
pub fn print_board(board: &[[u8; 9]; 9]) {
    let grid = format_grid(board);
    for line in grid {
        println!("{}", line);
    }

    println!("Line format: {}", format_line(board)); // Line representation
}

/// Formats the Sudoku board into a grid representation.
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

        assert_eq!(format_grid(&board), expected);
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
        assert_eq!(format_line(&board), expected);
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

        assert_eq!(format_grid(&board), expected);
    }

    #[test]
    fn test_format_line_empty_board() {
        let board = [[0; 9]; 9];
        let expected =
            ".................................................................................";
        assert_eq!(format_line(&board), expected);
    }
}
