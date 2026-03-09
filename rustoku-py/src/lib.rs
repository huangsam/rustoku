use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rustoku_lib::core::Difficulty;
use rustoku_lib::{Rustoku, generate_board_by_difficulty};

fn parse_difficulty(s: &str) -> PyResult<Difficulty> {
    match s.to_lowercase().as_str() {
        "easy" => Ok(Difficulty::Easy),
        "medium" => Ok(Difficulty::Medium),
        "hard" => Ok(Difficulty::Hard),
        "expert" => Ok(Difficulty::Expert),
        other => Err(PyValueError::new_err(format!(
            "unknown difficulty {:?}; expected one of: easy, medium, hard, expert",
            other
        ))),
    }
}

fn board_to_line(display: String) -> String {
    display
        .split("Line format: ")
        .nth(1)
        .unwrap_or("")
        .trim()
        .to_string()
}

/// Solves a Sudoku puzzle.
///
/// Returns the 81-character solved string, or an empty string if the puzzle
/// cannot be solved. The input must be an 81-character string where `0`, `.`,
/// or `_` represent empty cells and `1`-`9` represent clues.
///
/// Raises `ValueError` if the input is malformed.
#[pyfunction]
fn solve(puzzle: &str) -> PyResult<String> {
    let mut rustoku =
        Rustoku::new_from_str(puzzle).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(rustoku
        .solve_any()
        .map(|s| board_to_line(format!("{}", s.board)))
        .unwrap_or_default())
}

/// Generates a Sudoku puzzle of the given difficulty.
///
/// Valid inputs: `"easy"`, `"medium"`, `"hard"`, `"expert"`.
/// Returns the 81-character puzzle string.
///
/// Raises `ValueError` if the difficulty string is invalid or generation fails.
#[pyfunction]
fn generate(difficulty: &str) -> PyResult<String> {
    let diff = parse_difficulty(difficulty)?;
    let board = generate_board_by_difficulty(diff, 100)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(board_to_line(format!("{}", board)))
}

/// Checks if an 81-character Sudoku string is a valid, fully-solved board.
///
/// Returns `True` if the board is complete and valid, `False` otherwise.
///
/// Raises `ValueError` if the input is malformed.
#[pyfunction]
fn check(board_str: &str) -> PyResult<bool> {
    let rustoku =
        Rustoku::new_from_str(board_str).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(rustoku.is_solved())
}

/// Rustoku — lightning-fast Sudoku solving and generation.
#[pymodule]
fn rustoku(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    m.add_function(wrap_pyfunction!(generate, m)?)?;
    m.add_function(wrap_pyfunction!(check, m)?)?;
    Ok(())
}
