ifdef RELEASE
	BUILD_FLAGS = --release
else
	BUILD_FLAGS = --features bevy/dynamic
endif

RUSTFLAGS += --remap-path-prefix=$(shell realpath src)=src
SERVE_DOMAIN ?= localhost

BINDGEN_PATH := ./target/bindgen

build:
	cargo build ${BUILD_FLAGS}
.PHONY: build

run:
	cargo run ${BUILD_FLAGS}
.PHONY: run

build-web: | ${BINDGEN_PATH}/index.html ${BINDGEN_PATH}/assets
	cargo build --target wasm32-unknown-unknown --no-default-features --release
	wasm-bindgen --out-dir ${BINDGEN_PATH} --target web target/wasm32-unknown-unknown/release/last-question.wasm
.PHONY: build-web

${BINDGEN_PATH}/assets: $(shell find assets) | ${BINDGEN_PATH}
	mkdir -p $@
	cp -r assets/* $@

serve:
	sudo caddy file-server --domain ${SERVE_DOMAIN} --root ${BINDGEN_PATH}
.PHONY: serve

${BINDGEN_PATH}:
	mkdir -p $@

${BINDGEN_PATH}/index.html: index.html | ${BINDGEN_PATH}
	cp $< $@

push:
	git remote | xargs -n1 git push
	git remote | xargs -n1 git push --tags
.PHONY: push

format fmt:
	cargo fmt
.PHONY: format fmt
