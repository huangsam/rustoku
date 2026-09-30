//! Board generator for Sudoku puzzles with configurable clues, symmetry, and difficulty.
//!
//! Provides the [`BoardGenerator`] builder, [`Symmetry`] types, and top-level generation helpers.

use std::collections::HashSet;

use rand::prelude::SliceRandom;
use rand::rng;

use super::board::Board;
use super::solution::SolveStep;
use super::solver::Rustoku;
use super::techniques::flags::{Difficulty, TechniqueFlags};
use crate::error::RustokuError;

/// Represents the type of symmetry to apply during board generation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Symmetry {
    /// No symmetry (clues placed randomly).
    #[default]
    None,
    /// 180-degree rotational (point) symmetry.
    Rotational180,
    /// 90-degree rotational symmetry.
    Rotational90,
    /// Mirror symmetry across the vertical center line.
    MirrorVertical,
    /// Mirror symmetry across the horizontal center line.
    MirrorHorizontal,
    /// Mirror symmetry across the main diagonal (top-left to bottom-right).
    MirrorDiagonal,
}

impl Symmetry {
    /// Returns the symmetric partners for a given cell (r, c).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustoku_lib::core::Symmetry;
    ///
    /// let partners = Symmetry::Rotational180.get_partners(0, 0);
    /// assert!(partners.contains(&(0, 0)));
    /// assert!(partners.contains(&(8, 8)));
    /// ```
    pub fn get_partners(&self, r: usize, c: usize) -> Vec<(usize, usize)> {
        let mut partners = HashSet::new();
        partners.insert((r, c));

        match self {
            Symmetry::None => {}
            Symmetry::Rotational180 => {
                partners.insert((8 - r, 8 - c));
            }
            Symmetry::Rotational90 => {
                partners.insert((c, 8 - r));
                partners.insert((8 - r, 8 - c));
                partners.insert((8 - c, r));
            }
            Symmetry::MirrorVertical => {
                partners.insert((r, 8 - c));
            }
            Symmetry::MirrorHorizontal => {
                partners.insert((8 - r, c));
            }
            Symmetry::MirrorDiagonal => {
                partners.insert((c, r));
            }
        }

        partners.into_iter().collect()
    }
}

/// A builder for generating Sudoku puzzles with various constraints and properties.
///
/// `BoardGenerator` provides a unified interface for creating puzzles with specific
/// clue counts, symmetry types, and difficulty levels.
///
/// # Example
///
/// ```
/// use rustoku_lib::{BoardGenerator, Symmetry};
///
/// let board = BoardGenerator::new()
///     .clues(25)
///     .symmetry(Symmetry::Rotational180)
///     .generate();
///
/// assert!(board.is_ok());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BoardGenerator {
    num_clues: usize,
    symmetry: Symmetry,
    difficulty: Option<Difficulty>,
    max_attempts: usize,
}

impl Default for BoardGenerator {
    fn default() -> Self {
        Self {
            num_clues: 30,
            symmetry: Symmetry::None,
            difficulty: None,
            max_attempts: 1,
        }
    }
}

impl BoardGenerator {
    /// Creates a new `BoardGenerator` with default settings (30 clues, no symmetry).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the target number of clues for the generated puzzle.
    pub fn clues(mut self, num_clues: usize) -> Self {
        self.num_clues = num_clues;
        self
    }

    /// Sets the type of symmetry to apply to the generated puzzle.
    pub fn symmetry(mut self, symmetry: Symmetry) -> Self {
        self.symmetry = symmetry;
        self
    }

    /// Sets the target difficulty for the generated puzzle.
    ///
    /// If a difficulty is set, the generator will attempt to find a puzzle
    /// of that exact difficulty by repeatedly generating candidates.
    pub fn difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = Some(difficulty);
        self
    }

    /// Sets the maximum number of attempts when generating a puzzle with a specific difficulty.
    pub fn max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Generates a new Sudoku puzzle based on the current configuration.
    pub fn generate(&self) -> Result<Board, RustokuError> {
        if let Some(target_difficulty) = self.difficulty {
            self.generate_with_difficulty(target_difficulty)
        } else {
            self.generate_single()
        }
    }

    /// Generates a valid Sudoku board with a unique solution and requested symmetry using the dig-hole method.
    fn generate_single(&self) -> Result<Board, RustokuError> {
        if !(17..=81).contains(&self.num_clues) {
            return Err(RustokuError::InvalidClueCount);
        }

        // 1. Generate a complete, randomized, valid 81-cell solved board
        let mut rustoku = Rustoku::new(Board::default())?;
        let solution = rustoku.solve_any().ok_or(RustokuError::DuplicateValues)?;
        let mut board = solution.board;

        // 2. Partition all 81 cells into symmetry groups
        let mut visited = [[false; 9]; 9];
        let mut groups = Vec::new();

        // Iterate in a deterministic but shuffled order to ensure variety
        let mut cells: Vec<(usize, usize)> = board.iter_cells().collect();
        cells.shuffle(&mut rng());

        for (r, c) in cells {
            if !visited[r][c] {
                let partners = self.symmetry.get_partners(r, c);
                for &(pr, pc) in &partners {
                    visited[pr][pc] = true;
                }
                groups.push(partners);
            }
        }

        // Re-shuffle groups to ensure we don't always try to remove the same regions first
        groups.shuffle(&mut rng());

        let mut clues = 81;

        // 3. Iteratively carve out clues while maintaining a unique solution
        for group in groups {
            if clues <= self.num_clues {
                break;
            }

            // Collect clues currently filled in this symmetry group
            let mut group_clues = Vec::new();
            for &(r, c) in &group {
                let val = board.cells[r][c];
                if val != 0 {
                    group_clues.push((r, c, val));
                }
            }

            if group_clues.is_empty() {
                continue;
            }

            // Tentatively clear the entire symmetry group
            for &(r, c, _) in &group_clues {
                board.cells[r][c] = 0;
            }

            // Uniqueness check: bounded solve to verify exactly 1 solution exists
            if Rustoku::new(board)?.solve_until(2).len() != 1 {
                // More than 1 solution found (or 0): restore clues to maintain uniqueness
                for &(r, c, val) in &group_clues {
                    board.cells[r][c] = val;
                }
            } else {
                // Successfully removed clues while maintaining unique solvability
                clues -= group_clues.len();
            }
        }

        // Final safety check to ensure generated board is valid and uniquely solvable
        if Rustoku::new(board)?.solve_until(2).len() != 1 {
            return Err(RustokuError::GenerateFailure);
        }

        Ok(board)
    }

    /// Generates a Sudoku board classified at a specific difficulty level without requiring guessing.
    fn generate_with_difficulty(
        &self,
        target_difficulty: Difficulty,
    ) -> Result<Board, RustokuError> {
        use rand::RngExt;

        for _ in 0..self.max_attempts {
            // Heuristic clue count range tailored to the desired difficulty level
            let clues = match target_difficulty {
                Difficulty::Easy => rand::rng().random_range(34..=42),
                Difficulty::Medium => rand::rng().random_range(28..=34),
                Difficulty::Hard => rand::rng().random_range(22..=28),
                Difficulty::Expert => rand::rng().random_range(17..=22),
            };

            // Generate a uniquely solvable candidate board with symmetry
            let mut sub_generator = *self;
            sub_generator.num_clues = clues;
            sub_generator.difficulty = None; // Avoid recursion

            if let Ok(board) = sub_generator.generate_single() {
                let mut rustoku = Rustoku::builder()
                    .board(board)
                    .techniques(TechniqueFlags::all())
                    .build()?;

                // Evaluate whether human deduction techniques can fully solve the board
                let solutions = rustoku.solve_all();
                if solutions.len() == 1 {
                    let solution = &solutions[0];

                    // Inspect the solve path to evaluate difficulty and verify no guessing occurred
                    let mut max_difficulty = Difficulty::Easy;
                    let mut required_guessing = false;

                    for step in &solution.solve_path.steps {
                        match step {
                            SolveStep::Placement { flags, .. } => {
                                // Placements with empty flags indicate DFS backtracking / guessing
                                if flags.is_empty() {
                                    required_guessing = true;
                                    break;
                                }
                                let step_diff = flags.difficulty();
                                if step_diff > max_difficulty {
                                    max_difficulty = step_diff;
                                }
                            }
                            SolveStep::CandidateElimination { flags, .. } => {
                                if !flags.is_empty() {
                                    let step_diff = flags.difficulty();
                                    if step_diff > max_difficulty {
                                        max_difficulty = step_diff;
                                    }
                                }
                            }
                        }
                    }

                    // Accept board only if solvable purely through logic and matching target difficulty
                    if !required_guessing && max_difficulty == target_difficulty {
                        return Ok(board);
                    }
                }
            }
        }

        Err(RustokuError::GenerateFailure)
    }
}

/// Generates a new Sudoku puzzle with a unique solution.
///
/// The `num_clues` parameter specifies the desired number of initially
/// filled cells (clues) in the generated puzzle. Fewer clues generally
/// result in a harder puzzle. The actual number of clues may be slightly
/// more than `num_clues` if it's impossible to remove more numbers
/// while maintaining a unique solution.
///
/// This is a convenience shim for `BoardGenerator::new().clues(num_clues).generate()`.
///
/// # Examples
///
/// ```
/// use rustoku_lib::generate_board;
///
/// let puzzle = generate_board(30);
/// assert!(puzzle.is_ok());
/// ```
pub fn generate_board(num_clues: usize) -> Result<Board, RustokuError> {
    BoardGenerator::new().clues(num_clues).generate()
}

/// Generates a new Sudoku puzzle that matches a specific difficulty level.
///
/// This is a convenience shim for `BoardGenerator::new().difficulty(difficulty).max_attempts(max_attempts).generate()`.
///
/// # Examples
///
/// ```
/// use rustoku_lib::{Difficulty, generate_board_by_difficulty};
///
/// let board = generate_board_by_difficulty(Difficulty::Easy, 100);
/// assert!(board.is_ok());
/// ```
pub fn generate_board_by_difficulty(
    difficulty: Difficulty,
    max_attempts: usize,
) -> Result<Board, RustokuError> {
    BoardGenerator::new()
        .difficulty(difficulty)
        .max_attempts(max_attempts)
        .generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symmetry_partners() {
        assert_eq!(Symmetry::None.get_partners(2, 3), vec![(2, 3)]);

        let rot180 = Symmetry::Rotational180.get_partners(0, 0);
        assert!(rot180.contains(&(0, 0)));
        assert!(rot180.contains(&(8, 8)));
        assert_eq!(rot180.len(), 2);

        let rot90 = Symmetry::Rotational90.get_partners(0, 1);
        assert!(rot90.contains(&(0, 1)));
        assert!(rot90.contains(&(1, 8)));
        assert!(rot90.contains(&(8, 7)));
        assert!(rot90.contains(&(7, 0)));
        assert_eq!(rot90.len(), 4);

        let mirror_v = Symmetry::MirrorVertical.get_partners(1, 2);
        assert!(mirror_v.contains(&(1, 2)));
        assert!(mirror_v.contains(&(1, 6)));
        assert_eq!(mirror_v.len(), 2);

        let mirror_h = Symmetry::MirrorHorizontal.get_partners(1, 2);
        assert!(mirror_h.contains(&(1, 2)));
        assert!(mirror_h.contains(&(7, 2)));
        assert_eq!(mirror_h.len(), 2);

        let mirror_d = Symmetry::MirrorDiagonal.get_partners(1, 2);
        assert!(mirror_d.contains(&(1, 2)));
        assert!(mirror_d.contains(&(2, 1)));
        assert_eq!(mirror_d.len(), 2);
    }

    #[test]
    fn test_generate_with_enough_clues() {
        (20..=80).step_by(20).for_each(|num_clues| {
            let board = generate_board(num_clues)
                .expect("Board generation failed - check clue count is between 17 and 81");
            let mut rustoku =
                Rustoku::new(board).expect("Rustoku creation failed from generated board");
            let clues_count = board
                .cells
                .iter()
                .flatten()
                .filter(|&&cell| cell != 0)
                .count();
            assert!(
                clues_count >= num_clues,
                "Expected at least {num_clues} clues, but found {clues_count} clues"
            );

            let solutions = rustoku.solve_all();
            assert_eq!(
                1,
                solutions.len(),
                "Generated puzzle with {num_clues} clues should have a unique solution"
            );
        })
    }

    #[test]
    fn test_generate_with_too_few_clues() {
        let num_clues = 16;
        let result = generate_board(num_clues);
        assert!(matches!(result, Err(RustokuError::InvalidClueCount)));
    }

    #[test]
    fn test_generate_with_too_many_clues() {
        let num_clues = 82;
        let result = generate_board(num_clues);
        assert!(matches!(result, Err(RustokuError::InvalidClueCount)));
    }

    #[test]
    fn test_generate_by_difficulty_easy() {
        let board = generate_board_by_difficulty(Difficulty::Easy, 100)
            .expect("Failed to generate an Easy puzzle within 100 attempts");

        let mut rustoku = Rustoku::builder()
            .board(board)
            .techniques(TechniqueFlags::all())
            .build()
            .unwrap();

        let solutions = rustoku.solve_all();
        assert_eq!(solutions.len(), 1);

        let mut required_guessing = false;
        let mut max_difficulty = Difficulty::Easy;

        for step in &solutions[0].solve_path.steps {
            match step {
                crate::core::solution::SolveStep::Placement { flags, .. } => {
                    if flags.is_empty() {
                        required_guessing = true;
                    }
                    if flags.difficulty() > max_difficulty {
                        max_difficulty = flags.difficulty();
                    }
                }
                crate::core::solution::SolveStep::CandidateElimination { flags, .. } => {
                    if flags.difficulty() > max_difficulty {
                        max_difficulty = flags.difficulty();
                    }
                }
            }
        }

        assert!(
            !required_guessing,
            "Easy puzzle should not require guessing"
        );
        assert_eq!(
            max_difficulty,
            Difficulty::Easy,
            "Puzzle exceeded target difficulty"
        );
    }

    #[test]
    fn test_generate_by_difficulty_hard() {
        let board = generate_board_by_difficulty(Difficulty::Hard, 1000)
            .expect("Failed to generate a Hard puzzle within 1000 attempts");

        let mut rustoku = Rustoku::builder()
            .board(board)
            .techniques(TechniqueFlags::all())
            .build()
            .unwrap();

        let solutions = rustoku.solve_all();
        assert_eq!(solutions.len(), 1);

        let mut required_guessing = false;
        let mut max_difficulty = Difficulty::Easy;

        for step in &solutions[0].solve_path.steps {
            match step {
                crate::core::solution::SolveStep::Placement { flags, .. } => {
                    if flags.is_empty() {
                        required_guessing = true;
                    }
                    if flags.difficulty() > max_difficulty {
                        max_difficulty = flags.difficulty();
                    }
                }
                crate::core::solution::SolveStep::CandidateElimination { flags, .. } => {
                    if flags.difficulty() > max_difficulty {
                        max_difficulty = flags.difficulty();
                    }
                }
            }
        }

        assert!(
            !required_guessing,
            "Hard puzzle should not require guessing"
        );
        assert_eq!(
            max_difficulty,
            Difficulty::Hard,
            "Puzzle exceeded target difficulty"
        );
    }
}
