use crate::core::SolvePath;

use super::{TechniquePropagator, TechniqueRule, units};

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

impl XWing {
    /// Finds X-Wing patterns in rows and eliminates candidates.
    fn find_row_based_x_wings(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        let rows_with_two = units::find_units_with_n_candidates(
            candidate_bit,
            2,
            prop.candidates,
            prop.board,
            units::UnitType::Row,
        );

        let mut eliminations_made = false;

        // Check pairs of rows to see if they form an X-Wing
        for i in 0..rows_with_two.len() {
            for j in (i + 1)..rows_with_two.len() {
                let (r1, ref cols1) = rows_with_two[i];
                let (r2, ref cols2) = rows_with_two[j];

                if cols1[0] == cols2[0] && cols1[1] == cols2[1] {
                    let c1 = cols1[0];
                    let c2 = cols1[1];

                    // Found X-Wing - eliminate from other cells in these columns
                    eliminations_made |= Self::eliminate_from_columns_excluding_rows(
                        prop,
                        candidate_bit,
                        c1,
                        c2,
                        &[r1, r2],
                        path,
                        flags,
                    );
                }
            }
        }

        eliminations_made
    }

    /// Finds X-Wing patterns in columns and eliminates candidates.
    fn find_column_based_x_wings(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        let cols_with_two = units::find_units_with_n_candidates(
            candidate_bit,
            2,
            prop.candidates,
            prop.board,
            units::UnitType::Column,
        );

        let mut eliminations_made = false;

        for i in 0..cols_with_two.len() {
            for j in (i + 1)..cols_with_two.len() {
                let (c1, ref rows1) = cols_with_two[i];
                let (c2, ref rows2) = cols_with_two[j];

                if rows1[0] == rows2[0] && rows1[1] == rows2[1] {
                    let r1 = rows1[0];
                    let r2 = rows1[1];

                    // Found X-Wing - eliminate from other cells in these rows
                    eliminations_made |= Self::eliminate_from_rows_excluding_columns(
                        prop,
                        candidate_bit,
                        r1,
                        r2,
                        &[c1, c2],
                        path,
                        flags,
                    );
                }
            }
        }

        eliminations_made
    }

    /// Eliminates a candidate from specified columns, excluding certain rows.
    fn eliminate_from_columns_excluding_rows(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        col1: usize,
        col2: usize,
        exclude_rows: &[usize],
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        for col in [col1, col2] {
            for row in 0..9 {
                if !exclude_rows.contains(&row)
                    && prop.board.is_empty(row, col)
                    && (prop.candidates.get(row, col) & candidate_bit) != 0
                {
                    eliminations_made |=
                        prop.eliminate_candidate(row, col, candidate_bit, flags, path);
                }
            }
        }

        eliminations_made
    }

    /// Eliminates a candidate from specified rows, excluding certain columns.
    fn eliminate_from_rows_excluding_columns(
        prop: &mut TechniquePropagator,
        candidate_bit: u16,
        row1: usize,
        row2: usize,
        exclude_cols: &[usize],
        path: &mut SolvePath,
        flags: crate::core::TechniqueFlags,
    ) -> bool {
        let mut eliminations_made = false;

        for row in [row1, row2] {
            for col in 0..9 {
                if !exclude_cols.contains(&col)
                    && prop.board.is_empty(row, col)
                    && (prop.candidates.get(row, col) & candidate_bit) != 0
                {
                    eliminations_made |=
                        prop.eliminate_candidate(row, col, candidate_bit, flags, path);
                }
            }
        }

        eliminations_made
    }
}

impl TechniqueRule for XWing {
    /// Applies the X-Wing technique by checking for both row-based and column-based X-Wings
    /// for each candidate value (1-9).
    ///
    /// Returns true if any candidate eliminations were made.
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        let mut eliminations_made = false;

        for candidate_val in 1..=9 {
            let candidate_bit = 1 << (candidate_val - 1);

            // Check for row-based X-Wings
            eliminations_made |=
                Self::find_row_based_x_wings(prop, candidate_bit, path, self.flags());

            // Check for column-based X-Wings
            eliminations_made |=
                Self::find_column_based_x_wings(prop, candidate_bit, path, self.flags());
        }

        eliminations_made
    }

    fn flags(&self) -> crate::core::TechniqueFlags {
        crate::core::TechniqueFlags::XWING
    }
}
