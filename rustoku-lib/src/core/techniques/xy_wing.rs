use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule};

/// XY-Wing technique implementation.
///
/// An XY-Wing involves three cells:
/// - A **pivot** cell with exactly 2 candidates {X, Y}
/// - A **wing** cell that sees the pivot, with exactly 2 candidates {X, Z}
/// - Another **wing** cell that sees the pivot, with exactly 2 candidates {Y, Z}
///
/// Since the pivot must be either X or Y:
/// - If pivot = X, then the {X,Z} wing must be Z
/// - If pivot = Y, then the {Y,Z} wing must be Z
///
/// Either way, one of the two wings must be Z. Therefore, any cell that sees
/// both wings can have candidate Z eliminated.
///
/// "Sees" means sharing a row, column, or box.
pub struct XYWing;

impl XYWing {
    /// Returns true if two cells see each other (share a row, column, or box).
    fn cells_see_each_other(r1: usize, c1: usize, r2: usize, c2: usize) -> bool {
        r1 == r2 || c1 == c2 || (r1 / 3 == r2 / 3 && c1 / 3 == c2 / 3)
    }

    /// Gets all empty cells with exactly 2 candidates.
    fn bivalue_cells(prop: &TechniquePropagator) -> Vec<(usize, usize, u16)> {
        let mut cells = Vec::new();
        for r in 0..9 {
            for c in 0..9 {
                if prop.board.is_empty(r, c) {
                    let mask = prop.candidates.get(r, c);
                    if mask.count_ones() == 2 {
                        cells.push((r, c, mask));
                    }
                }
            }
        }
        cells
    }

    /// Finds all XY-Wing patterns and eliminates candidates.
    fn find_xy_wings(
        prop: &mut TechniquePropagator,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        // Snapshot all bivalue cells up front so we don't conflict with mutable prop
        let bivalue = Self::bivalue_cells(prop);
        let mut eliminations_made = false;

        for pivot_idx in 0..bivalue.len() {
            let (pr, pc, pivot_mask) = bivalue[pivot_idx];

            // Extract the two candidates of the pivot
            let x = pivot_mask & pivot_mask.wrapping_neg(); // lowest bit
            let y = pivot_mask ^ x; // the other bit

            // Find wing candidates among bivalue cells that see the pivot
            let mut x_wings: Vec<(usize, usize, u16)> = Vec::new(); // cells with {X, Z}
            let mut y_wings: Vec<(usize, usize, u16)> = Vec::new(); // cells with {Y, Z}

            for &(wr, wc, wmask) in &bivalue {
                if wr == pr && wc == pc {
                    continue;
                }
                if !Self::cells_see_each_other(pr, pc, wr, wc) {
                    continue;
                }

                // Wing must share exactly one candidate with the pivot
                let shared = wmask & pivot_mask;
                if shared.count_ones() != 1 {
                    continue;
                }

                if shared == x {
                    x_wings.push((wr, wc, wmask));
                } else {
                    y_wings.push((wr, wc, wmask));
                }
            }

            // Try all pairs of (x_wing, y_wing)
            for &(xr, xc, xmask) in &x_wings {
                for &(yr, yc, ymask) in &y_wings {
                    // The wings should not be the same cell
                    if xr == yr && xc == yc {
                        continue;
                    }

                    // Z is the non-pivot candidate in each wing; both wings must agree on Z
                    let z_from_x = xmask & !x; // should be Z
                    let z_from_y = ymask & !y; // should be Z
                    if z_from_x != z_from_y {
                        continue;
                    }
                    let z_bit = z_from_x;

                    // Eliminate Z from all cells that see both wings
                    eliminations_made |= Self::eliminate_z_from_common_peers(
                        prop,
                        (xr, xc),
                        (yr, yc),
                        z_bit,
                        flags,
                        path,
                    );
                }
            }
        }

        eliminations_made
    }

    /// Eliminates candidate Z from all cells that see both wing cells.
    fn eliminate_z_from_common_peers(
        prop: &mut TechniquePropagator,
        wing1: (usize, usize),
        wing2: (usize, usize),
        z_bit: u16,
        flags: crate::core::TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let mut eliminations_made = false;
        let (w1r, w1c) = wing1;
        let (w2r, w2c) = wing2;

        // Collect peers of both wings by iterating all cells
        for r in 0..9 {
            for c in 0..9 {
                // Skip the wing cells themselves
                if (r == w1r && c == w1c) || (r == w2r && c == w2c) {
                    continue;
                }

                if prop.board.is_empty(r, c)
                    && (prop.candidates.get(r, c) & z_bit) != 0
                    && Self::cells_see_each_other(r, c, w1r, w1c)
                    && Self::cells_see_each_other(r, c, w2r, w2c)
                {
                    eliminations_made |= prop.eliminate_candidate(r, c, z_bit, flags, path);
                }
            }
        }

        eliminations_made
    }

    /// Gets all peers (cells that share a row, column, or box) of a given cell.
    #[allow(dead_code)]
    fn peers_of(r: usize, c: usize) -> Vec<(usize, usize)> {
        let mut peers = Vec::new();
        // Same row
        for col in 0..9 {
            if col != c {
                peers.push((r, col));
            }
        }
        // Same column
        for row in 0..9 {
            if row != r {
                peers.push((row, c));
            }
        }
        // Same box (avoid duplicates already in row/col)
        let br = (r / 3) * 3;
        let bc = (c / 3) * 3;
        for row in br..br + 3 {
            for col in bc..bc + 3 {
                if row != r && col != c {
                    peers.push((row, col));
                }
            }
        }
        peers
    }
}

impl TechniqueRule for XYWing {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        Self::find_xy_wings(prop, path, self.flags())
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::XY_WING
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_xy_wing_eliminates_z_from_peers() {
        // Hodoku XY-Wing example
        let s = "000060000000010863003009000904000000300000704570820000000006580690007000000040030";
        let mut rustoku = Rustoku::new_from_str(s).unwrap().with_techniques(
            TechniqueFlags::EASY | TechniqueFlags::MEDIUM | TechniqueFlags::XY_WING,
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
                } if flags.contains(TechniqueFlags::XY_WING) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "XY-Wing should produce at least one candidate elimination"
        );

        for &(r, c, v) in &eliminations {
            let cand_bit = 1u16 << (v - 1);
            let remaining = rustoku.candidates.get(r, c);
            assert_eq!(
                remaining & cand_bit,
                0,
                "Candidate {v} should be eliminated from ({r},{c}) by XY-Wing"
            );
        }
    }
}
