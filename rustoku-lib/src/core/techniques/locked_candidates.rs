use crate::core::{SolvePath, TechniqueFlags};

use super::{TechniquePropagator, TechniqueRule};

/// Locked candidates technique implementation.
///
/// This technique identifies "locked candidates" - situations where a candidate
/// value is confined to a specific region within a unit (row, column, or box).
/// There are two main types of locked candidates:
///
/// 1. **Pointing Pairs/Triples**: When all candidates for a number in a box
///    are confined to a single row or column within that box, the candidate
///    can be eliminated from that row/column outside the box.
///
/// 2. **Box/Line Reduction**: When all candidates for a number in a row or column
///    are confined to a single box, the candidate can be eliminated from the
///    rest of that box.
///
/// Example of pointing pair:
/// If in box 1, candidate 5 only appears in row 1, columns 1-2, then 5 can be
/// eliminated from row 1, columns 4-9 (outside the box).
///
/// This technique is also known as "Pointing Pairs/Triples" and "Box/Line Reduction".
pub struct LockedCandidates;

impl LockedCandidates {
    /// Processes pointing pairs/triples for a specific row.
    ///
    /// For each candidate (1-9), checks if all occurrences of that candidate
    /// in the given row are confined to a single 3x3 box. If so, eliminates
    /// that candidate from the rest of the box (outside this row).
    ///
    /// This is the "pointing pair/triple" elimination for rows.
    fn process_row_for_locked_candidates(
        prop: &mut TechniquePropagator,
        row: usize,
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            // Track which boxes in this row contain this candidate
            // box_mask uses bits 0-8 to represent boxes 0-8
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

            // If candidate appears in exactly one box within this row,
            // eliminate it from other cells in that box (different rows)
            if found_any && box_mask.count_ones() == 1 {
                let box_idx = box_mask.trailing_zeros() as usize;
                let start_row = (box_idx / 3) * 3;
                let start_col = (box_idx % 3) * 3;

                // Eliminate candidate from other rows in the same box
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

    /// Processes pointing pairs/triples for a specific column.
    ///
    /// For each candidate (1-9), checks if all occurrences of that candidate
    /// in the given column are confined to a single 3x3 box. If so, eliminates
    /// that candidate from the rest of the box (outside this column).
    ///
    /// This is the "pointing pair/triple" elimination for columns.
    fn process_col_for_locked_candidates(
        prop: &mut TechniquePropagator,
        col: usize,
        path: &mut SolvePath,
        flags: TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            // Track which boxes in this column contain this candidate
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

            // If candidate appears in exactly one box within this column,
            // eliminate it from other cells in that box (different columns)
            if found_any && box_mask.count_ones() == 1 {
                let box_idx = box_mask.trailing_zeros() as usize;
                let start_row = (box_idx / 3) * 3;
                let start_col = (box_idx % 3) * 3;

                // Eliminate candidate from other columns in the same box
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

    /// Processes box/line reduction for a specific 3x3 box.
    ///
    /// For each candidate (1-9), checks if all occurrences of that candidate
    /// in the given box are confined to a single row or column. If so, eliminates
    /// that candidate from the rest of the row/column (outside this box).
    ///
    /// This is the "box/line reduction" or "claiming" elimination.
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

            // Track which rows and columns in this box contain this candidate
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

            // If all candidates in this box are in the same row,
            // eliminate from other cells in that row (outside this box)
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

            // If all candidates in this box are in the same column,
            // eliminate from other cells in that column (outside this box)
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
    /// Applies the locked candidates technique by checking all rows, columns, and boxes
    /// for pointing pairs/triples and box/line reductions.
    ///
    /// Returns true if any candidate eliminations were made.
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
