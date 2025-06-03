use clap::{Parser, Subcommand};
use rustoku_lib::core::TechniqueFlags;
use rustoku_lib::{Rustoku, generate_board};

/// Root of the Rustoku CLI.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = ">>> Rustoku: Lightning-fast Sudoku <<<",
    long_about = "Rustoku handles solving, checking, and generating puzzles, delivering unparalleled speed and clarity."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Commands for the Rustoku CLI.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generates a new Sudoku puzzle with a unique solution
    Generate {
        /// The desired number of initially filled cells (clues) for the puzzle
        #[arg(short, long, default_value_t = 30)] // Default to 30 clues
        clues: usize,
    },
    /// Solves a given Sudoku puzzle
    Solve {
        #[command(subcommand)]
        solve_command: SolveCommands,
    },
    /// Checks if a given Sudoku puzzle is solved correctly
    Check {
        /// The Sudoku puzzle string (81 characters: `0-9` or `.` or `_`)
        puzzle: String,
    },
    /// Shows the Sudoku puzzle in a grid-like format
    Show {
        /// The Sudoku puzzle string (81 characters: `0-9` or `.` or `_`)
        puzzle: String,
    },
}

/// Subcommands for solving Sudoku puzzles.
#[derive(Subcommand, Debug)]
pub enum SolveCommands {
    /// Attempts to find any puzzle solution
    Any {
        /// The Sudoku puzzle string (81 characters: `0-9` or `.` or `_`)
        puzzle: String,
    },
    /// Attempts to find all puzzle solutions
    All {
        /// The Sudoku puzzle string (81 characters: `0-9` or `.` or `_`)
        puzzle: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Generate { clues } => generate_board(clues).map(|board| print!("{}", board)),
        Commands::Solve { solve_command } => match solve_command {
            SolveCommands::Any { puzzle } => Rustoku::new_from_str(&puzzle).map(|mut rustoku| {
                rustoku = rustoku.with_techniques(TechniqueFlags::all());
                match rustoku.solve_any() {
                    None => println!("No solution found."),
                    Some(solution) => print!("{}", solution),
                }
            }),
            SolveCommands::All { puzzle } => Rustoku::new_from_str(&puzzle).map(|mut rustoku| {
                rustoku = rustoku.with_techniques(TechniqueFlags::all());
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
            }),
        },
        Commands::Check { puzzle } => Rustoku::new_from_str(&puzzle).map(|rustoku| {
            println!(
                "The puzzle is {}solved correctly.",
                if rustoku.is_solved() { "" } else { "NOT " }
            );
        }),
        Commands::Show { puzzle } => Rustoku::new_from_str(&puzzle).map(|rustoku| {
            print!("{}", rustoku.board);
        }),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
