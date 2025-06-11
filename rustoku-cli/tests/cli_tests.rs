use assert_cmd::prelude::*;
use rustoku_lib::RustokuError;
use std::process::Command;

/// Helper function to get the path to our compiled binary
fn get_rustoku_bin() -> Command {
    // This assumes our binary is named 'rustoku' and is in target/debug or target/release
    // We'll use cargo's `env!("CARGO_BIN_EXE_rustoku-cli")` for robustness
    Command::new(env!("CARGO_BIN_EXE_rustoku-cli"))
}

#[test]
fn test_version_command() {
    get_rustoku_bin()
        .arg("-V")
        .assert()
        .success() // Assert that the command exited successfully
        .stdout(predicates::str::starts_with("rustoku-cli ")); // Assert stdout starts with "rustoku "
}

#[test]
fn test_generate_default_clues() {
    let output = get_rustoku_bin()
        .arg("generate")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    let puzzle = output_str
        .lines()
        .find(|line| line.starts_with("Line format:"))
        .expect("Line representation is missing")
        .trim_start_matches("Line format: ");
    assert_eq!(puzzle.len(), 81);
}

#[test]
fn test_generate_custom_clues() {
    get_rustoku_bin()
        .arg("generate")
        .arg("--clues")
        .arg("25")
        .assert()
        .success(); // This should have the same output as the default
}

#[test]
fn test_solve_any_some_solution() {
    get_rustoku_bin()
        .arg("solve")
        .arg("any")
        .arg("530070000600195000098000060800060003400803001700020006060000280000419005000080079")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        ));
}

#[test]
fn test_solve_any_no_solution() {
    get_rustoku_bin()
        .arg("solve")
        .arg("any")
        .arg("078002609030008020002000083000000040043090000007300090200001036001840902050003007")
        .assert()
        .success()
        .stdout(predicates::str::contains("No solution found"));
}

#[test]
fn test_solve_any_invalid_puzzle() {
    get_rustoku_bin()
        .arg("solve")
        .arg("any")
        .arg("short") // Invalid length
        .assert()
        .failure() // Expect the command to fail
        .stderr(predicates::str::contains(
            RustokuError::InvalidInputLength.to_string(),
        ));
}

#[test]
fn test_solve_all_two_solutions() {
    get_rustoku_bin()
        .arg("solve")
        .arg("all")
        .arg("295743861431865900876192543387459216612387495549216738763504189928671354154930000")
        .assert()
        .success()
        .stdout(predicates::str::contains("Found 2 solutions"));
}

#[test]
fn test_check_correct_solution() {
    get_rustoku_bin()
        .arg("check")
        .arg("295743861431865927876192543387459216612387495549216738763524189154938672928671354")
        .assert()
        .success()
        .stdout(predicates::str::contains("Puzzle is solved correctly"));
}

#[test]
fn test_check_incorrect_solution() {
    get_rustoku_bin()
        .arg("check")
        .arg("295743861431865927876192543387459216612387495549216738763524189154938672928671350") // Last digit changed to 0
        .assert()
        .success()
        .stdout(predicates::str::contains("Puzzle is not solved correctly"));
}

#[test]
fn test_show_puzzle() {
    get_rustoku_bin()
        .arg("show")
        .arg("900507400007080000830401006402000300090000065000050080200908000080074000700210803")
        .assert()
        .success()
        .stdout(predicates::str::contains("9005")) // Board start
        .stdout(predicates::str::contains("0803")); // Board end
}
