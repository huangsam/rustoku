use assert_cmd::prelude::*;
use rustoku::error::RustokuError;
use std::process::Command;

// Helper function to get the path to your compiled binary
fn get_rustoku_bin() -> Command {
    // This assumes your binary is named 'rustoku' and is in target/debug or target/release
    // For release builds, you'd typically run `cargo test --release`.
    // We'll use cargo's `env!("CARGO_BIN_EXE_rustoku")` for robustness.
    Command::new(env!("CARGO_BIN_EXE_rustoku"))
}

#[test]
fn test_version_command() {
    get_rustoku_bin()
        .arg("-V")
        .assert()
        .success() // Assert that the command exited successfully
        .stdout(predicates::str::starts_with("rustoku ")); // Assert stdout starts with "rustoku "
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

    // You could also parse the output to count clues if needed, but a basic check is fine for smoke test.
    let output_str = String::from_utf8(output).unwrap();
    // A simple check to ensure it looks like a Sudoku board (e.g., contains numbers and empty cells)
    assert!(output_str.len() > 100); // Very rough check for a full board
}

#[test]
fn test_generate_custom_clues() {
    get_rustoku_bin()
        .arg("generate")
        .arg("--clues")
        .arg("25")
        .assert()
        .success();
}

#[test]
fn test_solve_valid_puzzle() {
    get_rustoku_bin()
        .arg("solve")
        .arg("53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )); // Exact solved board
}

#[test]
fn test_solve_invalid_puzzle_length() {
    get_rustoku_bin()
        .arg("solve")
        .arg("short") // Invalid length
        .assert()
        .failure() // Expect the command to fail
        .stderr(predicates::str::contains(
            RustokuError::InvalidInputLength.to_string(),
        ));
}

#[test]
fn test_solve_all_solutions() {
    get_rustoku_bin()
        .arg("solve")
        .arg("2957438614318659..8761925433874592166123874955492167387635.41899286713541549386..")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicates::str::contains("Found 2 solution(s).")); // Based on your example, this puzzle has 2 solutions
}

#[test]
fn test_check_correct_solution() {
    get_rustoku_bin()
        .arg("check")
        .arg("295743861431865927876192543387459216612387495549216738763524189154938672928671354")
        .assert()
        .success()
        .stdout(predicates::str::contains("The puzzle is solved correctly."));
}

#[test]
fn test_check_incorrect_solution() {
    get_rustoku_bin()
        .arg("check")
        .arg("295743861431865927876192543387459216612387495549216738763524189154938672928671350") // Last digit changed to 0
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "The puzzle is NOT solved correctly.",
        ));
}

#[test]
fn test_show_puzzle() {
    get_rustoku_bin()
        .arg("show")
        .arg("9..5.74....7.8....83.4.1..64.2...3...9.....65....5..8.2..9.8....8..74...7..21.8.3")
        .assert()
        .success()
        .stdout(predicates::str::contains("9..5")) // Beginning of the board
        .stdout(predicates::str::contains(".8.3")); // End of the board
}
