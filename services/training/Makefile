include ../../make/rust.mk

ifneq ($(origin DATABASE_URL),undefined)
training_DATABASE_URL := $(value DATABASE_URL)
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
training_DATABASE_URL_TEST := $(value DATABASE_URL_TEST)
endif
ifneq ($(origin PORT),undefined)
training_PORT := $(value PORT)
endif
ifneq ($(origin TRAINING_REPORTING_TOKEN),undefined)
training_TRAINING_REPORTING_TOKEN := $(value TRAINING_REPORTING_TOKEN)
endif
include $(wildcard .env)
ifneq ($(origin training_DATABASE_URL),undefined)
DATABASE_URL := $(training_DATABASE_URL)
endif
ifneq ($(origin DATABASE_URL),undefined)
export DATABASE_URL
endif
ifneq ($(origin training_DATABASE_URL_TEST),undefined)
DATABASE_URL_TEST := $(training_DATABASE_URL_TEST)
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
export DATABASE_URL_TEST
endif
ifneq ($(origin training_PORT),undefined)
PORT := $(training_PORT)
endif
ifneq ($(origin PORT),undefined)
export PORT
endif
ifneq ($(origin training_TRAINING_REPORTING_TOKEN),undefined)
TRAINING_REPORTING_TOKEN := $(training_TRAINING_REPORTING_TOKEN)
endif
ifneq ($(origin TRAINING_REPORTING_TOKEN),undefined)
export TRAINING_REPORTING_TOKEN
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
	cargo test --bin training postgres_tests -- --ignored
