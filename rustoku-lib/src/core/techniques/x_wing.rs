use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule};

/// X-Wing technique implementation.
pub struct XWing;

impl TechniqueRule for XWing {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut eliminations_made = false;

        for candidate_val in 1..=9 {
            let candidate_bit = 1 << (candidate_val - 1);

            // Check for row-based X-Wings
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

            for i in 0..rows_with_two_candidates.len() {
                for j in (i + 1)..rows_with_two_candidates.len() {
                    let r1 = rows_with_two_candidates[i];
                    let r2 = rows_with_two_candidates[j];
                    let cols1 = &candidate_cols_in_rows[i];
                    let cols2 = &candidate_cols_in_rows[j];

                    if cols1[0] == cols2[0] && cols1[1] == cols2[1] {
                        let c1 = cols1[0];
                        let c2 = cols1[1];

                        // Found an X-Wing in columns c1 and c2 across rows r1 and r2
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
