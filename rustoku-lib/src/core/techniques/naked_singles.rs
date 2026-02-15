use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule};

/// Naked singles technique implementation.
///
/// A naked single occurs when a cell has only one possible candidate number remaining.
/// Since the cell must contain that number, we can place it immediately.
///
/// If a cell has only one candidate, that candidate must be the correct value for that cell.
/// This is the most basic and fundamental Sudoku solving technique.
///
/// If a cell has candidates {5}, then 5 must be placed in that cell.
///
/// 1. Scan all empty cells
/// 2. If a cell has exactly one candidate, place that number
/// 3. Update constraints and candidates for the entire board
pub struct NakedSingles;

impl TechniqueRule for NakedSingles {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut placements_made = false;

        for r in 0..9 {
            for c in 0..9 {
                if prop.board.is_empty(r, c) {
                    let cand_mask = prop.candidates.get(r, c);
                    if cand_mask.count_ones() == 1 {
                        let num = cand_mask.trailing_zeros() as u8 + 1;
                        prop.place_and_update(r, c, num, self.flags(), path);
                        placements_made = true;
                    }
                }
            }
        }
        placements_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::NAKED_SINGLES
    }
}
