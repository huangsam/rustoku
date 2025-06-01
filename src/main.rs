use clap::{Parser, Subcommand};
use rustoku::core::{Rustoku, RustokuBoard, RustokuTechniques, generate_board};

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
    /// Displays the Sudoku puzzle in a grid-like format
    Show {
        /// The Sudoku puzzle string (81 characters: 0-9 for numbers, . or _ for empty cells)
        puzzle: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Generate { clues } => generate_board(clues).map(|board| println!("{}", board)),
        Commands::Solve { puzzle, all } => {
            RustokuBoard::try_from(puzzle.as_str()).and_then(|board| {
                let mut rustoku = Rustoku::new(board)?;
                rustoku = rustoku.with_techniques(RustokuTechniques::ALL);
                if all {
                    let solutions = rustoku.solve_all();
                    if solutions.is_empty() {
                        println!("No solutions found.");
                    } else {
                        solutions.iter().enumerate().for_each(|(i, solution)| {
                            println!("\n--- Solution {} ---", i + 1);
                            print!("{}", solution);
                        });
                        println!("\nFound {} solution(s).", solutions.len());
                    }
                    Ok(())
                } else {
                    match rustoku.solve_any() {
                        None => println!("No solution found."),
                        Some(solution) => print!("{}", solution),
                    }
                    Ok(())
                }
            })
        }
        Commands::Check { puzzle } => RustokuBoard::try_from(puzzle.as_str()).and_then(|board| {
            let rustoku = Rustoku::new(board)?;
            println!(
                "The puzzle is {}solved correctly.",
                if rustoku.is_solved() { "" } else { "NOT " }
            );
            Ok(())
        }),
        Commands::Show { puzzle } => RustokuBoard::try_from(puzzle.as_str()).map(|board| {
            print!("{}", board);
        }),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
