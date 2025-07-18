use criterion::{Criterion, criterion_group, criterion_main};
use rustoku_lib::Rustoku;
use rustoku_lib::core::Board;
use rustoku_lib::generate_board;
use std::hint::black_box;

// Constants for puzzles (can be defined directly or loaded from files)
const UNIQUE_PUZZLE: &str =
    "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
const TWO_PUZZLE: &str =
    "295743861431865900876192543387459216612387495549216738763504189928671354154938600";

fn benchmark_solve_any(c: &mut Criterion) {
    let mut group = c.benchmark_group("Solve Sudoku Puzzles");

    // Benchmark `solve_any` for a unique puzzle
    group.bench_function("solve_any_unique", |b| {
        let board = Board::try_from(UNIQUE_PUZZLE).unwrap();
        let rustoku = Rustoku::new(board).unwrap();
        b.iter(|| {
            // Use black_box to prevent the compiler from optimizing away the computation
            black_box(rustoku.clone().solve_any());
        });
    });

    // Benchmark `solve_any` for a puzzle with two solutions (might be slightly different behavior)
    group.bench_function("solve_any_two_solutions", |b| {
        let board = Board::try_from(TWO_PUZZLE).unwrap();
        let rustoku = Rustoku::new(board).unwrap();
        b.iter(|| {
            black_box(rustoku.clone().solve_any());
        });
    });

    group.finish();
}

fn benchmark_solve_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("Solve All Sudoku Puzzles");

    // Benchmark `solve_all` for a unique puzzle
    group.bench_function("solve_all_unique", |b| {
        let board = Board::try_from(UNIQUE_PUZZLE).unwrap();
        let rustoku = Rustoku::new(board).unwrap();
        b.iter(|| {
            black_box(rustoku.clone().solve_all());
        });
    });

    // Benchmark `solve_all` for a puzzle with two solutions
    group.bench_function("solve_all_two_solutions", |b| {
        let board = Board::try_from(TWO_PUZZLE).unwrap();
        let rustoku = Rustoku::new(board).unwrap();
        b.iter(|| {
            black_box(rustoku.clone().solve_all());
        });
    });

    group.finish();
}

fn benchmark_generate_board(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generate Sudoku Puzzles");

    // Benchmark `generate_board` with a moderate number of clues
    group.bench_function("generate_board_40_clues", |b| {
        b.iter(|| {
            black_box(generate_board(40).unwrap());
        });
    });

    // Benchmark `generate_board` with fewer clues
    group.bench_function("generate_board_30_clues", |b| {
        b.iter(|| {
            black_box(generate_board(30).unwrap());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_solve_any,
    benchmark_solve_all,
    benchmark_generate_board
);
criterion_main!(benches);
