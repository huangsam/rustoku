use clap::{Parser, Subcommand};
use rustoku::core::{Rustoku, RustokuError};
use rustoku::format::print_board;

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
        /// The Sudoku puzzle string (81 characters: 0-9 for numbers, . or _ for empty cells)
        puzzle: String,
    },
}

fn main() -> Result<(), RustokuError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { clues } => {
            let puzzle = Rustoku::generate(clues)?;
            print_board(&puzzle);
        }
        Commands::Solve { puzzle, all } => {
            let mut rustoku = Rustoku::try_from(puzzle.as_str())?;
            if all {
                let solutions = rustoku.solve_all();
                solutions.iter().enumerate().for_each(|(i, solution)| {
                    println!("\n--- Solution {} ---", i + 1);
                    print_board(&solution.board);
                });
                println!("Found {} solution(s).", solutions.len());
            } else if let Some(solution) = rustoku.solve_any() {
                print_board(&solution.board);
            } else {
                println!("No solution found.");
            }
        }
        Commands::Check { puzzle } => {
            let rustoku = Rustoku::try_from(puzzle.as_str())?;
            println!(
                "The puzzle is {}solved correctly.",
                if rustoku.is_solved() { "" } else { "NOT " }
            );
        }
    }

    Ok(())
}
