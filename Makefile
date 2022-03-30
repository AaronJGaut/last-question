run-dev:
	cargo run --features bevy/dynamic
.PHONY: run-dev

build-native:
	cargo build --release
.PHONY: build-native

#build-web:
