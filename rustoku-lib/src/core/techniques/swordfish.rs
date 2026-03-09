use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule, units};

/// Swordfish technique implementation.
///
/// A Swordfish is a generalization of X-Wing from 2 defining lines to 3.
/// For a candidate digit, if it appears in at most 3 positions across exactly
/// 3 rows, and those positions collectively span exactly 3 columns, then that
/// candidate can be eliminated from those 3 columns in all other rows.
/// The same logic applies symmetrically for columns → rows.
///
/// Example of row-based Swordfish:
/// If candidate 7 appears only in columns {1,4,8} across rows 2, 5, and 7,
/// and each of those rows has the candidate in at most 3 of those columns,
/// then 7 can be eliminated from columns 1, 4, and 8 in all other rows.
pub struct Swordfish;

impl Swordfish {
    /// Finds Swordfish patterns in rows and eliminates candidates from columns.
    fn find_row_based_swordfish(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        // Collect rows where the candidate appears in 2 or 3 positions
        let eligible_rows = Self::find_eligible_units(prop, candidate_bit, units::UnitType::Row);

        let mut eliminations_made = false;

        // Check all triples of eligible rows
        for i in 0..eligible_rows.len() {
            for j in (i + 1)..eligible_rows.len() {
                for k in (j + 1)..eligible_rows.len() {
                    let (r1, ref cols1) = eligible_rows[i];
                    let (r2, ref cols2) = eligible_rows[j];
                    let (r3, ref cols3) = eligible_rows[k];

                    // Compute the union of column positions
                    let mut col_set: u16 = 0;
                    for &c in cols1.iter().chain(cols2.iter()).chain(cols3.iter()) {
                        col_set |= 1 << c;
                    }

                    // Swordfish requires exactly 3 columns
                    if col_set.count_ones() == 3 {
                        let defining_rows = [r1, r2, r3];
                        let cols: Vec<usize> =
                            (0..9).filter(|&c| col_set & (1 << c) != 0).collect();

                        for &col in &cols {
                            for row in 0..9 {
                                if !defining_rows.contains(&row)
                                    && prop.board.is_empty(row, col)
                                    && (prop.candidates.get(row, col) & candidate_bit) != 0
                                {
                                    eliminations_made |= prop.eliminate_candidate(
                                        row,
                                        col,
                                        candidate_bit,
                                        flags,
                                        path,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        eliminations_made
    }

    /// Finds Swordfish patterns in columns and eliminates candidates from rows.
    fn find_column_based_swordfish(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        let eligible_cols = Self::find_eligible_units(prop, candidate_bit, units::UnitType::Column);

        let mut eliminations_made = false;

        for i in 0..eligible_cols.len() {
            for j in (i + 1)..eligible_cols.len() {
                for k in (j + 1)..eligible_cols.len() {
                    let (c1, ref rows1) = eligible_cols[i];
                    let (c2, ref rows2) = eligible_cols[j];
                    let (c3, ref rows3) = eligible_cols[k];

                    let mut row_set: u16 = 0;
                    for &r in rows1.iter().chain(rows2.iter()).chain(rows3.iter()) {
                        row_set |= 1 << r;
                    }

                    if row_set.count_ones() == 3 {
                        let defining_cols = [c1, c2, c3];
                        let rows: Vec<usize> =
                            (0..9).filter(|&r| row_set & (1 << r) != 0).collect();

                        for &row in &rows {
                            for col in 0..9 {
                                if !defining_cols.contains(&col)
                                    && prop.board.is_empty(row, col)
                                    && (prop.candidates.get(row, col) & candidate_bit) != 0
                                {
                                    eliminations_made |= prop.eliminate_candidate(
                                        row,
                                        col,
                                        candidate_bit,
                                        flags,
                                        path,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        eliminations_made
    }

    /// Finds units (rows or columns) where a candidate appears in 2 or 3 positions.
    /// Returns a vector of (unit_index, positions) tuples where positions are the
    /// column indices (for rows) or row indices (for columns).
    fn find_eligible_units(
        prop: &TechniquePropagator,
        candidate_bit: u16,
        unit_type: units::UnitType,
    ) -> Vec<(usize, Vec<usize>)> {
        let mut result = Vec::new();

        for unit_idx in 0..9 {
            let unit_cells = match unit_type {
                units::UnitType::Row => units::row_cells(unit_idx),
                units::UnitType::Column => units::col_cells(unit_idx),
            };

            let positions: Vec<usize> = unit_cells
                .iter()
                .enumerate()
                .filter(|&(_, &(r, c))| {
                    prop.board.is_empty(r, c) && (prop.candidates.get(r, c) & candidate_bit) != 0
                })
                .map(|(pos, _)| pos)
                .collect();

            if positions.len() == 2 || positions.len() == 3 {
                result.push((unit_idx, positions));
            }
        }

        result
    }
}

impl TechniqueRule for Swordfish {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut eliminations_made = false;

        for candidate_val in 1..=9 {
            let candidate_bit = 1 << (candidate_val - 1);
            eliminations_made |=
                Self::find_row_based_swordfish(prop, candidate_bit, path, self.flags());
            eliminations_made |=
                Self::find_column_based_swordfish(prop, candidate_bit, path, self.flags());
        }

        eliminations_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::SWORDFISH
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_swordfish_eliminates_from_correct_lines() {
        // Hodoku Swordfish example
        // https://hodoku.sourceforge.net/en/show_example.php?file=bf301&tech=Swordfish
        let s = "160540070008001030030800000700050069600902057000000000000030040000000016000164500";
        let mut rustoku = Rustoku::new_from_str(s).unwrap().with_techniques(
            TechniqueFlags::EASY | TechniqueFlags::MEDIUM | TechniqueFlags::SWORDFISH,
        );
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
                } if flags.contains(TechniqueFlags::SWORDFISH) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "Swordfish should produce at least one candidate elimination"
        );

        for &(r, c, v) in &eliminations {
            let cand_bit = 1u16 << (v - 1);
            let remaining = rustoku.candidates.get(r, c);
            assert_eq!(
                remaining & cand_bit,
                0,
                "Candidate {v} should be eliminated from ({r},{c}) by Swordfish"
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
