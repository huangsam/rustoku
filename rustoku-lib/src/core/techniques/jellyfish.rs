use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule, units};

/// Jellyfish technique implementation.
///
/// A Jellyfish is a generalization of Swordfish from 3 defining lines to 4.
/// For a candidate digit, if it appears in at most 4 positions across exactly
/// 4 rows, and those positions collectively span exactly 4 columns, then that
/// candidate can be eliminated from those 4 columns in all other rows.
/// The same logic applies symmetrically for columns → rows.
pub struct Jellyfish;

impl Jellyfish {
    fn find_row_based_jellyfish(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        let eligible_rows = Self::find_eligible_units(prop, candidate_bit, units::UnitType::Row);

        let mut eliminations_made = false;

        for i in 0..eligible_rows.len() {
            for j in (i + 1)..eligible_rows.len() {
                for k in (j + 1)..eligible_rows.len() {
                    for l in (k + 1)..eligible_rows.len() {
                        let (r1, ref cols1) = eligible_rows[i];
                        let (r2, ref cols2) = eligible_rows[j];
                        let (r3, ref cols3) = eligible_rows[k];
                        let (r4, ref cols4) = eligible_rows[l];

                        let mut col_set: u16 = 0;
                        for &c in cols1
                            .iter()
                            .chain(cols2.iter())
                            .chain(cols3.iter())
                            .chain(cols4.iter())
                        {
                            col_set |= 1 << c;
                        }

                        if col_set.count_ones() == 4 {
                            let defining_rows = [r1, r2, r3, r4];
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
        }

        eliminations_made
    }

    fn find_column_based_jellyfish(
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
                    for l in (k + 1)..eligible_cols.len() {
                        let (c1, ref rows1) = eligible_cols[i];
                        let (c2, ref rows2) = eligible_cols[j];
                        let (c3, ref rows3) = eligible_cols[k];
                        let (c4, ref rows4) = eligible_cols[l];

                        let mut row_set: u16 = 0;
                        for &r in rows1
                            .iter()
                            .chain(rows2.iter())
                            .chain(rows3.iter())
                            .chain(rows4.iter())
                        {
                            row_set |= 1 << r;
                        }

                        if row_set.count_ones() == 4 {
                            let defining_cols = [c1, c2, c3, c4];
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
        }

        eliminations_made
    }

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

            if positions.len() >= 2 && positions.len() <= 4 {
                result.push((unit_idx, positions));
            }
        }

        result
    }
}

impl TechniqueRule for Jellyfish {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut eliminations_made = false;

        for candidate_val in 1..=9 {
            let candidate_bit = 1 << (candidate_val - 1);
            eliminations_made |=
                Self::find_row_based_jellyfish(prop, candidate_bit, path, self.flags());
            eliminations_made |=
                Self::find_column_based_jellyfish(prop, candidate_bit, path, self.flags());
        }

        eliminations_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::JELLYFISH
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_jellyfish_eliminates_from_correct_lines() {
        // Hodoku Jellyfish example
        let s = "200000003080030050003402100001205400000090000009308600002506900090020070400000001";
        let mut rustoku = Rustoku::new_from_str(s).unwrap().with_techniques(
            TechniqueFlags::EASY | TechniqueFlags::MEDIUM | TechniqueFlags::JELLYFISH,
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
                } if flags.contains(TechniqueFlags::JELLYFISH) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "Jellyfish should produce at least one candidate elimination"
        );

        for &(r, c, v) in &eliminations {
            let cand_bit = 1u16 << (v - 1);
            let remaining = rustoku.candidates.get(r, c);
            assert_eq!(
                remaining & cand_bit,
                0,
                "Candidate {v} should be eliminated from ({r},{c}) by Jellyfish"
            );
        }
    }
}
