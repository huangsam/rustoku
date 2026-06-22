.PHONY: wasm demo-install dev build clean

# Build WASM, run its tests, and sync into demo/pkg
wasm:
	cd rustoku-wasm && npm install && npm run all

# Install demo npm deps
demo-install:
	cd demo && npm install

# Full local dev setup: build WASM then launch dev server
dev: wasm demo-install
	cd demo && npm run dev

# Full production build (mirrors CI)
build: wasm demo-install
	cd demo && npm run build

clean:
	rm -rf rustoku-wasm/pkg demo/pkg demo/dist
