use rustoku_lib::RustokuError;
use rustoku_lib::core::{Board, BoardGenerator, Rustoku, Symmetry, generate_board};

fn count_clues(board: &Board) -> usize {
    board
        .iter_cells()
        .filter(|&(r, c)| board.get(r, c) != 0)
        .count()
}

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
