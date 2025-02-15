PROJECT_NAME := diskdb

.PHONY: all build run debug clean fmt clippy test

all: build

build:
	cargo build --release

debug:
	cargo build

run: build
	cargo run --release

debug-run: debug
	cargo run

clean:
	cargo clean

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features

test:
	cargo test
