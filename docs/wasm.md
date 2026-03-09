# WebAssembly User Guide (`rustoku-wasm`)

`rustoku-wasm` allows you to run the Rustoku Sudoku engine directly in the browser with near-native performance.

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

### `check(board_str: string): boolean`
Validates a solved Sudoku board.
- **Input**: 81-character string.
- **Output**: `true` if valid and complete, `false` otherwise.

## Web Integration Example (Vite)

1. **Build the pkg**:
    ```bash
    wasm-pack build --target web --out-dir brainstorm/web/pkg
    ```

2. **Use in `main.js`**:
    ```javascript
    import init, { solve, generate } from './pkg/rustoku_wasm.js';

    async function run() {
      // Initialize WASM
      await init();

      // Use the engine
      const puzzle = generate("easy");
      const solution = solve(puzzle);
      console.log("Solved puzzle:", solution);
    }

    run();
    ```

## Integration Tips

- **Vite Plugin**: For even easier integration, consider `vite-plugin-wasm`.
- **Panic Hook**: The module automatically sets up a `console_error_panic_hook` on start, so any Rust-side panics will be logged clearly to the browser console.
