use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule, units};

/// X-Wing technique implementation (2-Fish).
///
/// An X-Wing occurs when a candidate digit appears in exactly two cells in each of two parallel
/// base lines (rows or columns), and both lines share the same two perpendicular cover lines.
/// Because the digit must appear in one of the two cells in each base line, it cannot appear
/// elsewhere along those cover lines.
///
/// ### Example (Row-based)
/// If candidate 5 appears only in columns 2 and 7 of row 1, and columns 2 and 7 of row 4,
/// then 5 can be eliminated from columns 2 and 7 in all other rows.
///
/// See: <https://hodoku.sourceforge.net/en/tech_fishb.php#bf2>
pub struct XWing;

impl XWing {
    /// Finds row-based X-Wings (base rows, cover columns) and eliminates candidate occurrences.
    fn find_row_based_x_wings(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        // Step 1: Collect all rows where the candidate appears in exactly 2 cells
        let rows_with_two = units::find_units_with_n_candidates(
            candidate_bit,
            2,
            prop.candidates,
            prop.board,
            units::UnitType::Row,
        );

        let mut eliminations_made = false;

        // Step 2: Compare each pair of candidate rows to find matching column pairs
        for i in 0..rows_with_two.len() {
            for j in (i + 1)..rows_with_two.len() {
                let (r1, ref cols1) = rows_with_two[i];
                let (r2, ref cols2) = rows_with_two[j];

                if cols1[0] == cols2[0] && cols1[1] == cols2[1] {
                    let c1 = cols1[0];
                    let c2 = cols1[1];

                    // Step 3: By the pigeonhole principle, candidate X is locked into the 4 intersection
                    // cells (r1/r2 x c1/c2). Eliminate candidate X from cover columns c1 and c2 in all other rows.
                    eliminations_made |= Self::eliminate_from_columns_excluding_rows(
                        prop,
                        candidate_bit,
                        c1,
                        c2,
                        &[r1, r2],
                        path,
                        flags,
                    );
                }
            }
        }

        eliminations_made
    }

    /// Finds column-based X-Wings (base columns, cover rows) and eliminates candidate occurrences.
    fn find_column_based_x_wings(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        // Step 1: Collect all columns where the candidate appears in exactly 2 cells
        let cols_with_two = units::find_units_with_n_candidates(
            candidate_bit,
            2,
            prop.candidates,
            prop.board,
            units::UnitType::Column,
        );

        let mut eliminations_made = false;

        // Step 2: Compare each pair of candidate columns to find matching row pairs
        for i in 0..cols_with_two.len() {
            for j in (i + 1)..cols_with_two.len() {
                let (c1, ref rows1) = cols_with_two[i];
                let (c2, ref rows2) = cols_with_two[j];

                if rows1[0] == rows2[0] && rows1[1] == rows2[1] {
                    let r1 = rows1[0];
                    let r2 = rows1[1];

                    // Step 3: By the pigeonhole principle, candidate X is locked into the 4 intersection
                    // cells (r1/r2 x c1/c2). Eliminate candidate X from cover rows r1 and r2 in all other columns.
                    eliminations_made |= Self::eliminate_from_rows_excluding_columns(
                        prop,
                        candidate_bit,
                        r1,
                        r2,
                        &[c1, c2],
                        path,
                        flags,
                    );
                }
            }
        }

        eliminations_made
    }

    /// Eliminates `candidate_bit` from two cover columns, skipping cells in `exclude_rows`.
    fn eliminate_from_columns_excluding_rows(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        col1: usize,
        col2: usize,
        exclude_rows: &[usize],
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        for col in [col1, col2] {
            for row in 0..9 {
                if !exclude_rows.contains(&row)
                    && prop.board.is_empty(row, col)
                    && (prop.candidates.get(row, col) & candidate_bit) != 0
                {
                    eliminations_made |=
                        prop.eliminate_candidate(row, col, candidate_bit, flags, path);
                }
            }
        }

        eliminations_made
    }

    /// Eliminates `candidate_bit` from two cover rows, skipping cells in `exclude_cols`.
    fn eliminate_from_rows_excluding_columns(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        row1: usize,
        row2: usize,
        exclude_cols: &[usize],
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        for row in [row1, row2] {
            for col in 0..9 {
                if !exclude_cols.contains(&col)
                    && prop.board.is_empty(row, col)
                    && (prop.candidates.get(row, col) & candidate_bit) != 0
                {
                    eliminations_made |=
                        prop.eliminate_candidate(row, col, candidate_bit, flags, path);
                }
            }
        }

        eliminations_made
    }
}

impl TechniqueRule for XWing {
    /// Applies row-based and column-based X-Wing searches for each candidate digit 1..=9.
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut eliminations_made = false;

        for candidate_val in 1..=9 {
            let candidate_bit = 1 << (candidate_val - 1);

            eliminations_made |=
                Self::find_row_based_x_wings(prop, candidate_bit, path, self.flags());
            eliminations_made |=
                Self::find_column_based_x_wings(prop, candidate_bit, path, self.flags());
        }

        eliminations_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::X_WING
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_xwing_eliminates_from_correct_lines() {
        // Hodoku X-Wing example
        // https://hodoku.sourceforge.net/en/show_example.php?file=bf201&tech=X-Wing
        let s = "000000000760003002002640009403900070000004903005000020010560000370090041000000060";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::X_WING);
        let mut path = SolvePath::default();
        rustoku.techniques_make_valid_changes(&mut path);

        let eliminations: Vec<_> = path
            .steps
            .iter()
            .filter_map(|step| match step {
                SolveStep::CandidateElimination {
                    row,
                    col,
                    value,
                    flags,
                    ..
                } if flags.contains(TechniqueFlags::X_WING) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "X-Wing should produce at least one candidate elimination"
        );

        for &(r, c, v) in &eliminations {
            let cand_bit = 1u16 << (v - 1);
            let remaining = rustoku.candidates.get(r, c);
            assert_eq!(
                remaining & cand_bit,
                0,
                "Candidate {v} should be eliminated from ({r},{c}) by X-Wing"
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
