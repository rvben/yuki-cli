.PHONY: build release install test lint fmt run check clean release-patch release-minor release-major

build:
	cargo build

release:
	cargo build --release

install:
	cargo install --path .

test:
	cargo test

lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings

fmt:
	cargo fmt

run:
	cargo run --

check: lint test

clean:
	cargo clean

release-patch:
	vership bump patch

release-minor:
	vership bump minor

release-major:
	vership bump major
