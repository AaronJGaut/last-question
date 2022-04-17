ifdef RELEASE
	BUILD_FLAGS = --release
else
	BUILD_FLAGS = --features bevy/dynamic
endif

build:
	cargo build ${BUILD_FLAGS}
.PHONY: build

run:
	cargo run ${BUILD_FLAGS}
.PHONY: run

build-web:
	cargo build --target wasm32-unknown-unknown --no-default-features
	wasm-bindgen --out-dir . --target web target/wasm32-unknown-unknown/release/last-question.wasm
.PHONY: build-web

push:
	git remote | xargs -n1 git push
	git remote | xargs -n1 git push --tags
.PHONY: push
