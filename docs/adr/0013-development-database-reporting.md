# 13. Database reporting for the development dashboard

## Status

Accepted

## Date

2026-10-17

## Context

Developers need to compare table populations and PostgreSQL storage usage in
retailer sourcing and object storage. These diagnostics do not belong in the
public object storage client or require scanning large payload tables.

## Decision

Each service owns a reporting module exposing authenticated
`GET /api/reporting/database-tables`, documented in its REST OpenAPI schema.
Both use the same typed JSON contract: active schema, UTC generation timestamp,
`row_count_kind: "estimated"`, and table names, row counts, table bytes, index
bytes, and total bytes.

Reports include ordinary tables in `current_schema()`, including internal bus
and migration tables, and exclude system schemas and views. Live-row estimates
come from `pg_stat_user_tables.n_live_tup`. Table bytes use `pg_table_size`,
index bytes use `pg_indexes_size`, and total bytes are their sum. Requests do
not run COUNT, ANALYZE, or mutate storage.

The development app owns private, typed HTTP clients in its server binary. It
requests both reports concurrently with bounded timeouts, no retries, and no
redirects. `/dashboard` renders HTML with responsive CSS, a manual refresh
control, and independent service error states. Credentials stay on the server.
No public client API, GraphQL field, database migration, or history store is
introduced.

## Consequences

The dashboard remains useful when one service is unavailable. Estimated counts
can lag writes, and the UI labels them accordingly. Byte sizes include physical
PostgreSQL overhead, not just logical payload content. Reports are snapshots,
not atomic measurements across services. The development app retains its
existing access model and is intended for the development environment.
