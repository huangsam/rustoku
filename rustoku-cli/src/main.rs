use clap::{Parser, Subcommand};
use rustoku_lib::core::TechniqueFlags;
use rustoku_lib::{Rustoku, generate_board};

/// Root of the Rustoku CLI.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "🦀 Rustoku: Lightning-fast Sudoku solver 🦀",
    long_about = "Rustoku solves and generates puzzles, delivering unparalleled speed and clarity",
    color = clap::ColorChoice::Auto,
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Commands for the Rustoku CLI.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 🎲 Generates a new Sudoku puzzle with a unique solution
    Generate {
        /// The desired number of initially filled cells (clues) for the puzzle
        #[arg(short, long, default_value_t = 30)] // Default to 30 clues
        clues: usize,
    },
    /// 🧩 Solves a given Sudoku puzzle
    Solve {
        #[command(subcommand)]
        solve_command: SolveCommands,
    },
    /// ✅ Checks if a given Sudoku puzzle is solved correctly
    Check {
        /// The Sudoku puzzle string (81 characters: `0-9` or `.` or `_`)
        puzzle: String,
    },
    /// 📋 Shows the Sudoku puzzle in a grid-like format
    Show {
        /// The Sudoku puzzle string (81 characters: `0-9` or `.` or `_`)
        puzzle: String,
    },
}

/// Subcommands for solving Sudoku puzzles.
#[derive(Subcommand, Debug)]
pub enum SolveCommands {
    /// 🎯 Attempts to find any puzzle solution
    Any {
        /// The Sudoku puzzle string (81 characters: `0-9` or `.` or `_`)
        puzzle: String,
    },
    /// 🔍 Attempts to find all puzzle solutions
    All {
        /// The Sudoku puzzle string (81 characters: `0-9` or `.` or `_`)
        puzzle: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Generate { clues } => generate_board(clues).map(|board| {
            println!("🎲 Generated puzzle with {} clues:\n", clues);
            println!("{}", board)
        }),
        Commands::Solve { solve_command } => match solve_command {
            SolveCommands::Any { puzzle } => Rustoku::new_from_str(&puzzle).map(|mut rustoku| {
                rustoku = rustoku.with_techniques(TechniqueFlags::all());
                match rustoku.solve_any() {
                    None => println!("🚫 No solution found"),
                    Some(solution) => {
                        println!("🎯 Solution found:\n");
                        println!("{}", solution);
                    }
                }
            }),
            SolveCommands::All { puzzle } => Rustoku::new_from_str(&puzzle).map(|mut rustoku| {
                rustoku = rustoku.with_techniques(TechniqueFlags::all());
                let solutions = rustoku.solve_all();
                match solutions.len() {
                    0 => println!("🚫 No solutions found"),
                    1 => {
                        println!("🎯 Found 1 unique solution:\n");
                        println!("{}", solutions[0]);
                    }
                    n if n <= 5 => {
                        println!("🔍 Found {} solutions:\n", n);
                        solutions.iter().enumerate().for_each(|(i, solution)| {
                            println!("--- Solution {} ---", i + 1);
                            println!("{}", solution);
                            if i < solutions.len() - 1 {
                                println!(); // Add spacing between solutions
                            }
                        });
                        println!("\n✅ All solutions displayed");
                    }
                    n => {
                        println!("⚠️ Found {} solutions (showing first 3):\n", n);
                        solutions
                            .iter()
                            .take(3)
                            .enumerate()
                            .for_each(|(i, solution)| {
                                println!("--- Solution {} ---", i + 1);
                                println!("{}", solution);
                                if i < 2 {
                                    println!(); // Add spacing between first 3 solutions
                                }
                            });
                        println!("\n💡 Puzzle has multiple solutions (+{} more)", n - 3);
                    }
                }
            }),
        },
        Commands::Check { puzzle } => Rustoku::new_from_str(&puzzle).map(|rustoku| {
            if rustoku.is_solved() {
                println!("✅ Puzzle is solved correctly!");
            } else {
                println!("❌ Puzzle is not solved correctly");
            }
        }),
        Commands::Show { puzzle } => Rustoku::new_from_str(&puzzle).map(|rustoku| {
            println!("📋 Puzzle display:\n");
            println!("{}", rustoku.board);
        }),
    };

    if let Err(e) = result {
        eprintln!("💥 Error: {}", e);
        std::process::exit(1);
    }
}
