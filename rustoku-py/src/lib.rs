//! Python bindings for Rustoku — lightning-fast Sudoku solving and generation.
//!
//! ## Error Handling
//!
//! Functions validate inputs strictly and raise `ValueError` for:
//! - Malformed puzzle strings (invalid length, non-digit characters)
//! - Unknown difficulty levels (e.g., `"invalid_difficulty"`)
//! - Out-of-range generation parameters
//!
//! A puzzle that cannot be solved returns an empty string or empty list (not an error),
//! as unsolvability is a normal result, not an exceptional condition.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rustoku_lib::RustokuError;
use rustoku_lib::bind::{
    SolveOutput, candidates_grid, generate_str, is_valid_solution, solve_all_str, solve_any_str,
    solve_with_steps,
};

// ── internal helpers ──────────────────────────────────────────────────────────

fn to_py_err(e: RustokuError) -> PyErr {
    PyValueError::new_err(e.to_string())
}

fn solve_output_to_dict<'py>(py: Python<'py>, output: SolveOutput) -> PyResult<Bound<'py, PyDict>> {
    let result = PyDict::new(py);
    result.set_item("board", output.board)?;
    let steps = PyList::empty(py);
    for step in output.steps {
        let s = PyDict::new(py);
        s.set_item("type", step.step_type)?;
        s.set_item("row", step.row)?;
        s.set_item("col", step.col)?;
        s.set_item("value", step.value)?;
        s.set_item("technique", step.technique)?;
        s.set_item("step_number", step.step_number)?;
        s.set_item("candidates_eliminated", step.candidates_eliminated)?;
        s.set_item("related_cell_count", step.related_cell_count)?;
        s.set_item("difficulty_point", step.difficulty_point)?;
        steps.append(&s)?;
    }
    result.set_item("steps", &steps)?;
    Ok(result)
}

// ── exported functions ────────────────────────────────────────────────────────

/// Solves a Sudoku puzzle.
///
/// Returns the 81-character solved string, or an empty string if the puzzle
/// cannot be solved. The input must be an 81-character string where `0`, `.`,
/// or `_` represent empty cells and `1`-`9` represent clues.
///
/// Raises `ValueError` if the input is malformed.
#[pyfunction]
fn solve(puzzle: &str) -> PyResult<String> {
    solve_any_str(puzzle)
        .map_err(to_py_err)
        .map(|o| o.unwrap_or_default())
}

/// Finds all solutions for a Sudoku puzzle.
///
/// Returns a list of 81-character solved strings (may be empty if unsolvable).
///
/// Raises `ValueError` if the input is malformed.
#[pyfunction]
fn solve_all(puzzle: &str) -> PyResult<Vec<String>> {
    solve_all_str(puzzle).map_err(to_py_err)
}

/// Solves a Sudoku puzzle and returns a step-by-step solution trace.
///
/// Returns a dict `{"board": str, "steps": list[dict]}` or `None` if unsolvable.
/// Each step dict contains: `type`, `row`, `col`, `value`, `technique`,
/// `step_number`, `candidates_eliminated`, `related_cell_count`, `difficulty_point`.
///
/// `difficulty` controls which human techniques are applied: `"easy"`, `"medium"`,
/// `"hard"`, or `"expert"` (default).
///
/// Raises `ValueError` if the input is malformed or the difficulty is unknown.
#[pyfunction]
#[pyo3(signature = (puzzle, difficulty = "expert"))]
fn solve_steps<'py>(
    py: Python<'py>,
    puzzle: &str,
    difficulty: &str,
) -> PyResult<Option<Bound<'py, PyDict>>> {
    let output = solve_with_steps(puzzle, difficulty).map_err(to_py_err)?;
    output.map(|o| solve_output_to_dict(py, o)).transpose()
}

/// Returns the candidate digits for every cell in the puzzle.
///
/// Returns a 9×9 list of lists. Each inner list contains the valid candidate
/// digits (1–9) for that cell; filled cells return an empty list `[]`.
///
/// Raises `ValueError` if the input is malformed.
#[pyfunction]
fn candidates(puzzle: &str) -> PyResult<Vec<Vec<Vec<u8>>>> {
    candidates_grid(puzzle).map_err(to_py_err)
}

/// Generates a Sudoku puzzle of the given difficulty.
///
/// Valid inputs: `"easy"`, `"medium"`, `"hard"`, `"expert"`.
/// Returns the 81-character puzzle string.
///
/// Raises `ValueError` if the difficulty string is invalid or generation fails.
#[pyfunction]
fn generate(difficulty: &str) -> PyResult<String> {
    generate_str(difficulty).map_err(to_py_err)
}

/// Checks if an 81-character Sudoku string is a valid, fully-solved board.
///
/// Returns `True` if the board is complete and valid, `False` otherwise.
///
/// Raises `ValueError` if the input is malformed.
#[pyfunction]
fn check(board_str: &str) -> PyResult<bool> {
    is_valid_solution(board_str).map_err(to_py_err)
}

/// Rustoku — lightning-fast Sudoku solving and generation.
#[pymodule]
fn rustoku(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    m.add_function(wrap_pyfunction!(solve_all, m)?)?;
    m.add_function(wrap_pyfunction!(solve_steps, m)?)?;
    m.add_function(wrap_pyfunction!(candidates, m)?)?;
    m.add_function(wrap_pyfunction!(generate, m)?)?;
    m.add_function(wrap_pyfunction!(check, m)?)?;
    Ok(())
}
