use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule};

/// X-Wing technique implementation.
///
/// An X-Wing is a pattern where a candidate appears in exactly two cells in each of
/// two different rows (or columns), and these cells form a rectangle. This creates
/// a situation where the candidate must be in one of the two cells in each row,
/// allowing elimination of that candidate from other cells in the same columns.
///
/// Example of row-based X-Wing:
/// If candidate 5 appears only in columns 2 and 7 of row 1, and only in columns 2 and 7 of row 4,
/// then 5 can be eliminated from columns 2 and 7 in all other rows.
///
/// The technique works because in a valid solution, each number must appear exactly once
/// in each row, column, and box. The X-Wing pattern forces the candidate to be placed
/// in specific positions, eliminating it from other possibilities in those columns.
///
/// This technique can be applied to both rows (row-based X-Wing) and columns (column-based X-Wing).
pub struct XWing;

impl TechniqueRule for XWing {
    /// Applies the X-Wing technique by checking for both row-based and column-based X-Wings
    /// for each candidate value (1-9).
    ///
    /// Returns true if any candidate eliminations were made.
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut eliminations_made = false;

        for candidate_val in 1..=9 {
            let candidate_bit = 1 << (candidate_val - 1);

            // Find rows that have exactly 2 cells containing this candidate
            // These could potentially form an X-Wing pattern
            let mut rows_with_two_candidates: Vec<usize> = Vec::new();
            let mut candidate_cols_in_rows: Vec<Vec<usize>> = Vec::new();

            for r in 0..9 {
                let mut cols_for_candidate_in_row: Vec<usize> = Vec::new();
                for c in 0..9 {
                    if prop.board.is_empty(r, c) && (prop.candidates.get(r, c) & candidate_bit) != 0
                    {
                        cols_for_candidate_in_row.push(c);
                    }
                }
                if cols_for_candidate_in_row.len() == 2 {
                    rows_with_two_candidates.push(r);
                    candidate_cols_in_rows.push(cols_for_candidate_in_row);
                }
            }

            // Check pairs of rows to see if they form an X-Wing
            // Two rows form an X-Wing if they have candidates in exactly the same two columns
            for i in 0..rows_with_two_candidates.len() {
                for j in (i + 1)..rows_with_two_candidates.len() {
                    let r1 = rows_with_two_candidates[i];
                    let r2 = rows_with_two_candidates[j];
                    let cols1 = &candidate_cols_in_rows[i];
                    let cols2 = &candidate_cols_in_rows[j];

                    if cols1[0] == cols2[0] && cols1[1] == cols2[1] {
                        let c1 = cols1[0];
                        let c2 = cols1[1];

                        // Found an X-Wing! The candidate must be in either (r1,c1) or (r1,c2) for row r1,
                        // and either (r2,c1) or (r2,c2) for row r2. Therefore, it cannot appear
                        // anywhere else in columns c1 and c2.
                        // Remove candidate from other cells in column c1 (excluding r1, r2)
                        for r_other in 0..9 {
                            if r_other != r1 && r_other != r2 && prop.board.is_empty(r_other, c1) {
                                let initial_mask = prop.candidates.get(r_other, c1);
                                if (initial_mask & candidate_bit) != 0 {
                                    eliminations_made |= prop.eliminate_candidate(
                                        r_other,
                                        c1,
                                        candidate_bit,
                                        self.flags(),
                                        path,
                                    );
                                }
                            }
                        }

                        // Remove candidate from other cells in column c2 (excluding r1, r2)
                        for r_other in 0..9 {
                            if r_other != r1 && r_other != r2 && prop.board.is_empty(r_other, c2) {
                                let initial_mask = prop.candidates.get(r_other, c2);
                                if (initial_mask & candidate_bit) != 0 {
                                    eliminations_made |= prop.eliminate_candidate(
                                        r_other,
                                        c2,
                                        candidate_bit,
                                        self.flags(),
                                        path,
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // Check for column-based X-Wings (symmetric to row-based)
            // Find columns that have exactly 2 cells containing this candidate
            let mut cols_with_two_candidates: Vec<usize> = Vec::new();
            let mut candidate_rows_in_cols: Vec<Vec<usize>> = Vec::new();

            for c in 0..9 {
                let mut rows_for_candidate_in_col: Vec<usize> = Vec::new();
                for r in 0..9 {
                    if prop.board.is_empty(r, c) && (prop.candidates.get(r, c) & candidate_bit) != 0
                    {
                        rows_for_candidate_in_col.push(r);
                    }
                }
                if rows_for_candidate_in_col.len() == 2 {
                    cols_with_two_candidates.push(c);
                    candidate_rows_in_cols.push(rows_for_candidate_in_col);
                }
            }

            for i in 0..cols_with_two_candidates.len() {
                for j in (i + 1)..cols_with_two_candidates.len() {
                    let c1 = cols_with_two_candidates[i];
                    let c2 = cols_with_two_candidates[j];
                    let rows1 = &candidate_rows_in_cols[i];
                    let rows2 = &candidate_rows_in_cols[j];

                    if rows1[0] == rows2[0] && rows1[1] == rows2[1] {
                        let r1 = rows1[0];
                        let r2 = rows1[1];

                        // Found an X-Wing in rows r1 and r2 across columns c1 and c2
                        // Remove candidate from other cells in row r1 (excluding c1, c2)
                        for c_other in 0..9 {
                            if c_other != c1 && c_other != c2 && prop.board.is_empty(r1, c_other) {
                                let initial_mask = prop.candidates.get(r1, c_other);
                                if (initial_mask & candidate_bit) != 0 {
                                    eliminations_made |= prop.eliminate_candidate(
                                        r1,
                                        c_other,
                                        candidate_bit,
                                        self.flags(),
                                        path,
                                    );
                                }
                            }
                        }

                        // Remove candidate from other cells in row r2 (excluding c1, c2)
                        for c_other in 0..9 {
                            if c_other != c1 && c_other != c2 && prop.board.is_empty(r2, c_other) {
                                let initial_mask = prop.candidates.get(r2, c_other);
                                if (initial_mask & candidate_bit) != 0 {
                                    eliminations_made |= prop.eliminate_candidate(
                                        r2,
                                        c_other,
                                        candidate_bit,
                                        self.flags(),
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

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::XWING
    }
}
