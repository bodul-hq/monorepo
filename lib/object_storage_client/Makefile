include ../../make/rust.mk

.PHONY: doc test-live

doc:
	cargo doc --no-deps

# Explicit opt-in: the test creates immutable artifacts in the selected server.
test-live:
	@test -n "$$OBJECT_STORAGE_URL" || { echo "OBJECT_STORAGE_URL is required"; exit 1; }
	@test -n "$$OBJECT_STORAGE_TOKEN" || { echo "OBJECT_STORAGE_TOKEN is required"; exit 1; }
	cargo test --test live -- --ignored
