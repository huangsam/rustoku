use clap::{ColorChoice, Parser, Subcommand};
use rustoku_lib::Rustoku;
use rustoku_lib::core::TechniqueFlags;

mod csv;

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
        #[arg(short, long, group = "gen_mode")]
        clues: Option<usize>,

        /// The difficulty of the puzzle to generate
        #[arg(short, long, group = "gen_mode")]
        difficulty: Option<rustoku_lib::Difficulty>,
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
        /// Use all human solving techniques (slower but more thorough)
        #[arg(long)]
        human: bool,
    },
    /// 🔍 Attempts to find all puzzle solutions
    All {
        /// The Sudoku puzzle string (81 characters: `0-9` or `.` or `_`)
        puzzle: String,
        /// Show detailed solve path and techniques used
        #[arg(short, long)]
        verbose: bool,
        /// Stop after finding this many solutions (0 = find all)
        #[arg(short = 'u', long = "until", default_value_t = 0_usize)]
        until: usize,
        /// Use all human solving techniques (slower but more thorough)
        #[arg(long)]
        human: bool,
    },
    /// 📊 Solve puzzles from a CSV file
    Csv {
        /// Path to CSV file with 'quizzes' and optional 'solutions' columns
        #[arg(value_name = "FILE")]
        file: String,
        /// Output file for results (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
        /// Use all human solving techniques (slower but more thorough)
        #[arg(long)]
        human: bool,
        /// Show statistics only (count of solved/unsolved)
        #[arg(long)]
        stats_only: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Generate { clues, difficulty } => handle_generate(clues, difficulty),
        Commands::Solve { solve_command } => match solve_command {
            SolveCommands::Any {
                puzzle,
                verbose,
                human,
            } => handle_solve_any(&puzzle, verbose, human),
            SolveCommands::All {
                puzzle,
                verbose,
                until,
                human,
            } => handle_solve_all(&puzzle, verbose, until, human),
            SolveCommands::Csv {
                file,
                output,
                human,
                stats_only,
            } => csv::solve_csv_file(&file, output, human, stats_only),
        },
        Commands::Check { puzzle } => handle_check(&puzzle),
        Commands::Show { puzzle } => handle_show(&puzzle),
    };

    if let Err(e) = result {
        eprintln!("💥 Error: {e}");
        std::process::exit(1);
    }
}

fn handle_generate(
    clues: Option<usize>,
    difficulty: Option<rustoku_lib::Difficulty>,
) -> Result<(), rustoku_lib::RustokuError> {
    if let Some(diff) = difficulty {
        rustoku_lib::generate_board_by_difficulty(diff, 1000).map(|board| {
            println!("🎲 Generated {} puzzle:", diff);
            println!("{board}")
        })
    } else {
        let clues = clues.unwrap_or(30);
        rustoku_lib::generate_board(clues).map(|board| {
            println!("🎲 Generated puzzle with {clues} clues:");
            println!("{board}")
        })
    }
}

fn handle_solve_any(
    puzzle: &str,
    verbose: bool,
    human: bool,
) -> Result<(), rustoku_lib::RustokuError> {
    let techniques = if human {
        TechniqueFlags::all()
    } else {
        TechniqueFlags::EASY
    };

    Rustoku::builder()
        .board_from_str(puzzle)
        .and_then(|b| b.techniques(techniques).build())
        .map(|mut rustoku| match rustoku.solve_any() {
            None => println!("🚫 No solution found"),
            Some(solution) => {
                println!("🎯 Solution found:");
                if verbose {
                    println!("{}\n\n{}", solution.board, solution.solve_path);
                } else {
                    println!("{}", solution.board);
                }
            }
        })
}

fn handle_solve_all(
    puzzle: &str,
    verbose: bool,
    until: usize,
    human: bool,
) -> Result<(), rustoku_lib::RustokuError> {
    let techniques = if human {
        TechniqueFlags::all()
    } else {
        TechniqueFlags::EASY
    };

    Rustoku::builder()
        .board_from_str(puzzle)
        .and_then(|b| b.techniques(techniques).build())
        .map(|mut rustoku| {
            let solutions = if until > 0 {
                rustoku.solve_until(until)
            } else {
                rustoku.solve_all()
            };

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
                    println!("🔍 Found {n} solutions:");
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

fn handle_check(puzzle: &str) -> Result<(), rustoku_lib::RustokuError> {
    Rustoku::builder()
        .board_from_str(puzzle)
        .and_then(|b| b.build())
        .map(|rustoku| {
            if rustoku.is_solved() {
                println!("✅ Puzzle is solved correctly!");
            } else {
                println!("❌ Puzzle is not solved correctly");
            }
        })
}

fn handle_show(puzzle: &str) -> Result<(), rustoku_lib::RustokuError> {
    Rustoku::builder()
        .board_from_str(puzzle)
        .and_then(|b| b.build())
        .map(|rustoku| {
            println!("🎨 Show puzzle:");
            println!("{}", rustoku.board);
        })
}
