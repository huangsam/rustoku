use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule};

/// Naked singles technique implementation.
///
/// A naked single occurs when an empty cell has only one possible candidate remaining.
/// Since every cell must hold a valid digit from 1 to 9, that remaining candidate must
/// be placed immediately into the cell.
///
/// ### Example
/// If cell `(8, 8)` has candidate mask `0b000100000` (candidate `{6}`), then digit 6
/// is placed at `(8, 8)`.
///
/// See: <https://hodoku.sourceforge.net/en/tech_singles.php#n1>
pub struct NakedSingles;

impl TechniqueRule for NakedSingles {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut placements_made = false;

        // Step 1: Scan all cells across the grid
        for r in 0..9 {
            for c in 0..9 {
                // Step 2: For each empty cell, check if exactly one candidate remains
                if prop.board.is_empty(r, c) {
                    let cand_mask = prop.candidates.get(r, c);
                    if cand_mask.count_ones() == 1 {
                        // Step 3: Extract the single digit using bit manipulation (1-based index)
                        let num = cand_mask.trailing_zeros() as u8 + 1;
                        // Step 4: Place the digit and propagate constraints across row, col, and box
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

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_naked_singles_places_correct_value() {
        // Board with a single empty cell at (8,8) – only candidate is 6
        // Puzzle string normalized to 0s instead of dots
        let s = "385421967194756328627983145571892634839645271246137589462579813918364752753218490";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::NAKED_SINGLES);
        let mut path = SolvePath::default();
        rustoku.techniques_make_valid_changes(&mut path);

        // Expect exactly one placement at (8,8) = 6
        let placements: Vec<_> = path
            .steps
            .iter()
            .filter_map(|step| match step {
                SolveStep::Placement {
                    row, col, value, ..
                } => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            placements.contains(&(8, 8, 6)),
            "Expected placement of 6 at (8,8), got {:?}",
            placements
        );

        // Verify that initial clues were not altered
        let original = crate::core::Board::try_from(s).unwrap();
        for r in 0..9 {
            for c in 0..9 {
                let orig_val = original.get(r, c);
                if orig_val != 0 {
                    assert_eq!(
                        rustoku.board.get(r, c),
                        orig_val,
                        "Clue at ({r},{c}) was overwritten"
                    );
                }
            }
        }
    }
}
