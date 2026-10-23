include ../../make/rust.mk

# Shell/command-line values take precedence, including explicitly empty ones.
ifneq ($(origin DATABASE_URL),undefined)
offers_database_url := $(value DATABASE_URL)
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
offers_test_database_url := $(value DATABASE_URL_TEST)
endif
ifneq ($(origin PORT),undefined)
offers_port := $(value PORT)
endif
ifneq ($(origin OFFERS_REPORTING_TOKEN),undefined)
offers_reporting_token := $(value OFFERS_REPORTING_TOKEN)
endif
ifneq ($(origin AMQP_URL),undefined)
offers_amqp_url := $(value AMQP_URL)
endif
ifneq ($(origin OUTBOX_EXCHANGE),undefined)
offers_outbox_exchange := $(value OUTBOX_EXCHANGE)
endif
include $(wildcard .env)
ifneq ($(origin offers_amqp_url),undefined)
AMQP_URL := $(offers_amqp_url)
endif
ifneq ($(origin offers_outbox_exchange),undefined)
OUTBOX_EXCHANGE := $(offers_outbox_exchange)
endif
ifneq ($(origin AMQP_URL),undefined)
export AMQP_URL
endif
ifneq ($(origin OUTBOX_EXCHANGE),undefined)
export OUTBOX_EXCHANGE
endif

ifneq ($(origin offers_database_url),undefined)
DATABASE_URL := $(offers_database_url)
endif
ifneq ($(origin offers_test_database_url),undefined)
DATABASE_URL_TEST := $(offers_test_database_url)
endif
ifneq ($(origin offers_port),undefined)
PORT := $(offers_port)
endif
ifneq ($(origin offers_reporting_token),undefined)
OFFERS_REPORTING_TOKEN := $(offers_reporting_token)
endif
# Export only defined values. Exporting an unset name manufactures an empty
# variable, which would change "not configured" into a startup error.
ifneq ($(origin DATABASE_URL),undefined)
export DATABASE_URL
endif
ifneq ($(origin DATABASE_URL_TEST),undefined)
export DATABASE_URL_TEST
endif
ifneq ($(origin PORT),undefined)
export PORT
endif
ifneq ($(origin OFFERS_REPORTING_TOKEN),undefined)
export OFFERS_REPORTING_TOKEN
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
	cargo test --bin offers postgres_tests -- --ignored

.PHONY: test-rabbitmq
test-rabbitmq:
	@test -n "$$AMQP_URL_TEST" || { echo "AMQP_URL_TEST is required"; exit 1; }
	cargo test rabbitmq_confirms_delivery -- --ignored
