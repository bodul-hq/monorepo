include ../../make/rust.mk

# Local settings; a DATABASE_URL already set in the environment wins.
ifndef DATABASE_URL
-include .env
endif
export DATABASE_URL

.PHONY: serve migrate migrate-redo require-database-url

serve:
	cargo run

# Applies pending migrations to DATABASE_URL.
migrate: require-database-url
	diesel migration run

# Reverts and reapplies every migration on DATABASE_URL, which deletes its data.
migrate-redo: require-database-url
	diesel migration redo --all

require-database-url:
	@test -n "$(DATABASE_URL)" || { echo "DATABASE_URL must be set, e.g. in .env (make .env)"; exit 1; }

.env:
	cp .env.example .env
