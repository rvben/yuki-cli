.PHONY: build release install test lint fmt run check clean

build:
	cargo build

release:
	cargo build --release

install:
	cargo install --path .

test:
	cargo test

lint:
	cargo clippy -- -D warnings
	cargo fmt -- --check

fmt:
	cargo fmt

run:
	cargo run --

check: lint test

clean:
	cargo clean
