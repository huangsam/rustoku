use crate::core::{SolvePath, TechniqueFlags};

use super::{TechniquePropagator, TechniqueRule, units};

/// Naked quads technique implementation.
///
/// A naked quad occurs when four cells in the same unit (row, column, or box)
/// collectively contain exactly four candidate numbers. Each individual cell may
/// have 2, 3, or 4 of these candidates, but the union of all their candidates is
/// exactly 4 values.
///
/// Since these four cells must contain those four numbers between them, no other
/// cell in the same unit can contain any of those numbers.
///
/// The algorithm:
/// 1. Find all cells in a unit that have 2, 3, or 4 candidates
/// 2. For each quad of such cells, compute the union of their candidate masks
/// 3. If the union has exactly 4 bits set, it's a naked quad
/// 4. Eliminate those candidates from all other cells in the unit
pub struct NakedQuads;

impl NakedQuads {
    /// Process a single unit (row, column, or box) for naked quads.
    fn process_unit_for_naked_quads(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        // Find all cells with 2, 3, or 4 candidates
        let mut eligible_cells: Vec<(usize, usize, u16)> = Vec::new();
        for &(r, c) in unit_cells {
            if prop.board.is_empty(r, c) {
                let cand_mask = prop.candidates.get(r, c);
                let count = cand_mask.count_ones();
                if (2..=4).contains(&count) {
                    eligible_cells.push((r, c, cand_mask));
                }
            }
        }

        if eligible_cells.len() < 4 {
            return false;
        }

        // Check all quads
        for i in 0..eligible_cells.len() {
            for j in (i + 1)..eligible_cells.len() {
                for k in (j + 1)..eligible_cells.len() {
                    for l in (k + 1)..eligible_cells.len() {
                        let (r1, c1, mask1) = eligible_cells[i];
                        let (r2, c2, mask2) = eligible_cells[j];
                        let (r3, c3, mask3) = eligible_cells[k];
                        let (r4, c4, mask4) = eligible_cells[l];

                        let union_mask = mask1 | mask2 | mask3 | mask4;

                        // Naked quad requires exactly 4 candidates in the union
                        if union_mask.count_ones() == 4 {
                            let quad_cells = [(r1, c1), (r2, c2), (r3, c3), (r4, c4)];

                            // Eliminate from other cells in the unit
                            for &(other_r, other_c) in unit_cells {
                                if quad_cells.contains(&(other_r, other_c)) {
                                    continue;
                                }

                                if prop.board.is_empty(other_r, other_c) {
                                    let initial_mask = prop.candidates.get(other_r, other_c);
                                    if (initial_mask & union_mask) != 0 {
                                        eliminations_made |= prop.eliminate_multiple_candidates(
                                            other_r, other_c, union_mask, flags, path,
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
}

impl TechniqueRule for NakedQuads {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_eliminations_made = false;

        // Process rows
        for i in 0..9 {
            let cells = units::row_cells(i);
            if Self::process_unit_for_naked_quads(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        // Process columns
        for i in 0..9 {
            let cells = units::col_cells(i);
            if Self::process_unit_for_naked_quads(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        // Process 3x3 boxes
        for i in 0..9 {
            let cells = units::box_cells(i);
            if Self::process_unit_for_naked_quads(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        overall_eliminations_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::NAKED_QUADS
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolveStep, TechniqueFlags};

    // Naked quad example
    #[test]
    fn test_naked_quads() {
        let mut sudoku = Rustoku::new_from_str(
            ".......6.....3..47.325.....6....7..52.7.1.9.8.81..4........2...........1..587....",
        )
        .unwrap();
        // Combine with basic subsets (Naked Singles etc.) to help setup the board so the quad is actionable if needed
        sudoku = sudoku.with_techniques(TechniqueFlags::NAKED_QUADS | TechniqueFlags::EASY);
        let sol = sudoku.solve_any().unwrap();

        let has_naked_quad = sol.solve_path.steps.iter().any(|step| match step {
            SolveStep::CandidateElimination { flags, .. } => {
                flags.contains(TechniqueFlags::NAKED_QUADS)
            }
            _ => false,
        });

        assert!(
            has_naked_quad,
            "Expected NAKED_QUADS technique to be used in solution path"
        );
    }
}
