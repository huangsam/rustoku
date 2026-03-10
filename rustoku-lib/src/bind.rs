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
use crate::{Rustoku, generate_board_by_difficulty};
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
fn technique_flags_from_str(s: &str) -> Result<TechniqueFlags, RustokuError> {
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

/// Solves `puzzle` using human techniques for the given `difficulty` and returns a full
/// step trace, or `None` if unsolvable.
///
/// `difficulty` is one of `"easy"`, `"medium"`, `"hard"`, `"expert"`.
pub fn solve_with_steps(
    puzzle: &str,
    difficulty: &str,
) -> Result<Option<SolveOutput>, RustokuError> {
    let flags = technique_flags_from_str(difficulty)?;
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

/// Returns `true` if `puzzle` is a fully-solved, valid Sudoku board.
pub fn is_valid_solution(puzzle: &str) -> Result<bool, RustokuError> {
    Rustoku::new_from_str(puzzle).map(|r| r.is_solved())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technique_flags_from_str_valid() {
        assert!(technique_flags_from_str("easy").is_ok());
        assert!(technique_flags_from_str("medium").is_ok());
        assert!(technique_flags_from_str("hard").is_ok());
        assert!(technique_flags_from_str("expert").is_ok());
        assert!(technique_flags_from_str("EASY").is_ok()); // case insensitive
    }

    #[test]
    fn test_technique_flags_from_str_invalid() {
        assert!(matches!(
            technique_flags_from_str("invalid"),
            Err(RustokuError::UnknownDifficulty(_))
        ));
    }

    #[test]
    fn test_solve_any_str_solvable() {
        let puzzle =
            "4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......";
        let result = solve_any_str(puzzle);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_solve_any_str_unsolvable() {
        // Invalid puzzle with conflicts
        let puzzle =
            "111111111111111111111111111111111111111111111111111111111111111111111111111111";
        let result = solve_any_str(puzzle);
        // All 1s creates a conflict, so it's invalid and returns an error
        assert!(result.is_err());
    }

    #[test]
    fn test_solve_any_str_invalid_input() {
        let puzzle = "invalid";
        assert!(solve_any_str(puzzle).is_err());
    }

    #[test]
    fn test_solve_all_str() {
        let puzzle =
            "4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......";
        let result = solve_all_str(puzzle);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_solve_with_steps() {
        let puzzle =
            "4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......";
        let result = solve_with_steps(puzzle, "expert");
        assert!(result.is_ok());
        let output = result.unwrap().unwrap();
        assert_eq!(output.board.len(), 81);
        assert!(!output.steps.is_empty());
    }

    #[test]
    fn test_candidates_grid() {
        let puzzle =
            "4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......";
        let result = candidates_grid(puzzle);
        assert!(result.is_ok());
        let grid = result.unwrap();
        assert_eq!(grid.len(), 9);
        assert_eq!(grid[0].len(), 9);
        // First cell is 4, so empty candidates
        assert!(grid[0][0].is_empty());
        // Some empty cell should have candidates
        assert!(!grid[0][1].is_empty());
    }

    #[test]
    fn test_generate_str_valid() {
        let result = generate_str("easy");
        assert!(result.is_ok());
        let board = result.unwrap();
        assert_eq!(board.len(), 81);
        // Should be solvable
        assert!(solve_any_str(&board).unwrap().is_some());
    }

    #[test]
    fn test_generate_str_invalid() {
        assert!(generate_str("invalid").is_err());
    }

    #[test]
    fn test_is_valid_solution_valid() {
        let solved =
            "417369825632158947958724316825437169791586432346912758289643571573291684164875293";
        assert_eq!(is_valid_solution(solved).unwrap(), true);
    }

    #[test]
    fn test_is_valid_solution_invalid() {
        let unsolved =
            "4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......";
        assert_eq!(is_valid_solution(unsolved).unwrap(), false);
    }

    #[test]
    fn test_is_valid_solution_malformed() {
        assert!(is_valid_solution("invalid").is_err());
    }
}
