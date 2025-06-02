use super::{TechniquePropagator, TechniqueRule};
use std::collections::HashSet;

/// Locked candidates technique implementation.
pub struct LockedCandidates;

impl LockedCandidates {
    // Helper function for Locked Candidates (row), private to this impl block
    fn process_row_for_locked_candidates(
        prop: &mut TechniquePropagator,
        row: usize,
        path: &mut Vec<(usize, usize, u8)>,
    ) -> bool {
        let mut placements_made = false;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            let candidate_cells: Vec<usize> = (0..9)
                .filter(|&col| {
                    prop.board.is_empty(row, col)
                        && (prop.candidates_cache.get(row, col) & candidate_bit) != 0
                })
                .collect();

            let boxes: HashSet<usize> = candidate_cells
                .iter()
                .map(|&col| (row / 3) * 3 + (col / 3))
                .collect();

            if boxes.len() == 1 {
                let box_idx = *boxes.iter().next().unwrap();
                let start_row = (box_idx / 3) * 3;
                let start_col = (box_idx % 3) * 3;

                for r in start_row..(start_row + 3) {
                    for c in start_col..(start_col + 3) {
                        if r != row && prop.board.is_empty(r, c) {
                            let initial_mask = prop.candidates_cache.get(r, c);
                            if (initial_mask & candidate_bit) != 0 {
                                let refined_mask = initial_mask & !candidate_bit;
                                prop.candidates_cache.set(r, c, refined_mask);
                                placements_made = true;

                                if refined_mask.count_ones() == 1 {
                                    let num = refined_mask.trailing_zeros() as u8 + 1;
                                    if prop.masks.is_safe(r, c, num) {
                                        prop.place_and_update(r, c, num, path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        placements_made
    }

    // Helper function for Locked Candidates (column), private to this impl block
    fn process_col_for_locked_candidates(
        prop: &mut TechniquePropagator,
        col: usize,
        path: &mut Vec<(usize, usize, u8)>,
    ) -> bool {
        let mut placements_made = false;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            let candidate_cells: Vec<usize> = (0..9)
                .filter(|&row| {
                    prop.board.is_empty(row, col)
                        && (prop.candidates_cache.get(row, col) & candidate_bit) != 0
                })
                .collect();

            let boxes: HashSet<usize> = candidate_cells
                .iter()
                .map(|&row| (row / 3) * 3 + (col / 3))
                .collect();

            if boxes.len() == 1 {
                let box_idx = *boxes.iter().next().unwrap();
                let start_row = (box_idx / 3) * 3;
                let start_col = (box_idx % 3) * 3;

                for r in start_row..(start_row + 3) {
                    for c in start_col..(start_col + 3) {
                        if c != col && prop.board.is_empty(r, c) {
                            let initial_mask = prop.candidates_cache.get(r, c);
                            if (initial_mask & candidate_bit) != 0 {
                                let refined_mask = initial_mask & !candidate_bit;
                                prop.candidates_cache.set(r, c, refined_mask);
                                placements_made = true;

                                if refined_mask.count_ones() == 1 {
                                    let num = refined_mask.trailing_zeros() as u8 + 1;
                                    if prop.masks.is_safe(r, c, num) {
                                        prop.place_and_update(r, c, num, path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        placements_made
    }

    // Helper function for Locked Candidates (box), private to this impl block
    fn process_box_for_locked_candidates(
        prop: &mut TechniquePropagator,
        box_idx: usize,
        path: &mut Vec<(usize, usize, u8)>,
    ) -> bool {
        let mut placements_made = false;
        let start_row = (box_idx / 3) * 3;
        let start_col = (box_idx % 3) * 3;

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);

            let mut candidate_cells: Vec<(usize, usize)> = Vec::new();
            for r_offset in 0..3 {
                for c_offset in 0..3 {
                    let r = start_row + r_offset;
                    let c = start_col + c_offset;
                    if prop.board.is_empty(r, c)
                        && (prop.candidates_cache.get(r, c) & candidate_bit) != 0
                    {
                        candidate_cells.push((r, c));
                    }
                }
            }

            // Check if all candidates are in the same row
            let rows: HashSet<usize> = candidate_cells.iter().map(|&(r, _)| r).collect();
            if rows.len() == 1 {
                let row = *rows.iter().next().unwrap();

                for c in 0..9 {
                    if (c < start_col || c >= start_col + 3) && prop.board.is_empty(row, c) {
                        let initial_mask = prop.candidates_cache.get(row, c);
                        if (initial_mask & candidate_bit) != 0 {
                            let refined_mask = initial_mask & !candidate_bit;
                            prop.candidates_cache.set(row, c, refined_mask);
                            placements_made = true;

                            if refined_mask.count_ones() == 1 {
                                let num = refined_mask.trailing_zeros() as u8 + 1;
                                if prop.masks.is_safe(row, c, num) {
                                    prop.place_and_update(row, c, num, path);
                                }
                            }
                        }
                    }
                }
            }

            // Check if all candidates are in the same column
            let cols: HashSet<usize> = candidate_cells.iter().map(|&(_, c)| c).collect();
            if cols.len() == 1 {
                let col = *cols.iter().next().unwrap();

                for r in 0..9 {
                    if (r < start_row || r >= start_row + 3) && prop.board.is_empty(r, col) {
                        let initial_mask = prop.candidates_cache.get(r, col);
                        if (initial_mask & candidate_bit) != 0 {
                            let refined_mask = initial_mask & !candidate_bit;
                            prop.candidates_cache.set(r, col, refined_mask);
                            placements_made = true;

                            if refined_mask.count_ones() == 1 {
                                let num = refined_mask.trailing_zeros() as u8 + 1;
                                if prop.masks.is_safe(r, col, num) {
                                    prop.place_and_update(r, col, num, path);
                                }
                            }
                        }
                    }
                }
            }
        }
        placements_made
    }
}

impl TechniqueRule for LockedCandidates {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut overall_placements_made = false;

        // Check rows for pointing pairs/triples
        for row in 0..9 {
            overall_placements_made |= Self::process_row_for_locked_candidates(prop, row, path);
        }

        // Check columns for pointing pairs/triples
        for col in 0..9 {
            overall_placements_made |= Self::process_col_for_locked_candidates(prop, col, path);
        }

        // Check boxes for box/line reduction
        for box_idx in 0..9 {
            overall_placements_made |= Self::process_box_for_locked_candidates(prop, box_idx, path);
        }

        overall_placements_made
    }
}
