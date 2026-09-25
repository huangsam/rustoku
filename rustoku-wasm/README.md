# rustoku-wasm

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/huangsam/rustoku/ci.yml)](https://github.com/huangsam/rustoku/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/huangsam/rustoku/blob/main/LICENSE)

High-performance Sudoku solving and generation in WebAssembly, powered by the core Rustoku engine.

## Installation

```bash
npm install rustoku-wasm
```

## Quick Start

```javascript
import init, {
  generate,
  generate_advanced,
  solve,
  solve_all,
  solve_steps,
  check,
  candidates,
} from 'rustoku-wasm';

async function run() {
  // 1. Initialize WASM module
  await init();

  // 2. Generate a puzzle
  const puzzle = generate('medium');
  console.log('Puzzle:', puzzle);

  // 3. Solve a puzzle
  const solution = solve(puzzle);
  console.log('Solution:', solution);

  // 4. Validate a board
  const isValid = check(solution);
  console.log('Valid:', isValid);

  // 5. Generate with symmetry and difficulty
  const symmetric = generate_advanced('rotational180', 'hard');
  console.log('Symmetric puzzle:', symmetric);

  // 6. Find all solutions
  const solutions = solve_all(puzzle);
  console.log(`Found ${solutions.length} solution(s)`);

  // 7. Get step-by-step human technique solve trace
  const steps = solve_steps(puzzle, 'expert');
  console.log(`Solved in ${steps.length} steps:`, steps);
}

run();
```

## Live Demo

See this package in action in your browser at the [Rustoku Web Demo](https://sambyte.net/rustoku/).

## Features

- **Near-Native Performance**: Solves puzzles in microseconds inside browser and Node.js environments.
- **Human Solving Techniques**: Traces full human-like deduction steps from Naked/Hidden Singles up to Alternating Inference Chains (AIC).
- **Flexible Generation**: Supports difficulty targeting (`easy`, `medium`, `hard`, `expert`) and grid symmetries (`rotational180`, `rotational90`, `mirrorvertical`, `mirrorhorizontal`, `mirrordiagonal`).
- **Zero Heavy Dependencies**: Lightweight WASM binary (< 250 KB uncompressed) with full TypeScript typings included.

## Documentation

For full API documentation and bundler setup (Vite, Webpack, Next.js), see the [WebAssembly Guide](https://github.com/huangsam/rustoku/blob/main/docs/wasm.md) in the GitHub repository.
