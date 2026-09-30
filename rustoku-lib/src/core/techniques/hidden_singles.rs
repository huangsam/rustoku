use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule, units};

/// Hidden singles technique implementation.
///
/// A hidden single occurs when a candidate digit appears in only one cell within a house
/// (row, column, or box), even if that cell contains other candidates. Because the digit
/// must appear somewhere in the unit, it must be placed in that unique cell, eliminating
/// any other candidates from it.
///
/// ### Example
/// In a row with candidate distributions where digit 1 appears only in cell `(0, 3)` (e.g.
/// candidate mask `{1, 4}` while no other cell in row 0 contains candidate 1), digit 1
/// is placed at `(0, 3)`.
///
/// See: <https://hodoku.sourceforge.net/en/tech_singles.php#h1>
pub struct HiddenSingles;

impl TechniqueRule for HiddenSingles {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_placements_made = false;

        // Helper closure to evaluate hidden singles within a specific house (row, col, or box)
        let check_unit_hidden_singles =
            |unit_cells: &[(usize, usize)],
             prop: &mut TechniquePropagator,
             path: &mut SolvePath| {
                let mut unit_placement_made = false;
                // Step 1: For each candidate digit 1..=9, count occurrences within the unit
                for cand_val in 1..=9 {
                    let cand_bit = 1 << (cand_val - 1);
                    let mut potential_cell: Option<(usize, usize)> = None;
                    let mut cand_occurrences = 0;

                    for &(r, c) in unit_cells.iter() {
                        if prop.board.is_empty(r, c) {
                            let cell_cand_mask = prop.candidates.get(r, c);
                            if (cell_cand_mask & cand_bit) != 0 {
                                cand_occurrences += 1;
                                potential_cell = Some((r, c));
                            }
                        }
                    }

                    // Step 2: If the candidate appears in exactly one cell in the unit, place it
                    if cand_occurrences == 1
                        && let Some((r, c)) = potential_cell
                        && prop.board.is_empty(r, c)
                    {
                        prop.place_and_update(r, c, cand_val, self.flags(), path);
                        unit_placement_made = true;
                    }
                }
                unit_placement_made
            };

        // Step 3: Scan all 9 rows for hidden singles
        for r in 0..9 {
            let cells = units::row_cells(r);
            if check_unit_hidden_singles(&cells, prop, path) {
                overall_placements_made = true;
            }
        }

        // Step 4: Scan all 9 columns for hidden singles
        for c in 0..9 {
            let cells = units::col_cells(c);
            if check_unit_hidden_singles(&cells, prop, path) {
                overall_placements_made = true;
            }
        }

        // Step 5: Scan all 9 3x3 boxes for hidden singles
        for box_idx in 0..9 {
            let cells = units::box_cells(box_idx);
            if check_unit_hidden_singles(&cells, prop, path) {
                overall_placements_made = true;
            }
        }
        overall_placements_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::HIDDEN_SINGLES
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, TechniqueFlags};

    #[test]
    fn test_hidden_singles_places_in_correct_cell() {
        // Hodoku hidden single example
        // https://hodoku.sourceforge.net/en/show_example.php?file=h101&tech=Hidden+Single
        let s = "008007000016083000000000051107290000000000000000046307290000000000860140000300700";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::HIDDEN_SINGLES);
        let mut path = SolvePath::default();
        rustoku.techniques_make_valid_changes(&mut path);

        let placements: Vec<_> = path
            .steps
            .iter()
            .filter_map(|step| match step {
                crate::core::SolveStep::Placement {
                    row,
                    col,
                    value,
                    flags,
                    ..
                } if flags.contains(TechniqueFlags::HIDDEN_SINGLES) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !placements.is_empty(),
            "Hidden singles should produce at least one placement"
        );

        // Verify each placement is valid in the final board
        for &(r, c, v) in &placements {
            assert!((1..=9).contains(&v), "Placed value must be 1-9, got {v}");
            assert_eq!(
                rustoku.board.get(r, c),
                v,
                "Board cell ({r},{c}) should be {v} after hidden single"
            );
        }

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
