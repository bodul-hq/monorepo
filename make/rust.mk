.PHONY: test check build release fmt fmt-check fix clippy lint ci clean

test:
	cargo test

check:
	cargo check

build:
	cargo build

release:
	cargo build --release

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

fix:
	cargo fix

clippy:
	cargo clippy

lint:
	cargo clippy --all-targets -- -D warnings

ci: fmt-check lint test

clean:
	cargo clean
