# WebAssembly User Guide (`rustoku-wasm`)

`rustoku-wasm` allows you to run the Rustoku Sudoku engine directly in the browser with near-native performance.

## About This Binding

The WASM binding exposes the **same core API** as [`rustoku-py`](python.md) and the [Rust library](library.md). All three share identical function signatures and behavior — only the language syntax differs. See the [demo](https://sambyte.net/rustoku/) for a live working example.

## Getting Started

### Prerequisites
- Rust toolchain
- `wasm-pack` (`cargo install wasm-pack`)
- Node.js and a bundler (e.g., Vite)

### Building the WASM Module
The easiest way to integrate with a modern web project is to build targeting `web` directly into your web project's source tree.

```bash
cd rustoku-wasm
wasm-pack build --target web --out-dir ../path/to/web-app/pkg
```

## API Reference (JS/TS)

The following functions are exported from the generated JS bindings:

### `async init()`
Initializes the WASM module. This must be called (or the default export must be awaited) before using other functions.

### `solve(board_str: string): string`
Solves a Sudoku puzzle.
- **Input**: 81-character string.
- **Output**: 81-character solved string, or an empty string if it cannot be solved.

### `generate(difficulty: string): string`
Generates a new Sudoku puzzle.
- **Input**: `"easy"`, `"medium"`, `"hard"`, or `"expert"`.
- **Output**: 81-character puzzle string.

### `solve_all(board_str: string): string[]`
Finds every solution for a puzzle.
- **Output**: Array of 81-character strings (empty array if unsolvable or input is invalid).

### `solve_steps(board_str: string, difficulty: string): { board: string, steps: Step[] } | null`
Solves a puzzle and returns a full step-by-step trace.
- **`difficulty`**: `"easy"`, `"medium"`, `"hard"`, or `"expert"`.
- **Output**: Object `{ board: string, steps: Step[] }`, or `null` if
  unsolvable or input is invalid. Each `Step` has: `type` (`"placement"` or
  `"elimination"`), `row`, `col`, `value`, `technique`, `step_number`,
  `candidates_eliminated`, `related_cell_count`, `difficulty_point`.

### `candidates(board_str: string): number[][][]`
Returns the valid candidate digits for every cell.
- **Output**: 9×9 array. Filled cells return `[]`. Returns `null` if the input is invalid.

### `check(board_str: string): boolean`
Validates a solved Sudoku board.
- **Input**: 81-character string.
- **Output**: `true` if valid and complete, `false` otherwise.

## Web Integration Example (Vite + TypeScript)

1. **Build the pkg**:
    ```bash
    wasm-pack build --target web --out-dir brainstorm/web/pkg
    ```

2. **Use in `main.ts`**:
    ```typescript
    import init, { solve, solve_all, solve_steps, candidates, generate, check } from './pkg/rustoku_wasm.js';

    async function run(): Promise<void> {
      await init();

      // Generate and solve
      const puzzle: string = generate("hard");
      const solution: string = solve(puzzle);
      console.log("Solved:", solution);

      // All solutions (uniqueness check)
      const all: string[] = solve_all(puzzle);
      console.log("Solution count:", all.length);

      // Step-by-step trace
      const trace = solve_steps(puzzle, "hard");
      if (trace) {
        trace.steps.slice(0, 3).forEach((s) =>
          console.log(`R${s.row}C${s.col} = ${s.value} via ${s.technique}`)
        );
      }

      // Pencil-mark candidates
      const grid = candidates(puzzle);
      console.log("Candidates at R0C2:", grid[0][2]);
    }

    run();
    ```

## Error Handling

WASM functions return `null` or empty results on invalid input:

```javascript
const solution = solve(puzzle);
if (!solution || solution.length === 0) {
  console.error("Invalid puzzle or unsolvable");
}

const steps = solve_steps(puzzle, "hard");
if (!steps) {
  console.error("Puzzle is unsolvable or malformed");
}

const grid = candidates(puzzle);
if (!grid) {
  console.error("Invalid puzzle string");
}
```

## Integration Tips

- **Vite Plugin**: For even easier integration, consider `vite-plugin-wasm`.
- **Panic Hook**: The module automatically sets up a `console_error_panic_hook` on start, so any Rust-side panics will be logged clearly to the browser console.
