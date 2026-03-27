.PHONY: build test lint run clean check

build:
	cargo build

release:
	cargo build --release

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
