#![allow(clippy::collapsible_if)]
use proptest::prelude::*;
use rustoku_lib::RustokuError;
use rustoku_lib::core::{Board, Rustoku, Solutions, generate_board};

const TWO_PUZZLE: &str =
    "295743861431865900876192543387459216612387495549216738763504189928671354154938600";
const SIX_PUZZLE: &str =
    "295743001431865900876192543387459216612387495549216738763500000000000000000000000";

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

// Strategy for generating valid board characters: '0'-'9', '.', '_'
fn valid_board_char_strategy() -> impl Strategy<Value = char> {
    prop_oneof![
        Just('0'),
        Just('.'),
        Just('_'),
        (1..=9u32).prop_map(|d| char::from_digit(d, 10).unwrap()),
    ]
}

// Strategy for generating valid 81-character board strings
fn board_string_81_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(valid_board_char_strategy(), 81)
        .prop_map(|chars| chars.into_iter().collect())
}

// Strategy for generating random cell assignments (0 to 35 placements)
fn sparse_cells_strategy() -> impl Strategy<Value = [[u8; 9]; 9]> {
    prop::collection::vec(
        (
            cell_index_strategy(),
            cell_index_strategy(),
            digit_strategy(),
        ),
        0..=35,
    )
    .prop_map(|placements| {
        let mut cells = [[0u8; 9]; 9];
        for (r, c, val) in placements {
            cells[r][c] = val;
        }
        cells
    })
}

// Independent duplicate-checking oracle for rows, columns, and 3x3 boxes
fn board_has_duplicates(board: &Board) -> bool {
    // Check rows
    for r in 0..9 {
        let mut seen = 0u16;
        for c in 0..9 {
            let val = board.get(r, c);
            if val != 0 {
                let bit = 1 << val;
                if seen & bit != 0 {
                    return true;
                }
                seen |= bit;
            }
        }
    }
    // Check columns
    for c in 0..9 {
        let mut seen = 0u16;
        for r in 0..9 {
            let val = board.get(r, c);
            if val != 0 {
                let bit = 1 << val;
                if seen & bit != 0 {
                    return true;
                }
                seen |= bit;
            }
        }
    }
    // Check 3x3 boxes
    for box_row in 0..3 {
        for box_col in 0..3 {
            let mut seen = 0u16;
            for r in (box_row * 3)..(box_row * 3 + 3) {
                for c in (box_col * 3)..(box_col * 3 + 3) {
                    let val = board.get(r, c);
                    if val != 0 {
                        let bit = 1 << val;
                        if seen & bit != 0 {
                            return true;
                        }
                        seen |= bit;
                    }
                }
            }
        }
    }
    false
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
            prop_assert_eq!(result, Err(RustokuError::InvalidClueCount));
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
    fn prop_solve_until_bounds_two_solution_puzzle(limit in 0..=5usize) {
        let mut solver = Rustoku::new_from_str(TWO_PUZZLE).expect("valid puzzle");
        let solutions = solver.solve_until(limit);
        let expected = if limit == 0 { 2 } else { limit.min(2) };

        prop_assert_eq!(
            solutions.len(),
            expected,
            "solve_until({}) on TWO_PUZZLE should yield exactly {} solutions",
            limit,
            expected
        );

        // All returned solutions must be distinct
        for i in 0..solutions.len() {
            for j in (i + 1)..solutions.len() {
                prop_assert_ne!(
                    solutions[i].board,
                    solutions[j].board,
                    "Duplicate solution found at index {} and {}",
                    i,
                    j
                );
            }
        }

        // All returned solutions must be valid solved boards
        for sol in &solutions {
            prop_assert!(
                Rustoku::new(sol.board).is_ok_and(|r| r.is_solved()),
                "Every solution from solve_until must be a valid completed board"
            );
        }

        // Solutions iterator must match when limit > 0
        if limit > 0 {
            let iter_solver = Rustoku::new_from_str(TWO_PUZZLE).unwrap();
            let iter_count = Solutions::from_solver(iter_solver).take(limit).count();
            prop_assert_eq!(
                iter_count, expected,
                "Solutions iterator .take({}) count ({}) must match solve_until count ({})",
                limit, iter_count, expected
            );
        }
    }

    #[test]
    fn prop_solve_until_bounds_six_solution_puzzle(limit in 0..=10usize) {
        let mut solver = Rustoku::new_from_str(SIX_PUZZLE).expect("valid puzzle");
        let solutions = solver.solve_until(limit);
        let expected = if limit == 0 { 6 } else { limit.min(6) };

        prop_assert_eq!(
            solutions.len(),
            expected,
            "solve_until({}) on SIX_PUZZLE should yield exactly {} solutions",
            limit,
            expected
        );

        // All returned solutions must be distinct
        for i in 0..solutions.len() {
            for j in (i + 1)..solutions.len() {
                prop_assert_ne!(
                    solutions[i].board,
                    solutions[j].board,
                    "Duplicate solution found at index {} and {}",
                    i,
                    j
                );
            }
        }

        // All returned solutions must be valid solved boards
        for sol in &solutions {
            prop_assert!(
                Rustoku::new(sol.board).is_ok_and(|r| r.is_solved()),
                "Every solution from solve_until must be a valid completed board"
            );
        }

        // Solutions iterator must match when limit > 0
        if limit > 0 {
            let iter_solver = Rustoku::new_from_str(SIX_PUZZLE).unwrap();
            let iter_count = Solutions::from_solver(iter_solver).take(limit).count();
            prop_assert_eq!(
                iter_count, expected,
                "Solutions iterator .take({}) count ({}) must match solve_until count ({})",
                limit, iter_count, expected
            );
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

    #[test]
    fn prop_solutions_iterator_matches_solve_all(clues in clue_count_strategy()) {
        if let Ok(board) = generate_board(clues) {
            if let (Ok(solver1), Ok(mut solver2)) = (Rustoku::new(board), Rustoku::new(board)) {
                let iter_solutions: Vec<_> = Solutions::from_solver(solver1).collect();
                let all_solutions = solver2.solve_all();

                prop_assert_eq!(
                    iter_solutions.len(),
                    all_solutions.len(),
                    "Solutions iterator count ({}) must match solve_all count ({})",
                    iter_solutions.len(),
                    all_solutions.len()
                );

                for sol in &iter_solutions {
                    prop_assert!(
                        Rustoku::new(sol.board).is_ok_and(|r| r.is_solved()),
                        "Board yielded by Solutions iterator must be a valid solved board"
                    );
                }
            }
        }
    }

    #[test]
    fn prop_solutions_iterator_bounded_matches_solve_until(
        r1 in cell_index_strategy(),
        c1 in cell_index_strategy(),
        val1 in digit_strategy(),
        r2 in cell_index_strategy(),
        c2 in cell_index_strategy(),
        val2 in digit_strategy(),
        limit in 1..=3usize,
    ) {
        let mut cells = [[0u8; 9]; 9];
        cells[r1][c1] = val1;
        cells[r2][c2] = val2;
        let board = Board::new(cells);

        if let (Ok(solver1), Ok(mut solver2)) = (Rustoku::new(board), Rustoku::new(board)) {
            let iter_solutions: Vec<_> = Solutions::from_solver(solver1).take(limit).collect();
            let until_solutions = solver2.solve_until(limit);

            prop_assert_eq!(
                iter_solutions.len(),
                until_solutions.len(),
                "Bounded Solutions iterator count ({}) must match solve_until count ({})",
                iter_solutions.len(),
                until_solutions.len()
            );

            for sol in &iter_solutions {
                prop_assert!(
                    Rustoku::new(sol.board).is_ok_and(|r| r.is_solved()),
                    "Board yielded by bounded Solutions iterator must be a valid solved board"
                );
            }
        }
    }

    #[test]
    fn prop_board_from_arbitrary_string_does_not_panic(s in any::<String>()) {
        match Board::try_from(s.as_str()) {
            Ok(board) => {
                prop_assert_eq!(s.len(), 81);
                for (r, c) in board.iter_cells() {
                    prop_assert!(board.get(r, c) <= 9);
                }
            }
            Err(e) => {
                prop_assert!(
                    matches!(
                        e,
                        RustokuError::InvalidInputLength | RustokuError::InvalidInputCharacter
                    ),
                    "Unexpected error for input {s:?}: {e:?}"
                );
            }
        }
    }

    #[test]
    fn prop_rustoku_from_arbitrary_string_does_not_panic(s in any::<String>()) {
        match Rustoku::new_from_str(&s) {
            Ok(_) => {
                prop_assert_eq!(s.len(), 81);
            }
            Err(e) => {
                prop_assert!(
                    matches!(
                        e,
                        RustokuError::InvalidInputLength
                            | RustokuError::InvalidInputCharacter
                            | RustokuError::DuplicateValues
                    ),
                    "Unexpected error for input {s:?}: {e:?}"
                );
            }
        }
    }

    #[test]
    fn prop_board_from_valid_syntax_string_always_succeeds(s in board_string_81_strategy()) {
        let board_res = Board::try_from(s.as_str());
        prop_assert!(board_res.is_ok());

        // Rustoku::new_from_str should only fail on duplicate values
        if let Err(e) = Rustoku::new_from_str(&s) {
            prop_assert_eq!(e, RustokuError::DuplicateValues);
        }
    }

    #[test]
    fn prop_rustoku_duplicate_detection_and_bitmask_invariants(cells in sparse_cells_strategy()) {
        let board = Board::new(cells);
        let has_dups = board_has_duplicates(&board);
        let result = Rustoku::new(board);

        if has_dups {
            prop_assert_eq!(result.err(), Some(RustokuError::DuplicateValues));
        } else {
            prop_assert!(result.is_ok());
            let solver = result.unwrap();

            // Verify Candidate cache bitmask invariants:
            // For every empty cell, candidate presence must strictly match masks.is_safe(r, c, digit)
            for (r, c) in board.iter_empty_cells() {
                let candidates = solver.candidates.get_candidates(r, c);
                for v in 1..=9u8 {
                    let has_candidate = candidates.contains(&v);
                    let safe = solver.masks.is_safe(r, c, v);
                    prop_assert_eq!(
                        has_candidate,
                        safe,
                        "Candidate digit {} presence at ({}, {}) should equal is_safe",
                        v, r, c
                    );
                }
            }

            // For every filled cell, its digit must be recorded in masks
            for (r, c) in board.iter_cells() {
                let val = board.get(r, c);
                if val != 0 {
                    prop_assert!(
                        !solver.masks.is_safe(r, c, val),
                        "Placed digit {} at ({}, {}) should be recorded in masks",
                        val, r, c
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::count_clues;
    use rustoku_lib::RustokuError;
    use rustoku_lib::core::{Board, BoardGenerator, Rustoku, Symmetry, generate_board};

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
        assert_eq!(result, Err(RustokuError::InvalidClueCount));
    }

    #[test]
    fn test_clue_count_too_high() {
        let result = generate_board(82);
        assert_eq!(result, Err(RustokuError::InvalidClueCount));
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

    #[test]
    fn test_rotational_180_symmetry() {
        if let Ok(board) = BoardGenerator::new()
            .clues(25)
            .symmetry(Symmetry::Rotational180)
            .generate()
        {
            for (r, c) in board.iter_cells() {
                let val = board.get(r, c);
                let partner_val = board.get(8 - r, 8 - c);
                if val != 0 {
                    assert!(
                        partner_val != 0,
                        "Cell ({}, {}) has value but partner ({}, {}) is empty",
                        r,
                        c,
                        8 - r,
                        8 - c
                    );
                } else {
                    assert!(
                        partner_val == 0,
                        "Cell ({}, {}) is empty but partner ({}, {}) has value",
                        r,
                        c,
                        8 - r,
                        8 - c
                    );
                }
            }
        }
    }

    #[test]
    fn test_mirror_vertical_symmetry() {
        if let Ok(board) = BoardGenerator::new()
            .clues(25)
            .symmetry(Symmetry::MirrorVertical)
            .generate()
        {
            for (r, c) in board.iter_cells() {
                let val = board.get(r, c);
                let partner_val = board.get(r, 8 - c);
                if val != 0 {
                    assert!(
                        partner_val != 0,
                        "Cell ({}, {}) has value but partner ({}, {}) is empty",
                        r,
                        c,
                        r,
                        8 - c
                    );
                } else {
                    assert!(
                        partner_val == 0,
                        "Cell ({}, {}) is empty but partner ({}, {}) has value",
                        r,
                        c,
                        r,
                        8 - c
                    );
                }
            }
        }
    }

    #[test]
    fn test_rotational_90_symmetry() {
        if let Ok(board) = BoardGenerator::new()
            .clues(25)
            .symmetry(Symmetry::Rotational90)
            .generate()
        {
            for (r, c) in board.iter_cells() {
                let val = board.get(r, c);
                for (pr, pc) in Symmetry::Rotational90.get_partners(r, c) {
                    let partner_val = board.get(pr, pc);
                    if val != 0 {
                        assert!(
                            partner_val != 0,
                            "Cell ({}, {}) has value but 90-deg partner ({}, {}) is empty",
                            r,
                            c,
                            pr,
                            pc
                        );
                    } else {
                        assert!(
                            partner_val == 0,
                            "Cell ({}, {}) is empty but 90-deg partner ({}, {}) has value",
                            r,
                            c,
                            pr,
                            pc
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_mirror_horizontal_symmetry() {
        if let Ok(board) = BoardGenerator::new()
            .clues(25)
            .symmetry(Symmetry::MirrorHorizontal)
            .generate()
        {
            for (r, c) in board.iter_cells() {
                let val = board.get(r, c);
                let partner_val = board.get(8 - r, c);
                if val != 0 {
                    assert!(
                        partner_val != 0,
                        "Cell ({}, {}) has value but horizontal partner ({}, {}) is empty",
                        r,
                        c,
                        8 - r,
                        c
                    );
                } else {
                    assert!(
                        partner_val == 0,
                        "Cell ({}, {}) is empty but horizontal partner ({}, {}) has value",
                        r,
                        c,
                        8 - r,
                        c
                    );
                }
            }
        }
    }

    #[test]
    fn test_mirror_diagonal_symmetry() {
        if let Ok(board) = BoardGenerator::new()
            .clues(25)
            .symmetry(Symmetry::MirrorDiagonal)
            .generate()
        {
            for (r, c) in board.iter_cells() {
                let val = board.get(r, c);
                let partner_val = board.get(c, r);
                if val != 0 {
                    assert!(
                        partner_val != 0,
                        "Cell ({}, {}) has value but diagonal partner ({}, {}) is empty",
                        r,
                        c,
                        c,
                        r
                    );
                } else {
                    assert!(
                        partner_val == 0,
                        "Cell ({}, {}) is empty but diagonal partner ({}, {}) has value",
                        r,
                        c,
                        c,
                        r
                    );
                }
            }
        }
    }

    #[test]
    fn test_none_symmetry() {
        let board_res = BoardGenerator::new()
            .clues(25)
            .symmetry(Symmetry::None)
            .generate();
        assert!(
            board_res.is_ok(),
            "Generating puzzle with Symmetry::None should succeed"
        );
        let board = board_res.unwrap();
        let clues = count_clues(&board);
        assert!(clues >= 25, "Expected at least 25 clues, got {clues}");
    }
}
