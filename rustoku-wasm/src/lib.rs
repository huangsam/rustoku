use rustoku_lib::core::Difficulty;
use rustoku_lib::{Rustoku, format_line};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Solves a Sudoku puzzle.
/// Returns the 81-character solved string, or an empty string if it cannot be solved.
#[wasm_bindgen]
pub fn solve(board_str: &str) -> String {
    let Ok(mut rustoku) = Rustoku::new_from_str(board_str) else {
        return String::new();
    };

    if let Some(solution) = rustoku.solve_any() {
        format_line(&solution.board)
    } else {
        String::new()
    }
}

/// Generates a Sudoku puzzle of the specified difficulty.
/// Valid inputs: "easy", "medium", "hard", "expert".
/// Returns the 81-character string, or an empty string if generation fails.
#[wasm_bindgen]
pub fn generate(difficulty: &str) -> String {
    let Ok(diff) = Difficulty::from_str(difficulty) else {
        return String::new();
    };

    let Ok(board) = rustoku_lib::generate_board_by_difficulty(diff, 100) else {
        return String::new();
    };

    format_line(&board)
}

/// Checks if an 81-character Sudoku string is a valid solved board.
#[wasm_bindgen]
pub fn check(board_str: &str) -> bool {
    let Ok(rustoku) = Rustoku::new_from_str(board_str) else {
        return false;
    };
    rustoku.is_solved()
}
