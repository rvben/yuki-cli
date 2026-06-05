.PHONY: build release install test lint fmt run check clean release-patch release-minor release-major

build:
	cargo build --workspace

release:
	cargo build --release --workspace

install:
	cargo install --path .

test:
	cargo test --workspace

lint:
	cargo fmt -- --check
	cargo clippy --workspace --all-targets -- -D warnings

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
