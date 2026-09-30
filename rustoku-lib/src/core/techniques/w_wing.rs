use super::TechniqueFlags;
use super::{TechniquePropagator, TechniqueRule, units};
use crate::core::SolvePath;

/// W-Wing technique implementation.
///
/// A W-Wing connects two identical bivalue cells `{X, Y}` via a strong link (conjugate pair)
/// on one of the candidates (say, `X`).
///
/// If a strong link on `X` exists in some unit (row, column, or box) between cells `(s1, s2)`,
/// where one bivalue cell sees `s1` and the other sees `s2`:
/// - If `s1` is `X`, the first bivalue cell cannot be `X` and must be `Y`.
/// - If `s2` is `X`, the second bivalue cell cannot be `X` and must be `Y`.
///
/// Since either `s1` or `s2` must be `X`, at least one of the two bivalue cells must be `Y`.
/// Therefore, `Y` can be eliminated from any cell that sees both bivalue cells.
///
/// ### Example
/// If bivalue cell 1 at `(0, 1)` has `{3, 8}` and bivalue cell 2 at `(8, 7)` has `{3, 8}`,
/// and a strong link for candidate `3` connects `(0, 5)` and `(8, 5)` in column 5:
/// - `(0, 1)` sees `(0, 5)`
/// - `(8, 7)` sees `(8, 5)`
///
/// Then candidate `8` can be eliminated from any cell that sees both `(0, 1)` and `(8, 7)` (such as `(0, 7)` or `(8, 1)`).
///
/// See: <https://hodoku.sourceforge.net/en/tech_wings.php#w>
pub struct WWing;

impl WWing {
    fn find_w_wings(
        prop: &mut TechniquePropagator,
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        // Step 1: Collect all bivalue cells
        let bivalue_cells = Self::get_bivalue_cells(prop);
        if bivalue_cells.len() < 2 {
            return false;
        }

        // Step 2: Iterate through pairs of identical bivalue cells {X, Y}
        for (i, &(r1, c1, mask1)) in bivalue_cells.iter().enumerate() {
            for &(r2, c2, mask2) in bivalue_cells.iter().skip(i + 1) {
                if mask1 != mask2 {
                    continue;
                }

                // If they see each other directly, it's a Naked Pair (handled elsewhere)
                if r1 == r2 || c1 == c2 || (r1 / 3 == r2 / 3 && c1 / 3 == c2 / 3) {
                    continue;
                }

                // Extract candidates X and Y
                let candidates: Vec<u8> =
                    (1..=9).filter(|&v| (mask1 & (1 << (v - 1))) != 0).collect();

                let x_val = candidates[0];
                let y_val = candidates[1];

                // Step 3: Test each candidate as the strong link "bridge"
                eliminations_made |=
                    Self::check_pincer_pair(prop, (r1, c1), (r2, c2), x_val, y_val, flags, path);
                eliminations_made |=
                    Self::check_pincer_pair(prop, (r1, c1), (r2, c2), y_val, x_val, flags, path);
            }
        }

        eliminations_made
    }

    /// Checks if a pair of pincers {X, Y} are connected by a strong link on `bridge_val`.
    /// If so, eliminates `other_val` from common peers seeing both pincers.
    fn check_pincer_pair(
        prop: &mut TechniquePropagator,
        p1: (usize, usize),
        p2: (usize, usize),
        bridge_val: u8,
        other_val: u8,
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let bridge_bit = 1 << (bridge_val - 1);
        let other_bit = 1 << (other_val - 1);

        // Search all units (rows, columns, boxes) for a strong link on bridge_val
        for unit_idx in 0..9 {
            // Rows
            if let Some(elim) = Self::check_unit_strong_link(
                prop,
                p1,
                p2,
                &units::row_cells(unit_idx),
                bridge_bit,
                other_bit,
                flags,
                path,
            ) && elim
            {
                return true;
            }
            // Columns
            if let Some(elim) = Self::check_unit_strong_link(
                prop,
                p1,
                p2,
                &units::col_cells(unit_idx),
                bridge_bit,
                other_bit,
                flags,
                path,
            ) && elim
            {
                return true;
            }
            // Boxes
            if let Some(elim) = Self::check_unit_strong_link(
                prop,
                p1,
                p2,
                &units::box_cells(unit_idx),
                bridge_bit,
                other_bit,
                flags,
                path,
            ) && elim
            {
                return true;
            }
        }

        false
    }

    #[allow(clippy::too_many_arguments)]
    fn check_unit_strong_link(
        prop: &mut TechniquePropagator,
        p1: (usize, usize),
        p2: (usize, usize),
        unit_cells: &[(usize, usize)],
        bridge_bit: u16,
        other_bit: u16,
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> Option<bool> {
        let positions: Vec<(usize, usize)> = unit_cells
            .iter()
            .filter(|&&(r, c)| {
                prop.board.is_empty(r, c) && (prop.candidates.get(r, c) & bridge_bit) != 0
            })
            .cloned()
            .collect();

        // Strong link: candidate appears in exactly 2 cells within this unit
        if positions.len() == 2 {
            let s1 = positions[0];
            let s2 = positions[1];

            // Verify that one pincer sees s1 and the other sees s2
            let match_v1 = (Self::sees(p1, s1) && Self::sees(p2, s2))
                || (Self::sees(p1, s2) && Self::sees(p2, s1));

            if match_v1 {
                // Ensure the bridge cells are not the pincers themselves
                if s1 == p1 || s1 == p2 || s2 == p1 || s2 == p2 {
                    return None;
                }

                // Eliminate other_bit from common peers of p1 and p2
                let elim = Self::eliminate_from_common_peers(prop, p1, p2, other_bit, flags, path);
                if elim {
                    return Some(true);
                }
            }
        }
        None
    }

    /// Returns true if two cells see each other (share a row, column, or 3x3 box).
    fn sees(c1: (usize, usize), c2: (usize, usize)) -> bool {
        c1.0 == c2.0 || c1.1 == c2.1 || (c1.0 / 3 == c2.0 / 3 && c1.1 / 3 == c2.1 / 3)
    }

    /// Eliminates `val_bit` from any empty cell that sees both pincer cells `p1` and `p2`.
    fn eliminate_from_common_peers(
        prop: &mut TechniquePropagator,
        p1: (usize, usize),
        p2: (usize, usize),
        val_bit: u16,
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let mut eliminations_made = false;
        for r in 0..9 {
            for c in 0..9 {
                if prop.board.is_empty(r, c)
                    && (prop.candidates.get(r, c) & val_bit) != 0
                    && Self::sees((r, c), p1)
                    && Self::sees((r, c), p2)
                    && (r, c) != p1
                    && (r, c) != p2
                {
                    eliminations_made |= prop.eliminate_candidate(r, c, val_bit, flags, path);
                }
            }
        }
        eliminations_made
    }

    /// Collects all empty cells that have exactly 2 candidates.
    fn get_bivalue_cells(prop: &TechniquePropagator) -> Vec<(usize, usize, u16)> {
        let mut result = Vec::new();
        for r in 0..9 {
            for c in 0..9 {
                if prop.board.is_empty(r, c) {
                    let mask = prop.candidates.get(r, c);
                    if mask.count_ones() == 2 {
                        result.push((r, c, mask));
                    }
                }
            }
        }
        result
    }
}

impl TechniqueRule for WWing {
    /// Applies the W-Wing technique across all candidate pairs.
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        Self::find_w_wings(prop, path, self.flags())
    }

    fn flags(&self) -> TechniqueFlags {
        TechniqueFlags::W_WING
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_w_wing_eliminates_candidates() {
        // Hodoku W-Wing example
        // https://hodoku.sourceforge.net/en/show_example.php?file=w101&tech=W-Wing
        let s = "025100000000009030400708900040000800150400000000060004000000008263040000080390106";
        let mut rustoku = Rustoku::new_from_str(s).unwrap().with_techniques(
            TechniqueFlags::EASY | TechniqueFlags::MEDIUM | TechniqueFlags::W_WING,
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
                } if flags.contains(TechniqueFlags::W_WING) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "W-Wing should produce at least one candidate elimination"
        );

        for &(r, c, v) in &eliminations {
            let cand_bit = 1u16 << (v - 1);
            let remaining = rustoku.candidates.get(r, c);
            assert_eq!(
                remaining & cand_bit,
                0,
                "Candidate {v} should be eliminated from ({r},{c}) by W-Wing"
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
