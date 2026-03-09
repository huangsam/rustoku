use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule, units};

/// Skyscraper technique implementation.
///
/// A Skyscraper occurs when two rows (or columns) each contain exactly two candidates
/// for a specific digit, and they share exactly one column (or row). The two unshared
/// candidates (the "roof") cannot both be false (because that would force the shared
/// "base" candidates to both be true, which is impossible since they are in the same
/// unit). Therefore, any cell that sees *both* roof cells cannot contain the digit.
pub struct Skyscraper;

impl Skyscraper {
    /// Returns true if two cells can "see" each other (share a row, column, or box).
    fn sees(r1: usize, c1: usize, r2: usize, c2: usize) -> bool {
        r1 == r2 || c1 == c2 || (r1 / 3 == r2 / 3 && c1 / 3 == c2 / 3)
    }

    fn find_row_based_skyscraper(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        // Find rows with exactly 2 candidates
        let eligible_rows = Self::find_eligible_units(prop, candidate_bit, units::UnitType::Row);

        let mut eliminations_made = false;

        for i in 0..eligible_rows.len() {
            for j in (i + 1)..eligible_rows.len() {
                let (r1, ref cols1) = eligible_rows[i];
                let (r2, ref cols2) = eligible_rows[j];

                // For Skyscraper, they must share exactly 1 column (the base)
                // cols1 and cols2 both have length 2.
                let mut shared_cols = Vec::new();
                for &c1 in cols1 {
                    if cols2.contains(&c1) {
                        shared_cols.push(c1);
                    }
                }

                if shared_cols.len() == 1 {
                    let shared_col = shared_cols[0];
                    let roof_col1 = cols1.iter().find(|&&c| c != shared_col).unwrap();
                    let roof_col2 = cols2.iter().find(|&&c| c != shared_col).unwrap();

                    // Eliminate candidate from cells that see BOTH roof cells.
                    for r in 0..9 {
                        for c in 0..9 {
                            if (r == r1 && c == *roof_col1) || (r == r2 && c == *roof_col2) {
                                continue;
                            }
                            if Self::sees(r, c, r1, *roof_col1)
                                && Self::sees(r, c, r2, *roof_col2)
                                && prop.board.is_empty(r, c)
                                && (prop.candidates.get(r, c) & candidate_bit) != 0
                            {
                                eliminations_made |=
                                    prop.eliminate_candidate(r, c, candidate_bit, flags, path);
                            }
                        }
                    }
                }
            }
        }

        eliminations_made
    }

    fn find_col_based_skyscraper(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        let eligible_cols = Self::find_eligible_units(prop, candidate_bit, units::UnitType::Column);

        let mut eliminations_made = false;

        for i in 0..eligible_cols.len() {
            for j in (i + 1)..eligible_cols.len() {
                let (c1, ref rows1) = eligible_cols[i];
                let (c2, ref rows2) = eligible_cols[j];

                let mut shared_rows = Vec::new();
                for &r1 in rows1 {
                    if rows2.contains(&r1) {
                        shared_rows.push(r1);
                    }
                }

                if shared_rows.len() == 1 {
                    let shared_row = shared_rows[0];
                    let roof_row1 = rows1.iter().find(|&&r| r != shared_row).unwrap();
                    let roof_row2 = rows2.iter().find(|&&r| r != shared_row).unwrap();

                    for r in 0..9 {
                        for c in 0..9 {
                            if (r == *roof_row1 && c == c1) || (r == *roof_row2 && c == c2) {
                                continue;
                            }
                            if Self::sees(r, c, *roof_row1, c1)
                                && Self::sees(r, c, *roof_row2, c2)
                                && prop.board.is_empty(r, c)
                                && (prop.candidates.get(r, c) & candidate_bit) != 0
                            {
                                eliminations_made |=
                                    prop.eliminate_candidate(r, c, candidate_bit, flags, path);
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

            if positions.len() == 2 {
                result.push((unit_idx, positions));
            }
        }

        result
    }
}

impl TechniqueRule for Skyscraper {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut eliminations_made = false;

        for candidate_val in 1..=9 {
            let candidate_bit = 1 << (candidate_val - 1);
            eliminations_made |=
                Self::find_row_based_skyscraper(prop, candidate_bit, path, self.flags());
            eliminations_made |=
                Self::find_col_based_skyscraper(prop, candidate_bit, path, self.flags());
        }

        eliminations_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::SKYSCRAPER
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_skyscraper_eliminates_from_correct_lines() {
        // Hodoku Skyscraper example
        // https://hodoku.sourceforge.net/en/show_example.php?file=sk01&tech=Skyscraper
        let s = "000000000001902060000006790902000600370000950005000004140003005709024000000800000";
        let mut rustoku = Rustoku::new_from_str(s).unwrap().with_techniques(
            TechniqueFlags::EASY | TechniqueFlags::MEDIUM | TechniqueFlags::SKYSCRAPER,
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
                } if flags.contains(TechniqueFlags::SKYSCRAPER) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "Skyscraper should produce at least one candidate elimination"
        );

        for &(r, c, v) in &eliminations {
            let cand_bit = 1u16 << (v - 1);
            let remaining = rustoku.candidates.get(r, c);
            assert_eq!(
                remaining & cand_bit,
                0,
                "Candidate {v} should be eliminated from ({r},{c}) by Skyscraper"
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
