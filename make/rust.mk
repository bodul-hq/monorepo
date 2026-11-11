# Route compilation through sccache when it is installed, so repeated builds
# reuse cached objects. An inherited RUSTC_WRAPPER, including CI's, wins;
# disable with RUSTC_WRAPPER= on the command line.
ifneq ($(shell command -v sccache 2>/dev/null),)
ifeq ($(origin RUSTC_WRAPPER),undefined)
export RUSTC_WRAPPER := sccache
endif
endif

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
