use crate::core::{SolvePath, TechniqueFlags};

use super::{TechniquePropagator, TechniqueRule};

/// Locked candidates technique implementation.
pub struct LockedCandidates;

impl LockedCandidates {
    // Helper function for Locked Candidates (row), private to this impl block
    fn process_row_for_locked_candidates(
        prop: &mut TechniquePropagator,
        row: usize,
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            // Use a u16 bitmask to track which boxes contain this candidate in this row
            let mut box_mask: u16 = 0;
            let mut found_any = false;

            for col in 0..9 {
                if prop.board.is_empty(row, col)
                    && (prop.candidates.get(row, col) & candidate_bit) != 0
                {
                    let box_idx = (row / 3) * 3 + (col / 3);
                    box_mask |= 1 << box_idx;
                    found_any = true;
                }
            }

            // All candidate cells are in exactly one box
            if found_any && box_mask.count_ones() == 1 {
                let box_idx = box_mask.trailing_zeros() as usize;
                let start_row = (box_idx / 3) * 3;
                let start_col = (box_idx % 3) * 3;

                for r in start_row..(start_row + 3) {
                    for c in start_col..(start_col + 3) {
                        if r != row && prop.board.is_empty(r, c) {
                            let initial_mask = prop.candidates.get(r, c);
                            if (initial_mask & candidate_bit) != 0 {
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

    // Helper function for Locked Candidates (column), private to this impl block
    fn process_col_for_locked_candidates(
        prop: &mut TechniquePropagator,
        col: usize,
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            // Use a u16 bitmask to track which boxes contain this candidate in this column
            let mut box_mask: u16 = 0;
            let mut found_any = false;

            for row in 0..9 {
                if prop.board.is_empty(row, col)
                    && (prop.candidates.get(row, col) & candidate_bit) != 0
                {
                    let box_idx = (row / 3) * 3 + (col / 3);
                    box_mask |= 1 << box_idx;
                    found_any = true;
                }
            }

            // All candidate cells are in exactly one box
            if found_any && box_mask.count_ones() == 1 {
                let box_idx = box_mask.trailing_zeros() as usize;
                let start_row = (box_idx / 3) * 3;
                let start_col = (box_idx % 3) * 3;

                for r in start_row..(start_row + 3) {
                    for c in start_col..(start_col + 3) {
                        if c != col && prop.board.is_empty(r, c) {
                            let initial_mask = prop.candidates.get(r, c);
                            if (initial_mask & candidate_bit) != 0 {
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

    // Helper function for Locked Candidates (box), private to this impl block
    fn process_box_for_locked_candidates(
        prop: &mut TechniquePropagator,
        box_idx: usize,
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;
        let start_row = (box_idx / 3) * 3;
        let start_col = (box_idx % 3) * 3;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            // Use bitmasks to track which rows and columns contain this candidate in this box
            let mut row_mask: u16 = 0;
            let mut col_mask: u16 = 0;
            let mut found_any = false;

            for r_offset in 0..3 {
                for c_offset in 0..3 {
                    let r = start_row + r_offset;
                    let c = start_col + c_offset;
                    if prop.board.is_empty(r, c) && (prop.candidates.get(r, c) & candidate_bit) != 0
                    {
                        row_mask |= 1 << r;
                        col_mask |= 1 << c;
                        found_any = true;
                    }
                }
            }

            if !found_any {
                continue;
            }

            // Check if all candidates are in the same row
            if row_mask.count_ones() == 1 {
                let row = row_mask.trailing_zeros() as usize;

                for c in 0..9 {
                    if (c < start_col || c >= start_col + 3) && prop.board.is_empty(row, c) {
                        let initial_mask = prop.candidates.get(row, c);
                        if (initial_mask & candidate_bit) != 0 {
                            eliminations_made |=
                                prop.eliminate_candidate(row, c, candidate_bit, flags, path);
                        }
                    }
                }
            }

            // Check if all candidates are in the same column
            if col_mask.count_ones() == 1 {
                let col = col_mask.trailing_zeros() as usize;

                for r in 0..9 {
                    if (r < start_row || r >= start_row + 3) && prop.board.is_empty(r, col) {
                        let initial_mask = prop.candidates.get(r, col);
                        if (initial_mask & candidate_bit) != 0 {
                            eliminations_made |=
                                prop.eliminate_candidate(r, col, candidate_bit, flags, path);
                        }
                    }
                }
            }
        }
        eliminations_made
    }
}

impl TechniqueRule for LockedCandidates {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut overall_eliminations_made = false;

        // Check rows for pointing pairs/triples
        for row in 0..9 {
            overall_eliminations_made |=
                Self::process_row_for_locked_candidates(prop, row, path, self.flags());
        }

        // Check columns for pointing pairs/triples
        for col in 0..9 {
            overall_eliminations_made |=
                Self::process_col_for_locked_candidates(prop, col, path, self.flags());
        }

        // Check boxes for box/line reduction
        for box_idx in 0..9 {
            overall_eliminations_made |=
                Self::process_box_for_locked_candidates(prop, box_idx, path, self.flags());
        }

        overall_eliminations_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::LOCKED_CANDIDATES
    }
}
