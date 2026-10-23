# testimony_infra

Concrete event stores for [`testimony`](../testimony/README.md): an in-memory
store for tests and a PostgreSQL store using Diesel connection pools. Includes
database record types for events and JSON metadata, and an `event_store` table
migration under `migrations/`.

Import common types from `testimony_infra::prelude`. PostgreSQL consumers supply
a connection pool and apply the migration before using the store. Building this
crate requires libpq; the unit tests do not require a running database.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

## Caller-owned transactions

`ConnectionEventStore::new(&mut connection)` implements the same `EventStore`
contract as `PostgresEventStore`, using the caller's Diesel connection. Use it
inside `connection.transaction(...)` when event history must commit together
with identity rows, receipts, or other application writes. Both adapters share
the serialization and SQL implementation. The caller controls transaction
boundaries; a scoped adapter does not commit or publish events on its own.
Custom table names must be trusted SQL identifiers.
