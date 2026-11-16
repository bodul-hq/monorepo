include ../../make/rust.mk

ifneq ($(origin DATABASE_URL),undefined)
products_DATABASE_URL := $(value DATABASE_URL)
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
products_DATABASE_URL_TEST := $(value DATABASE_URL_TEST)
endif
ifneq ($(origin PORT),undefined)
products_PORT := $(value PORT)
endif
ifneq ($(origin PRODUCTS_REPORTING_TOKEN),undefined)
products_PRODUCTS_REPORTING_TOKEN := $(value PRODUCTS_REPORTING_TOKEN)
endif
include $(wildcard .env)
ifneq ($(origin products_DATABASE_URL),undefined)
DATABASE_URL := $(products_DATABASE_URL)
endif
ifneq ($(origin DATABASE_URL),undefined)
export DATABASE_URL
endif
ifneq ($(origin products_DATABASE_URL_TEST),undefined)
DATABASE_URL_TEST := $(products_DATABASE_URL_TEST)
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
export DATABASE_URL_TEST
endif
ifneq ($(origin products_PORT),undefined)
PORT := $(products_PORT)
endif
ifneq ($(origin PORT),undefined)
export PORT
endif
ifneq ($(origin products_PRODUCTS_REPORTING_TOKEN),undefined)
PRODUCTS_REPORTING_TOKEN := $(products_PRODUCTS_REPORTING_TOKEN)
endif
ifneq ($(origin PRODUCTS_REPORTING_TOKEN),undefined)
export PRODUCTS_REPORTING_TOKEN
endif
.PHONY: serve migrate migrate-redo require-database-url test-postgres

serve:
	cargo run

.env:
	cp .env.example .env

require-database-url:
	@test -n "$$DATABASE_URL" || { echo "DATABASE_URL is required"; exit 1; }

migrate: require-database-url
	diesel migration run

# Deletes data by reverting and reapplying every migration.
migrate-redo: require-database-url
	diesel migration redo --all

# Each case creates/drops its own schema; no existing tables are reset.
test-postgres:
	@test -n "$$DATABASE_URL_TEST" || { echo "DATABASE_URL_TEST is required; use a separate test database"; exit 1; }
	cargo test --bin products postgres_tests -- --ignored
