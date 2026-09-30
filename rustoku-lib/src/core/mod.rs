//! Core module for the Rustoku solver and generator.
//!
//! Provides the [`Rustoku`] solver, [`BoardGenerator`], and solution types.
//!
//! ### Architecture & Solving Pipeline
//! The engine operates across three synchronized data structures maintaining a ~1KB memory footprint:
//! - [`Board`]: 9x9 byte matrix storing current cell assignments (0 for empty, 1–9 for digits).
//! - [`Masks`]: Bitmasks for each row, column, and 3x3 box tracking placed digits.
//! - [`Candidates`]: Precomputed 9-bit bitmask cache per empty cell representing valid candidate digits.
//!
//! The solving pipeline combines deterministic human deduction with depth-first search:
//! 1. **Deterministic Constraint Propagation**: Enabled human techniques run in [`TechniquePropagator`]
//!    to eliminate candidates and place forced digits without guessing.
//! 2. **MRV-Guided Backtracking**: If techniques do not completely solve the puzzle, recursive DFS
//!    explores the state space using the Minimum Remaining Values (MRV / "fail-first") heuristic.
//! 3. **Parallel Search**: In [`Rustoku::solve_all`], top-level branches of the root MRV cell are
//!    explored in parallel across CPU threads via Rayon.
//! 4. **Lazy Iteration**: For on-demand solution streaming, [`Solutions`] provides an explicit-stack
//!    iterator that avoids computing all solutions up front.
//!
//! ### Bitmask Invariants
//!
//! | Structure | Storage | Representation | Invariant Rule |
//! | :--- | :--- | :--- | :--- |
//! | [`Masks`] | `[u16; 9]` per unit | Bit `1 << (d - 1)` set if digit `d` placed | Exactly one bit per placed digit per row, col, and box |
//! | [`Candidates`] | `[[u16; 9]; 9]` | Bit `1 << (d - 1)` set if digit `d` valid | `~(row \| col \| box) & 0x01FF` for empty; `0` for filled |

#[cfg(doc)]
use techniques::TechniquePropagator;

mod board;
mod candidates;
mod generator;
mod masks;
mod solution;
mod solutions;
mod solver;
mod techniques;

pub use board::Board;
pub use candidates::Candidates;
pub use generator::{BoardGenerator, Symmetry, generate_board, generate_board_by_difficulty};
pub use masks::Masks;
pub use solution::{Solution, SolvePath, SolveStep};
pub use solutions::Solutions;
pub use solver::{Rustoku, RustokuBuilder};
pub use techniques::flags::{Difficulty, TechniqueFlags};
