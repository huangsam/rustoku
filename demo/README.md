# Rustoku Web UI

A web interface for Rustoku, powered by Rust + WebAssembly.

## Quick Start

Build WASM first:

```bash
cd rustoku-wasm && wasm-pack build --target web --release
```

Then copy the WASM assets to `demo/pkg/` and run the dev server:

```bash
npm install
npm run dev
```

## Build & Deploy

GitHub Actions (`deploy.yml`) automatically builds WASM, copies to `demo/pkg/`, runs the web UI build to generate a `dist` directory and deploys the final artifacts to GitHub Pages on push to `main`.
