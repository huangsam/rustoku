use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule};

/// XYZ-Wing technique implementation.
///
/// An XYZ-Wing is an extension of the XY-Wing that involves three cells:
/// - A **pivot** cell with exactly 3 candidates {X, Y, Z}.
/// - A **pincer** cell that sees the pivot and has 2 candidates {X, Z}.
/// - Another **pincer** cell that sees the pivot and has 2 candidates {Y, Z}.
///
/// The pivot cell must be either X, Y, or Z:
/// - If pivot = X, the first pincer ({X, Z}) must be Z (if it also sees the Z-elimination target).
/// - If pivot = Y, the second pincer ({Y, Z}) must be Z (if it also sees the Z-elimination target).
/// - If pivot = Z, the pivot itself is Z.
///
/// In all three cases, at least one of these three cells must be Z. Therefore,
/// any cell that "sees" all three cells (the pivot and both pincers) can have
/// candidate Z eliminated.
///
/// Note that for a cell to see all three, the pivot and pincers must be arranged
/// such that they share a common intersection (usually the pivot and one pincer
/// in a house, and the pivot and the other pincer in a different house, with the
/// target cell seeing all three).
pub struct XyzWing;

impl XyzWing {
    /// Returns true if two cells see each other (share a row, column, or box).
    fn cells_see_each_other(r1: usize, c1: usize, r2: usize, c2: usize) -> bool {
        r1 == r2 || c1 == c2 || (r1 / 3 == r2 / 3 && c1 / 3 == c2 / 3)
    }

    /// Finds all XYZ-Wing patterns and eliminates candidates.
    fn find_xyz_wings(
        prop: &mut TechniquePropagator,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        let mut trivalue = Vec::new();
        let mut bivalue = Vec::new();

        for r in 0..9 {
            for c in 0..9 {
                if prop.board.is_empty(r, c) {
                    let mask = prop.candidates.get(r, c);
                    let count = mask.count_ones();
                    if count == 3 {
                        trivalue.push((r, c, mask));
                    } else if count == 2 {
                        bivalue.push((r, c, mask));
                    }
                }
            }
        }

        let mut eliminations_made = false;

        for &(pr, pc, pmask) in &trivalue {
            let mut pincers = Vec::new();
            for &(br, bc, bmask) in &bivalue {
                if !Self::cells_see_each_other(pr, pc, br, bc) {
                    continue;
                }
                if (bmask & !pmask) == 0 {
                    pincers.push((br, bc, bmask));
                }
            }

            for i in 0..pincers.len() {
                for j in (i + 1)..pincers.len() {
                    let (p1r, p1c, p1mask) = pincers[i];
                    let (p2r, p2c, p2mask) = pincers[j];

                    if (p1mask | p2mask) != pmask {
                        continue;
                    }

                    let shared = p1mask & p2mask;
                    if shared.count_ones() != 1 {
                        continue;
                    }
                    let z_bit = shared;

                    eliminations_made |= Self::eliminate_z_from_common_peers(
                        prop,
                        (pr, pc),
                        (p1r, p1c),
                        (p2r, p2c),
                        z_bit,
                        flags,
                        path,
                    );
                }
            }
        }

        eliminations_made
    }

    fn eliminate_z_from_common_peers(
        prop: &mut TechniquePropagator,
        pivot: (usize, usize),
        p1: (usize, usize),
        p2: (usize, usize),
        z_bit: u16,
        flags: crate::core::TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let mut eliminations_made = false;

        for r in 0..9 {
            for c in 0..9 {
                if (r == pivot.0 && c == pivot.1)
                    || (r == p1.0 && c == p1.1)
                    || (r == p2.0 && c == p2.1)
                {
                    continue;
                }

                if prop.board.is_empty(r, c)
                    && (prop.candidates.get(r, c) & z_bit) != 0
                    && Self::cells_see_each_other(r, c, pivot.0, pivot.1)
                    && Self::cells_see_each_other(r, c, p1.0, p1.1)
                    && Self::cells_see_each_other(r, c, p2.0, p2.1)
                {
                    eliminations_made |= prop.eliminate_candidate(r, c, z_bit, flags, path);
                }
            }
        }

        eliminations_made
    }
}

impl TechniqueRule for XyzWing {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        Self::find_xyz_wings(prop, path, self.flags())
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::XYZ_WING
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_xyz_wing_eliminates_z_from_peers() {
        // Hodoku XYZ-Wing example
        // https://hodoku.sourceforge.net/en/show_example.php?file=z101&tech=XYZ-Wing
        let s = "069000000000021000000800400001530080007600050000000100000000003902080010000340205";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::EASY | TechniqueFlags::XYZ_WING);
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
                } if flags.contains(TechniqueFlags::XYZ_WING) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "XYZ-Wing should produce at least one candidate elimination"
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
