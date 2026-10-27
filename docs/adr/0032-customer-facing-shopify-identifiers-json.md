# ADR 32: Customer-facing Shopify identifiers in JSONB

Status: Accepted; supersedes ADR 29's three customer-facing Shopify TEXT columns
only. GTIN/SKU, upstream storage, wire contracts and identifier event semantics remain.

Date: 2026-10-26

## Context

Customer-facing projects three optional Shopify identifiers as one logical value.
The storage change must retain existing data and preserve public responses.

## Decision

Store `offers.external_identifiers` as NOT NULL JSONB containing exactly
`shopify_product_gid`, `shopify_product_variation_gid` and `shopify_handle`.
All keys are present and each value is a string or JSON null. The default is the
all-null object. A named CHECK enforces shape with explicit required keys and an
`IS TRUE` predicate; shared Rust validation enforces nonempty, NUL-free strings.
GTIN/SKU remain separate nullable text columns.

A service-local `Identifier(ExternalIdentifiers)` implements Diesel JSONB
encoding/decoding, using guarded shared types and exact object shape checking.
Projection creation and complete identifier replacement bind this type within
the existing original-message receipt transaction. Browse and search share the
same typed record and unchanged API DTO mapping.

Migration 2026-10-26-000005 locks the table, checks invalid legacy empty strings,
copies values with `jsonb_build_object`, validates null-aware equality and drops
the three original columns. Down migration restores the latest JSON values to
TEXT, verifies equality and drops JSONB. Never reset identifiers or replay queues
as a migration strategy.

## Consequences

Use a coordinated customer-facing maintenance window. Both old readers and
writers must stop before migration; old/new binaries cannot share this schema.
Backfill rewrites rows, generates WAL and holds an access-exclusive table lock.
Measure capacity and duration on a representative copy and retain a backup.
Rollback requires stopping new instances, reverting only this migration, then
starting the old binary. Pending messages keep their original identities.

No new JSON index, public version, identifier matching, upstream migration or
historical event change is introduced. Invalid manually written values cause
sanitized read errors rather than silent empty identifiers.
