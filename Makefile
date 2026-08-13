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

# Publish only the release package. The versioned yuki-client dependency is
# released independently, and Cargo treats an already-published workspace
# member as a fatal error rather than skipping it.
publish:
	cargo publish -p yuki-cli --locked

publish-dry-run:
	cargo publish -p yuki-cli --dry-run --locked

clean:
	cargo clean

release-patch:
	vership bump patch

release-minor:
	vership bump minor

release-major:
	vership bump major
