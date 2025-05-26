use clap::{Parser, Subcommand};
use rustoku::{Rustoku, RustokuError, print_board};

#[derive(Parser, Debug)]
#[command(author, version, about = "A Sudoku puzzle solver and generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generates a new Sudoku puzzle with a unique solution
    Generate {
        /// The desired number of initially filled cells (clues) for the puzzle
        #[arg(short, long, default_value_t = 30)] // Default to 30 clues
        clues: usize,
    },
    /// Solves a given Sudoku puzzle
    Solve {
        /// The Sudoku puzzle string (81 characters: 0-9 for numbers, . or _ for empty cells)
        puzzle: String,
        /// Find all solutions instead of just the first one
        #[arg(short, long)]
        all: bool,
    },
    /// Checks if a given Sudoku puzzle is solved correctly
    Check {
        /// The Sudoku puzzle string (81 characters: 0-9 for numbers)
        puzzle: String,
    },
}

/// Prints a single line representation of the Sudoku board.
///
/// This output is useful for compact display or logging purposes.
/// Empty cells are represented by dots (`.`), and filled cells by their respective numbers.
fn line(board: &[[u8; 9]; 9]) -> String {
    board
        .iter()
        .flatten()
        .map(|&n| if n == 0 { '.' } else { (n + b'0') as char })
        .collect()
}

fn main() -> Result<(), RustokuError> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { clues } => {
            println!("Generating a Sudoku puzzle with {} clues...", clues);
            let puzzle = Rustoku::generate(*clues)?;
            print_board(&puzzle);

            let executable = std::env::args()
                .next()
                .unwrap_or_else(|| "cargo run".to_string());
            let command = format!("{} solve '{}'", executable, line(&puzzle));
            println!("\nTo solve this puzzle, run:\n{}", command);
        }
        Commands::Solve { puzzle, all } => {
            println!("Attempting to solve puzzle: {}", puzzle);
            let mut solver = Rustoku::try_from(puzzle.as_str())?;

            if *all {
                let solutions = solver.solve_all();
                println!("Found {} solution(s):", solutions.len());
                solutions.iter().enumerate().for_each(|(i, solution)| {
                    println!("\n--- Solution {} ---", i + 1);
                    print_board(solution);
                    println!("Line representation: {}", line(solution));
                });
            } else if let Some(solution) = solver.solve_any() {
                println!("\nSolution found:");
                print_board(&solution);
                println!("Line representation: {}", line(&solution));
            } else {
                println!("No solution found for the given puzzle.");
            }
        }
        Commands::Check { puzzle } => {
            println!("Checking if the puzzle is solved correctly: {}", puzzle);
            let solver = Rustoku::try_from(puzzle.as_str())?;
            if solver.is_solved() {
                println!("The puzzle is solved correctly!");
            } else {
                println!("The puzzle is NOT solved correctly.");
            }
        }
    }

    Ok(())
}
