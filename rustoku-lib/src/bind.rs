//! High-level helpers intended for language bindings (`rustoku-wasm`, `rustoku-py`, …).
//!
//! Rust consumers of `rustoku-lib` should use the core [`crate::Rustoku`] API directly.
//! This module deliberately sits at `rustoku_lib::bind` and is **not** re-exported
//! at the crate root so it stays out of the way.
//!
//! These functions wrap the core [`Rustoku`] primitives and return simple, fully-owned
//! Rust types so that binding crates (`rustoku-wasm`, `rustoku-py`, …) contain no
//! solving logic of their own – they only marshal results into the target language's
//! type system.

use crate::core::{Difficulty, SolveStep, TechniqueFlags};
use crate::error::RustokuError;
use crate::format::format_line;
use crate::{Rustoku, generate_board, generate_board_by_difficulty};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// ── Step / Solution types ─────────────────────────────────────────────────────

/// Flat, serialisable representation of a single solve step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveStepInfo {
    /// `"placement"` or `"elimination"`.
    #[serde(rename = "type")]
    pub step_type: String,
    pub row: usize,
    pub col: usize,
    pub value: u8,
    /// Human-readable name of the technique used (e.g. `"Naked Singles"`).
    pub technique: String,
    pub step_number: u32,
    pub candidates_eliminated: u32,
    pub related_cell_count: u8,
    /// Difficulty metric (0 = trivial, 10 = hardest).
    pub difficulty_point: u8,
}

impl From<&SolveStep> for SolveStepInfo {
    fn from(step: &SolveStep) -> Self {
        match step {
            SolveStep::Placement {
                row,
                col,
                value,
                flags,
                step_number,
                candidates_eliminated,
                related_cell_count,
                difficulty_point,
            } => SolveStepInfo {
                step_type: "placement".into(),
                row: *row,
                col: *col,
                value: *value,
                technique: flags.to_string(),
                step_number: *step_number,
                candidates_eliminated: *candidates_eliminated,
                related_cell_count: *related_cell_count,
                difficulty_point: *difficulty_point,
            },
            SolveStep::CandidateElimination {
                row,
                col,
                value,
                flags,
                step_number,
                candidates_eliminated,
                related_cell_count,
                difficulty_point,
            } => SolveStepInfo {
                step_type: "elimination".into(),
                row: *row,
                col: *col,
                value: *value,
                technique: flags.to_string(),
                step_number: *step_number,
                candidates_eliminated: *candidates_eliminated,
                related_cell_count: *related_cell_count,
                difficulty_point: *difficulty_point,
            },
        }
    }
}

/// Structured output from a solve-with-steps operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveOutput {
    /// The solved board as an 81-character string.
    pub board: String,
    /// Ordered list of steps taken to reach the solution.
    pub steps: Vec<SolveStepInfo>,
}

// ── Helper functions ──────────────────────────────────────────────────────────

/// Maps a difficulty string to cumulative [`TechniqueFlags`].
///
/// Each level includes all techniques from lower levels:
/// `"easy"` ⊂ `"medium"` ⊂ `"hard"` ⊂ `"expert"`.
pub fn technique_flags_from_str(s: &str) -> Result<TechniqueFlags, RustokuError> {
    match s.to_lowercase().as_str() {
        "easy" => Ok(TechniqueFlags::EASY),
        "medium" => Ok(TechniqueFlags::EASY | TechniqueFlags::MEDIUM),
        "hard" => Ok(TechniqueFlags::EASY | TechniqueFlags::MEDIUM | TechniqueFlags::HARD),
        "expert" => Ok(TechniqueFlags::all()),
        _ => Err(RustokuError::UnknownDifficulty(s.to_string())),
    }
}

/// Solves `puzzle` and returns the first solution as an 81-char string, or `None` if unsolvable.
pub fn solve_any_str(puzzle: &str) -> Result<Option<String>, RustokuError> {
    let mut rustoku = Rustoku::new_from_str(puzzle)?;
    Ok(rustoku.solve_any().map(|s| format_line(&s.board)))
}

/// Solves `puzzle` and returns **all** solutions as 81-char strings.
pub fn solve_all_str(puzzle: &str) -> Result<Vec<String>, RustokuError> {
    let mut rustoku = Rustoku::new_from_str(puzzle)?;
    Ok(rustoku
        .solve_all()
        .into_iter()
        .map(|s| format_line(&s.board))
        .collect())
}

/// Solves `puzzle` using `flags` and returns a full step trace, or `None` if unsolvable.
pub fn solve_with_steps(
    puzzle: &str,
    flags: TechniqueFlags,
) -> Result<Option<SolveOutput>, RustokuError> {
    let mut rustoku = Rustoku::new_from_str(puzzle)?.with_techniques(flags);
    Ok(rustoku.solve_any().map(|solution| SolveOutput {
        board: format_line(&solution.board),
        steps: solution
            .solve_path
            .steps
            .iter()
            .map(SolveStepInfo::from)
            .collect(),
    }))
}

/// Returns the candidate digits for every cell as a 9×9 grid.
///
/// Filled cells return an empty `Vec`. Empty cells return the digits (1–9)
/// still possible for that cell given the current constraints.
pub fn candidates_grid(puzzle: &str) -> Result<Vec<Vec<Vec<u8>>>, RustokuError> {
    let rustoku = Rustoku::new_from_str(puzzle)?;
    Ok((0..9)
        .map(|r| {
            (0..9)
                .map(|c| {
                    if rustoku.board.get(r, c) != 0 {
                        vec![]
                    } else {
                        rustoku.candidates.get_candidates(r, c)
                    }
                })
                .collect()
        })
        .collect())
}

/// Generates a puzzle for the given difficulty string and returns it as an 81-char string.
pub fn generate_str(difficulty: &str) -> Result<String, RustokuError> {
    let diff = Difficulty::from_str(difficulty)
        .map_err(|_| RustokuError::UnknownDifficulty(difficulty.to_string()))?;
    generate_board_by_difficulty(diff, 100).map(|b| format_line(&b))
}

/// Generates a puzzle with exactly `n` given clues (17–81) and returns it as an 81-char string.
pub fn generate_clues_str(n: usize) -> Result<String, RustokuError> {
    generate_board(n).map(|b| format_line(&b))
}

/// Returns `true` if `puzzle` is a fully-solved, valid Sudoku board.
pub fn is_valid_solution(puzzle: &str) -> Result<bool, RustokuError> {
    Rustoku::new_from_str(puzzle).map(|r| r.is_solved())
}
