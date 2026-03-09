use rustoku_lib::Rustoku;
use rustoku_lib::core::Difficulty;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_panic_hook() {
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
        format!("{}", solution.board)
            .split("Line format: ")
            .nth(1)
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    }
}

/// Generates a Sudoku puzzle of the specified difficulty.
/// Valid inputs: "easy", "medium", "hard", "expert".
/// Returns the 81-character string, or an empty string if generation fails.
#[wasm_bindgen]
pub fn generate(difficulty: &str) -> String {
    let diff = match difficulty.to_lowercase().as_str() {
        "easy" => Difficulty::Easy,
        "medium" => Difficulty::Medium,
        "hard" => Difficulty::Hard,
        "expert" => Difficulty::Expert,
        _ => return String::new(),
    };

    let Ok(board) = rustoku_lib::generate_board_by_difficulty(diff, 100) else {
        return String::new();
    };

    format!("{}", board)
        .split("Line format: ")
        .nth(1)
        .unwrap_or("")
        .to_string()
}

/// Checks if an 81-character Sudoku string is a valid solved board.
#[wasm_bindgen]
pub fn check(board_str: &str) -> bool {
    let Ok(rustoku) = Rustoku::new_from_str(board_str) else {
        return false;
    };
    rustoku.is_solved()
}
