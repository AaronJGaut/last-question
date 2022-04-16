run-dev:
	cargo run --features bevy/dynamic
.PHONY: run-dev

build-native:
	cargo build --release
.PHONY: build-native

build-web:
	cargo build --target wasm32-unknown-unknown --no-default-features
	wasm-bindgen --out-dir . --target web target/wasm32-unknown-unknown/release/last-question.wasm
