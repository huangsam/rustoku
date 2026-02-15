use rustoku_lib::Rustoku;
use rustoku_lib::core::TechniqueFlags;
use std::io::Write;

/// Process a CSV file with Sudoku puzzles.
pub fn solve_csv_file(
    file_path: &str,
    output_path: Option<String>,
    human: bool,
    stats_only: bool,
) -> Result<(), rustoku_lib::RustokuError> {
    let techniques = if human {
        TechniqueFlags::all()
    } else {
        TechniqueFlags::EASY
    };

    // Open the input CSV file
    let file = std::fs::File::open(file_path).map_err(|e| {
        eprintln!("❌ Failed to open file '{}': {}", file_path, e);
        rustoku_lib::RustokuError::GenerateFailure
    })?;
    let mut reader = csv::Reader::from_reader(file);

    let mut total = 0u32;
    let mut solved = 0u32;
    let mut unsolvable = 0u32;
    let mut results_vec = Vec::new();

    // Get headers to determine if solutions column exists
    let headers = reader
        .headers()
        .map_err(|e| {
            eprintln!("❌ Failed to read CSV headers: {}", e);
            rustoku_lib::RustokuError::GenerateFailure
        })?
        .clone();
    let has_solutions = headers.iter().any(|h| h.eq_ignore_ascii_case("solutions"));

    for result in reader.records() {
        let record = result.map_err(|e| {
            eprintln!("❌ Failed to read CSV record at line {}: {}", total + 2, e);
            rustoku_lib::RustokuError::GenerateFailure
        })?;
        total += 1;

        // Get the puzzle (quizzes column)
        let puzzle = record.get(0).unwrap_or("");

        // Skip empty puzzles
        if puzzle.is_empty() {
            continue;
        }

        // Try to solve the puzzle
        match Rustoku::builder()
            .board_from_str(puzzle)
            .and_then(|b| b.techniques(techniques).build())
            .and_then(|mut rustoku| {
                rustoku
                    .solve_any()
                    .ok_or(rustoku_lib::RustokuError::GenerateFailure)
            }) {
            Ok(solution) => {
                solved += 1;
                if !stats_only {
                    // Convert board to 81-character string
                    let mut solution_str = String::with_capacity(81);
                    for r in 0..9 {
                        for c in 0..9 {
                            solution_str.push_str(&solution.board.get(r, c).to_string());
                        }
                    }

                    let puzzle_clean = puzzle.replace(" ", "");

                    // Check if expected solution exists and matches
                    if has_solutions && record.len() > 1 {
                        let expected = record.get(1).unwrap_or("").replace(" ", "");
                        let matches = solution_str == expected;
                        results_vec.push((
                            puzzle_clean,
                            solution_str,
                            if matches { "✓" } else { "✗" },
                        ));
                    } else {
                        results_vec.push((puzzle_clean, solution_str, ""));
                    }
                }
            }
            Err(_) => {
                unsolvable += 1;
                if !stats_only {
                    results_vec.push((puzzle.to_string(), "UNSOLVABLE".to_string(), "✗"));
                }
            }
        }

        // Progress indicator for large files
        if total.is_multiple_of(10000) {
            eprintln!("📊 Processed {total} puzzles... ({solved} solved, {unsolvable} unsolvable)");
        }
    }

    // Output results
    if let Some(out_path) = output_path {
        // Write to file
        let mut out_file = std::fs::File::create(&out_path).map_err(|e| {
            eprintln!("❌ Failed to create output file '{}': {}", out_path, e);
            rustoku_lib::RustokuError::GenerateFailure
        })?;

        if has_solutions {
            writeln!(out_file, "quizzes,solutions,match").map_err(|e| {
                eprintln!("❌ Failed to write CSV header: {}", e);
                rustoku_lib::RustokuError::GenerateFailure
            })?;
            for (idx, (puzzle, solution, matches)) in results_vec.iter().enumerate() {
                writeln!(out_file, "{},{},{}", puzzle, solution, matches).map_err(|e| {
                    eprintln!("❌ Failed to write CSV row {}: {}", idx + 1, e);
                    rustoku_lib::RustokuError::GenerateFailure
                })?;
            }
        } else {
            writeln!(out_file, "quizzes,solutions").map_err(|e| {
                eprintln!("❌ Failed to write CSV header: {}", e);
                rustoku_lib::RustokuError::GenerateFailure
            })?;
            for (idx, (puzzle, solution, _)) in results_vec.iter().enumerate() {
                writeln!(out_file, "{},{}", puzzle, solution).map_err(|e| {
                    eprintln!("❌ Failed to write CSV row {}: {}", idx + 1, e);
                    rustoku_lib::RustokuError::GenerateFailure
                })?;
            }
        }

        println!("✅ Results written to {out_path}");
    } else if !stats_only {
        // Write to stdout
        if has_solutions {
            println!("quizzes,solutions,match");
            for (puzzle, solution, matches) in &results_vec {
                println!("{},{},{}", puzzle, solution, matches);
            }
        } else {
            println!("quizzes,solutions");
            for (puzzle, solution, _) in &results_vec {
                println!("{},{}", puzzle, solution);
            }
        }
    }

    // Always print statistics
    let unsolvable_pct = if total > 0 {
        (unsolvable as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    println!("\n📈 Statistics:");
    println!("  Total puzzles: {total}");
    println!("  ✅ Solved: {solved}");
    println!("  ❌ Unsolvable: {unsolvable} ({unsolvable_pct:.2}%)");

    Ok(())
}
