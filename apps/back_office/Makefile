include ../../make/rust.mk

ifneq ($(origin DATABASE_URL),undefined)
back_office_DATABASE_URL := $(value DATABASE_URL)
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
back_office_DATABASE_URL_TEST := $(value DATABASE_URL_TEST)
endif
ifneq ($(origin PORT),undefined)
back_office_PORT := $(value PORT)
endif
ifneq ($(origin BACK_OFFICE_COOKIE_SECURE),undefined)
back_office_BACK_OFFICE_COOKIE_SECURE := $(value BACK_OFFICE_COOKIE_SECURE)
endif
include $(wildcard .env)
ifneq ($(origin back_office_DATABASE_URL),undefined)
DATABASE_URL := $(back_office_DATABASE_URL)
endif
ifneq ($(origin DATABASE_URL),undefined)
export DATABASE_URL
endif
ifneq ($(origin back_office_DATABASE_URL_TEST),undefined)
DATABASE_URL_TEST := $(back_office_DATABASE_URL_TEST)
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
export DATABASE_URL_TEST
endif
ifneq ($(origin back_office_PORT),undefined)
PORT := $(back_office_PORT)
endif
ifneq ($(origin PORT),undefined)
export PORT
endif
ifneq ($(origin back_office_BACK_OFFICE_COOKIE_SECURE),undefined)
BACK_OFFICE_COOKIE_SECURE := $(back_office_BACK_OFFICE_COOKIE_SECURE)
endif
ifneq ($(origin BACK_OFFICE_COOKIE_SECURE),undefined)
export BACK_OFFICE_COOKIE_SECURE
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
	cargo test --bin back_office postgres_tests -- --ignored
