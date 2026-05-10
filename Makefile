.PHONY: wasm

wasm:
	wasm-pack build crates/slay-wasm --target web --out-dir ../../www/pkg --features browser
