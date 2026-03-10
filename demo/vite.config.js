import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import checker from 'vite-plugin-checker';

export default defineConfig({
  plugins: [
    wasm(),
    checker({
      typescript: true,
    }),
  ],
  optimizeDeps: {
    exclude: ['rustoku-wasm']
  }
});
