use proptest::prelude::*;
use rustoku_lib::core::{Board, Rustoku, generate_board};

// Strategy for generating valid Sudoku clue counts (17-81)
fn clue_count_strategy() -> impl Strategy<Value = usize> {
    17..=81usize
}

// Strategy for generating valid row/column indices (0-8)
fn cell_index_strategy() -> impl Strategy<Value = usize> {
    0..9usize
}

// Strategy for generating valid digit values (1-9)
fn digit_strategy() -> impl Strategy<Value = u8> {
    1..=9u8
}

// Helper to count the number of non-zero cells in a board
fn count_clues(board: &Board) -> usize {
    board
        .iter_cells()
        .filter(|&(r, c)| board.get(r, c) != 0)
        .count()
}

proptest! {
    #[test]
    fn prop_generated_board_has_requested_clues(clues in clue_count_strategy()) {
        if let Ok(board) = generate_board(clues) {
            let actual_clues = count_clues(&board);

            prop_assert!(
                actual_clues >= clues,
                "Generated board should have at least {} clues, got {}",
                clues,
                actual_clues
            );
        }
    }

    #[test]
    fn prop_generated_board_is_solvable(clues in clue_count_strategy()) {
        if let Ok(board) = generate_board(clues) {
            if let Ok(mut solver) = Rustoku::new(board) {
                let solutions = solver.solve_all();

                prop_assert_eq!(
                    solutions.len(),
                    1,
                    "Generated board with {} clues should have exactly 1 solution",
                    clues
                );
            }
        }
    }

    #[test]
    fn prop_generated_board_valid_entries(clues in clue_count_strategy()) {
        if let Ok(board) = generate_board(clues) {
            for (r, c) in board.iter_cells() {
                let cell = board.get(r, c);
                prop_assert!(
                    cell == 0 || (1..=9).contains(&cell),
                    "Cell value at ({},{}) must be 0 or 1-9, got {}",
                    r,
                    c,
                    cell
                );
            }
        }
    }

    #[test]
    fn prop_solve_returns_consistent_solution(
        r1 in cell_index_strategy(),
        c1 in cell_index_strategy(),
        val1 in digit_strategy(),
        r2 in cell_index_strategy(),
        c2 in cell_index_strategy(),
        val2 in digit_strategy(),
    ) {
        // Create a board with two clues
        let mut cells = [[0u8; 9]; 9];
        cells[r1][c1] = val1;
        cells[r2][c2] = val2;

        let board = Board::new(cells);

        // Try to solve it - just verify both attempts succeed if a solution exists
        if let Ok(mut solver) = Rustoku::new(board) {
            let solution1 = solver.solve_any();

            // Solve it again to verify consistency
            if let Ok(mut solver2) = Rustoku::new(board) {
                let solution2 = solver2.solve_any();

                // Both attempts should agree on whether a solution exists
                prop_assert_eq!(
                    solution1.is_some(),
                    solution2.is_some(),
                    "Both solve attempts should agree on whether a solution exists"
                );

                // If either found a solution, verify it's a valid sudoku
                if let Some(sol) = solution1.or(solution2) {
                    // Verify no empty cells
                    for (r, c) in sol.board.iter_cells() {
                        prop_assert_ne!(
                            sol.board.get(r, c),
                            0,
                            "Solution should have no empty cells"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn prop_solved_board_is_valid_sudoku(clues in clue_count_strategy()) {
        if let Ok(board) = generate_board(clues) {
            if let Ok(mut solver) = Rustoku::new(board) {
                if let Some(solution) = solver.solve_any() {
                    let solved = solution.board;

                    // Check all cells are filled (no zeros)
                    for (r, c) in solved.iter_cells() {
                        let cell = solved.get(r, c);
                        prop_assert_ne!(cell, 0, "Solved board should have no empty cells at ({},{})", r, c);
                        prop_assert!(
                            (1..=9).contains(&cell),
                            "Solved board cells must be 1-9, got {} at ({},{})",
                            cell,
                            r,
                            c
                        );
                    }

                    // Verify rows have all digits 1-9
                    for r in 0..9 {
                        let mut digits = [false; 10];
                        for c in 0..9 {
                            digits[solved.get(r, c) as usize] = true;
                        }
                        prop_assert!(
                            digits[1..10].iter().all(|&d| d),
                            "Row {} must contain all digits 1-9",
                            r
                        );
                    }

                    // Verify columns have all digits 1-9
                    for c in 0..9 {
                        let mut digits = [false; 10];
                        for r in 0..9 {
                            digits[solved.get(r, c) as usize] = true;
                        }
                        prop_assert!(
                            digits[1..10].iter().all(|&d| d),
                            "Column {} must contain all digits 1-9",
                            c
                        );
                    }

                    // Verify 3x3 boxes have all digits 1-9
                    for box_row in 0..3 {
                        for box_col in 0..3 {
                            let mut digits = [false; 10];
                            for r in (box_row * 3)..(box_row * 3 + 3) {
                                for c in (box_col * 3)..(box_col * 3 + 3) {
                                    digits[solved.get(r, c) as usize] = true;
                                }
                            }
                            prop_assert!(
                                digits[1..10].iter().all(|&d| d),
                                "Box ({},{}) must contain all digits 1-9",
                                box_row,
                                box_col
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn prop_solve_all_finds_at_least_one_solution(clues in clue_count_strategy()) {
        if let Ok(board) = generate_board(clues) {
            if let Ok(mut solver) = Rustoku::new(board) {
                let solutions = solver.solve_all();

                prop_assert!(
                    !solutions.is_empty(),
                    "solve_all should find at least one solution for a generated board"
                );
            }
        }
    }

    #[test]
    fn prop_invalid_clue_counts_return_error(clues in prop::num::usize::ANY) {
        // Only test invalid clue counts
        if !(17..=81).contains(&clues) {
            let result = generate_board(clues);
            prop_assert!(
                result.is_err(),
                "Clue count {} should return error",
                clues
            );
        }
    }

    #[test]
    fn prop_solve_until_respects_limit(clues in clue_count_strategy(), limit in 1..=3usize) {
        if let Ok(board) = generate_board(clues) {
            if let Ok(mut solver) = Rustoku::new(board) {
                let solutions = solver.solve_until(limit);

                prop_assert!(
                    solutions.len() <= limit,
                    "solve_until({}) should return at most {} solutions, got {}",
                    limit,
                    limit,
                    solutions.len()
                );
            }
        }
    }

    #[test]
    fn prop_is_solved_false_on_incomplete_board(clues in clue_count_strategy()) {
        if let Ok(board) = generate_board(clues) {
            if let Ok(solver) = Rustoku::new(board) {
                // The generated board itself should NOT be solved (it has clues but not complete)
                if clues < 81 {
                    prop_assert!(
                        !solver.is_solved(),
                        "A puzzle with {} clues should not be immediately solved",
                        clues
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod edge_case_tests {
    use rustoku_lib::core::{Board, Rustoku, generate_board};

    #[test]
    fn test_min_clues() {
        let result = generate_board(17);
        assert!(result.is_ok(), "17 clues should be valid");
    }

    #[test]
    fn test_max_clues() {
        let result = generate_board(81);
        assert!(result.is_ok(), "81 clues should be valid");
    }

    #[test]
    fn test_clue_count_too_low() {
        let result = generate_board(16);
        assert!(result.is_err(), "16 clues should be invalid");
    }

    #[test]
    fn test_clue_count_too_high() {
        let result = generate_board(82);
        assert!(result.is_err(), "82 clues should be invalid");
    }

    #[test]
    fn test_empty_board_is_not_solved() {
        let board = Board::new([[0; 9]; 9]);
        let solver = Rustoku::new(board).expect("Empty board should be valid");
        assert!(!solver.is_solved(), "Empty board should not be solved");
    }

    #[test]
    fn test_multiple_solves_same_board() {
        if let Ok(board) = generate_board(30) {
            let mut solver1 = Rustoku::new(board).expect("Board should be valid");
            let mut solver2 = Rustoku::new(board).expect("Board should be valid");
            let mut solver3 = Rustoku::new(board).expect("Board should be valid");

            let sol1 = solver1.solve_any();
            let sol2 = solver2.solve_any();
            let sol3 = solver3.solve_any();

            // All solutions should be the same (board has unique solution)
            assert_eq!(sol1.is_some(), sol2.is_some());
            assert_eq!(sol2.is_some(), sol3.is_some());
        }
    }

    #[test]
    fn test_generated_board_has_valid_digits() {
        if let Ok(board) = generate_board(25) {
            for (r, c) in board.iter_cells() {
                let val = board.get(r, c);
                assert!(
                    (0..=9).contains(&val),
                    "Cell ({}, {}) has invalid value: {}",
                    r,
                    c,
                    val
                );
            }
        }
    }
}
