use crate::core::{SolvePath, TechniqueFlags};

use super::{TechniquePropagator, TechniqueRule, units};

/// Naked triples technique implementation.
///
/// A naked triple occurs when three cells in the same unit (row, column, or box)
/// collectively contain exactly three candidate numbers. Each individual cell may
/// have 2 or 3 of these candidates, but the union of all their candidates is
/// exactly 3 values.
///
/// Since these three cells must contain those three numbers between them, no other
/// cell in the same unit can contain any of those numbers.
///
/// Example:
/// Consider cells in a row with candidates: {1,3}, {1,2}, {2,3}
/// The union is {1,2,3}. These three cells must hold 1, 2, and 3 between them,
/// so 1, 2, and 3 can be eliminated from all other cells in that row.
///
/// The algorithm:
/// 1. Find all cells in a unit that have 2 or 3 candidates
/// 2. For each triple of such cells, compute the union of their candidate masks
/// 3. If the union has exactly 3 bits set, it's a naked triple
/// 4. Eliminate those candidates from all other cells in the unit
pub struct NakedTriples;

impl NakedTriples {
    /// Process a single unit (row, column, or box) for naked triples.
    fn process_unit_for_naked_triples(
        prop: &mut TechniquePropagator,
        unit_cells: &[(usize, usize)],
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        // Find all cells with 2 or 3 candidates
        let mut eligible_cells: Vec<(usize, usize, u16)> = Vec::new();
        for &(r, c) in unit_cells {
            if prop.board.is_empty(r, c) {
                let cand_mask = prop.candidates.get(r, c);
                let count = cand_mask.count_ones();
                if count == 2 || count == 3 {
                    eligible_cells.push((r, c, cand_mask));
                }
            }
        }

        if eligible_cells.len() < 3 {
            return false;
        }

        // Check all triples
        for i in 0..eligible_cells.len() {
            for j in (i + 1)..eligible_cells.len() {
                for k in (j + 1)..eligible_cells.len() {
                    let (r1, c1, mask1) = eligible_cells[i];
                    let (r2, c2, mask2) = eligible_cells[j];
                    let (r3, c3, mask3) = eligible_cells[k];

                    let union_mask = mask1 | mask2 | mask3;

                    // Naked triple requires exactly 3 candidates in the union
                    if union_mask.count_ones() == 3 {
                        let triple_cells = [(r1, c1), (r2, c2), (r3, c3)];

                        // Eliminate from other cells in the unit
                        for &(other_r, other_c) in unit_cells {
                            if triple_cells.contains(&(other_r, other_c)) {
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

        eliminations_made
    }
}

impl TechniqueRule for NakedTriples {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_eliminations_made = false;

        // Process rows
        for i in 0..9 {
            let cells = units::row_cells(i);
            if Self::process_unit_for_naked_triples(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        // Process columns
        for i in 0..9 {
            let cells = units::col_cells(i);
            if Self::process_unit_for_naked_triples(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        // Process 3x3 boxes
        for i in 0..9 {
            let cells = units::box_cells(i);
            if Self::process_unit_for_naked_triples(prop, &cells, path, self.flags()) {
                overall_eliminations_made = true;
            }
        }

        overall_eliminations_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::NAKED_TRIPLES
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_naked_triples_eliminates_candidates() {
        // Hodoku Naked Triples example
        let s = "400500370320000004060000000800002030210840000000000090070090100040651000000070000";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::EASY | TechniqueFlags::NAKED_TRIPLES);
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
                } if flags.contains(TechniqueFlags::NAKED_TRIPLES) => Some((*row, *col, *value)),
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "Naked Triples should produce at least one candidate elimination"
        );

        for &(r, c, v) in &eliminations {
            let cand_bit = 1u16 << (v - 1);
            let remaining = rustoku.candidates.get(r, c);
            assert_eq!(
                remaining & cand_bit,
                0,
                "Candidate {v} should be eliminated from ({r},{c}) by Naked Triples"
            );
        }
    }
}
