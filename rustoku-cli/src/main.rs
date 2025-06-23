use clap::{ColorChoice, Parser, Subcommand};
use rustoku_lib::core::TechniqueFlags;
use rustoku_lib::{Rustoku, generate_board};

/// Root of the Rustoku CLI.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "🦀 Rustoku: Lightning-fast Sudoku solver 🦀",
    long_about = "Rustoku solves and generates puzzles, delivering unparalleled speed and clarity",
    color = ColorChoice::Always,
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
    /// 🎨 Shows the Sudoku puzzle in a grid-like format
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
        /// Show detailed solve path and techniques used
        #[arg(short, long)]
        verbose: bool,
    },
    /// 🔍 Attempts to find all puzzle solutions
    All {
        /// The Sudoku puzzle string (81 characters: `0-9` or `.` or `_`)
        puzzle: String,
        /// Show detailed solve path and techniques used
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Generate { clues } => generate_board(clues).map(|board| {
            println!("🎲 Generated puzzle with {} clues:", clues);
            println!("{}", board)
        }),
        Commands::Solve { solve_command } => match solve_command {
            SolveCommands::Any { puzzle, verbose } => {
                Rustoku::new_from_str(&puzzle).map(|mut rustoku| {
                    rustoku = rustoku.with_techniques(TechniqueFlags::all());
                    match rustoku.solve_any() {
                        None => println!("🚫 No solution found"),
                        Some(solution) => {
                            println!("🎯 Solution found:");
                            if verbose {
                                println!("{}\n\n{}", solution.board, solution.solve_path);
                            } else {
                                println!("{}", solution.board);
                            }
                        }
                    }
                })
            }
            SolveCommands::All { puzzle, verbose } => {
                Rustoku::new_from_str(&puzzle).map(|mut rustoku| {
                    rustoku = rustoku.with_techniques(TechniqueFlags::all());
                    let solutions = rustoku.solve_all();
                    match solutions.len() {
                        0 => println!("🚫 No solutions found"),
                        1 => {
                            println!("🎯 Found 1 unique solution:");
                            if verbose {
                                println!("{}\n\n{}", solutions[0].board, solutions[0].solve_path);
                            } else {
                                println!("{}", solutions[0].board);
                            }
                        }
                        n => {
                            println!("🔍 Found {} solutions:", n);
                            solutions.iter().enumerate().for_each(|(i, solution)| {
                                println!("\n--- Solution {} ---", i + 1);
                                if verbose {
                                    println!("{}\n\n{}", solution.board, solution.solve_path);
                                } else {
                                    println!("{}", solution.board);
                                }
                            });
                            println!("\n✅ All solutions displayed");
                        }
                    }
                })
            }
        },
        Commands::Check { puzzle } => Rustoku::new_from_str(&puzzle).map(|rustoku| {
            if rustoku.is_solved() {
                println!("✅ Puzzle is solved correctly!");
            } else {
                println!("❌ Puzzle is not solved correctly");
            }
        }),
        Commands::Show { puzzle } => Rustoku::new_from_str(&puzzle).map(|rustoku| {
            println!("🎨 Show puzzle:");
            println!("{}", rustoku.board);
        }),
    };

    if let Err(e) = result {
        eprintln!("💥 Error: {}", e);
        std::process::exit(1);
    }
}
