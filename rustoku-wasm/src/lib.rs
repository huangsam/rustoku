use rustoku_lib::bind::{
    candidates_grid, generate_clues_str, generate_str, is_valid_solution, solve_all_str,
    solve_any_str, solve_with_steps, technique_flags_from_str,
};
use wasm_bindgen::JsValue;
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
    solve_any_str(board_str).ok().flatten().unwrap_or_default()
}

/// Finds all solutions for a Sudoku puzzle.
/// Returns an array of 81-character strings.
/// Returns an empty array if no solution exists or the input is invalid.
#[wasm_bindgen]
pub fn solve_all(board_str: &str) -> JsValue {
    let boards = solve_all_str(board_str).unwrap_or_default();
    serde_wasm_bindgen::to_value(&boards).unwrap_or(JsValue::NULL)
}

/// Solves a puzzle and returns a step-by-step solution trace.
///
/// Returns an object `{ board: string, steps: Step[] }`.
/// `difficulty` is one of `"easy"`, `"medium"`, `"hard"`, `"expert"`.
/// Returns `null` if the puzzle is unsolvable or the input is invalid.
#[wasm_bindgen]
pub fn solve_steps(board_str: &str, difficulty: &str) -> JsValue {
    let flags = match technique_flags_from_str(difficulty) {
        Ok(f) => f,
        Err(_) => return JsValue::NULL,
    };
    match solve_with_steps(board_str, flags) {
        Ok(Some(output)) => serde_wasm_bindgen::to_value(&output).unwrap_or(JsValue::NULL),
        _ => JsValue::NULL,
    }
}

/// Returns the candidate digits for every cell as a 9×9 array.
/// Filled cells return `[]`. Returns `null` if the input is invalid.
#[wasm_bindgen]
pub fn candidates(board_str: &str) -> JsValue {
    match candidates_grid(board_str) {
        Ok(grid) => serde_wasm_bindgen::to_value(&grid).unwrap_or(JsValue::NULL),
        Err(_) => JsValue::NULL,
    }
}

/// Generates a Sudoku puzzle of the specified difficulty.
/// Valid inputs: `"easy"`, `"medium"`, `"hard"`, `"expert"`.
/// Returns the 81-character string, or an empty string on failure.
#[wasm_bindgen]
pub fn generate(difficulty: &str) -> String {
    generate_str(difficulty).unwrap_or_default()
}

/// Generates a Sudoku puzzle with exactly `num_clues` given cells (17–81).
/// Returns the 81-character string, or an empty string on failure.
#[wasm_bindgen]
pub fn generate_clues(num_clues: u32) -> String {
    generate_clues_str(num_clues as usize).unwrap_or_default()
}

/// Checks if an 81-character Sudoku string is a valid solved board.
#[wasm_bindgen]
pub fn check(board_str: &str) -> bool {
    is_valid_solution(board_str).unwrap_or(false)
}
