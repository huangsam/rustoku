use criterion::{Criterion, criterion_group, criterion_main};
use rustoku::core::Rustoku;
use rustoku::core::generate_puzzle;
use std::hint::black_box;

// Constants for puzzles (can be defined directly or loaded from files)
const UNIQUE_PUZZLE: &str =
    "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
const TWO_PUZZLE: &str =
    "2957438614318659..8761925433874592166123874955492167387635.41899286713541549386..";

fn benchmark_solve_any(c: &mut Criterion) {
    let mut group = c.benchmark_group("Solve Sudoku Puzzles");

    // Benchmark `solve_any` for a unique puzzle
    group.bench_function("solve_any_unique", |b| {
        let rustoku_puzzle = Rustoku::try_from(UNIQUE_PUZZLE).unwrap();
        b.iter(|| {
            // Use black_box to prevent the compiler from optimizing away the computation
            black_box(rustoku_puzzle.clone().solve_any());
        });
    });

    // Benchmark `solve_any` for a puzzle with two solutions (might be slightly different behavior)
    group.bench_function("solve_any_two_solutions", |b| {
        let rustoku_puzzle = Rustoku::try_from(TWO_PUZZLE).unwrap();
        b.iter(|| {
            black_box(rustoku_puzzle.clone().solve_any());
        });
    });

    group.finish();
}

fn benchmark_solve_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("Solve All Sudoku Puzzles");

    group.bench_function("solve_all_unique", |b| {
        let rustoku_puzzle = Rustoku::try_from(UNIQUE_PUZZLE).unwrap();
        b.iter(|| {
            black_box(rustoku_puzzle.clone().solve_all());
        });
    });

    group.bench_function("solve_all_two_solutions", |b| {
        let rustoku_puzzle = Rustoku::try_from(TWO_PUZZLE).unwrap();
        b.iter(|| {
            black_box(rustoku_puzzle.clone().solve_all());
        });
    });

    group.finish();
}

fn benchmark_generate_puzzle(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generate Sudoku Puzzles");

    // Benchmarking `generate_puzzle` with a moderate number of clues
    group.bench_function("generate_puzzle_40_clues", |b| {
        b.iter(|| {
            black_box(generate_puzzle(40).unwrap());
        });
    });

    // Benchmarking `generate_puzzle` with fewer clues (often harder to generate)
    group.bench_function("generate_puzzle_25_clues", |b| {
        b.iter(|| {
            black_box(generate_puzzle(25).unwrap());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_solve_any,
    benchmark_solve_all,
    benchmark_generate_puzzle
);
criterion_main!(benches);
