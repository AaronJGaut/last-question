run-dev:
	cargo run --features bevy/dynamic
.PHONY: run

build-release:
	cargo build --release
.PHONY: prod
