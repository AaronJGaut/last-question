build build-dev:
	cargo build --features bevy/dynamic
.PHONY: build build-dev

run run-dev:
	cargo run --features bevy/dynamic
.PHONY: run run-dev

build-release build-release-native:
	cargo build --release
.PHONY: build-release build-release-native

build-release-web:
	cargo build --target wasm32-unknown-unknown --no-default-features
	wasm-bindgen --out-dir . --target web target/wasm32-unknown-unknown/release/last-question.wasm
.PHONY: build-release-web

push:
	git remote | xargs -n1 git push
	git remote | xargs -n1 git push --tags
.PHONY: push
