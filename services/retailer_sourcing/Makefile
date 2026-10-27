include ../../make/rust.mk

# Preserve shell/command-line settings, including explicitly empty values.
ifneq ($(origin DATABASE_URL),undefined)
retailer_sourcing_database_url := $(value DATABASE_URL)
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
retailer_sourcing_test_database_url := $(value DATABASE_URL_TEST)
endif
ifneq ($(origin OBJECT_STORAGE_URL),undefined)
retailer_sourcing_object_storage_url := $(value OBJECT_STORAGE_URL)
endif
ifneq ($(origin OBJECT_STORAGE_TOKEN),undefined)
retailer_sourcing_object_storage_token := $(value OBJECT_STORAGE_TOKEN)
endif
ifneq ($(origin OFFERS_URL),undefined)
retailer_sourcing_offers_url := $(value OFFERS_URL)
endif
include $(wildcard .env)
ifneq ($(origin retailer_sourcing_offers_url),undefined)
OFFERS_URL := $(retailer_sourcing_offers_url)
endif
ifneq ($(origin OFFERS_URL),undefined)
export OFFERS_URL
endif
ifneq ($(origin retailer_sourcing_database_url),undefined)
DATABASE_URL := $(retailer_sourcing_database_url)
endif
ifneq ($(origin retailer_sourcing_object_storage_url),undefined)
OBJECT_STORAGE_URL := $(retailer_sourcing_object_storage_url)
endif
ifneq ($(origin retailer_sourcing_object_storage_token),undefined)
OBJECT_STORAGE_TOKEN := $(retailer_sourcing_object_storage_token)
endif
ifneq ($(origin retailer_sourcing_test_database_url),undefined)
DATABASE_URL_TEST := $(retailer_sourcing_test_database_url)
endif
export DATABASE_URL
export DATABASE_URL_TEST
export OBJECT_STORAGE_URL
export OBJECT_STORAGE_TOKEN

.PHONY: test-postgres serve migrate migrate-redo require-database-url

serve:
	cargo run

# Applies pending migrations to DATABASE_URL.
migrate: require-database-url
	diesel migration run

# Reverts and reapplies every migration on DATABASE_URL, which deletes its data.
migrate-redo: require-database-url
	diesel migration redo --all

require-database-url:
	@test -n "$$DATABASE_URL" || { echo "DATABASE_URL must be set, e.g. in .env (make .env)"; exit 1; }

.env:
	cp .env.example .env

# Tests use isolated schemas; never reset the selected database.
test-postgres:
	@test -n "$$DATABASE_URL_TEST" || { echo "DATABASE_URL_TEST is required; use a separate test database"; exit 1; }
	cargo test --bin retailer_sourcing postgres_tests -- --ignored

# Creates immutable artifacts in the configured object-storage service.
.PHONY: test-object-storage
test-object-storage:
	@test -n "$$DATABASE_URL_TEST" || { echo "DATABASE_URL_TEST is required"; exit 1; }
	@test -n "$$OBJECT_STORAGE_URL" || { echo "OBJECT_STORAGE_URL is required"; exit 1; }
	@test -n "$$OBJECT_STORAGE_TOKEN" || { echo "OBJECT_STORAGE_TOKEN is required"; exit 1; }
	cargo test --bin retailer_sourcing live_tests:: -- --ignored

.PHONY: export-offer-sources
DATA_DIR ?= ../../../data
export-offer-sources: require-database-url
	cargo run --release --bin export_offer_sources -- $(DATA_DIR)

.PHONY: export-sitemaps
export-sitemaps: require-database-url
	cargo run --release --bin export_sitemaps -- $(DATA_DIR)
