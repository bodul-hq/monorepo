include ../../make/rust.mk

# Preserve shell/command-line overrides, including explicitly empty values.
ifneq ($(origin DATABASE_URL),undefined)
customer_facing_database_url := $(value DATABASE_URL)
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
customer_facing_test_database_url := $(value DATABASE_URL_TEST)
endif
ifneq ($(origin PORT),undefined)
customer_facing_port := $(value PORT)
endif
include $(wildcard .env)
ifneq ($(origin customer_facing_database_url),undefined)
DATABASE_URL := $(customer_facing_database_url)
endif
ifneq ($(origin customer_facing_test_database_url),undefined)
DATABASE_URL_TEST := $(customer_facing_test_database_url)
endif
ifneq ($(origin customer_facing_port),undefined)
PORT := $(customer_facing_port)
endif
ifneq ($(origin DATABASE_URL),undefined)
export DATABASE_URL
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
export DATABASE_URL_TEST
endif
ifneq ($(origin PORT),undefined)
export PORT
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

# Every case creates/drops its own schema; existing database tables are untouched.
test-postgres:
	@test -n "$$DATABASE_URL_TEST" || { echo "DATABASE_URL_TEST is required; use a separate test database"; exit 1; }
	cargo test --bin customer_facing postgres_tests -- --ignored
