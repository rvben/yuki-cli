.PHONY: build release install test lint fmt run check clean publish publish-dry-run release-patch release-minor release-major

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

# Publishes every workspace member in dependency order, so yuki-client reaches
# crates.io before yuki-cli is verified against it. Members whose version is
# already published are skipped with a warning.
publish:
	cargo publish --workspace --locked

publish-dry-run:
	cargo publish --workspace --dry-run --locked

clean:
	cargo clean

release-patch:
	vership bump patch

release-minor:
	vership bump minor

release-major:
	vership bump major
