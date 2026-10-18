# ADR 12: Remaining pipeline payloads in object storage

- Status: Accepted
- Date: 2026-10-16
- Supersedes: ADRs 7–10's remaining local payload tables and storage metadata shapes

## Context

Raw sitemap bodies already use object storage. Processed and consolidated
sitemaps and the four offer stages should use the same client and validation
boundary. Offer rows currently preserve their IDs across content replacements,
while object storage rejects changed content under the same external ID.

## Decision

Use buckets matching the seven metadata table names. Add `sitemaps_processed`
and `sitemaps_consolidated` to the allowlist, retaining legacy singular buckets.
Sitemaps use their row UUIDs as external IDs and typed storage metadata containing
`content_size: u64` and `object_storage_id: Uuid`.

Offers retain their local row IDs and uniqueness by `offer_id`. Each write
allocates a fresh external UUIDv7 and saves it in typed `OfferStorageMetadata`
alongside content size and object ID. This supports immutable payload versions
without changing the existing upsert identities or concurrency behavior.

Upload raw bodies unchanged and serialize JSON once into compact text. Measure
the exact UTF-8 bytes uploaded. Validate existing size/count bounds before HTTP.
Upload before the local write, release database connections before HTTP, and
atomically write the complete reference with stage metadata. Only successful
writes produce success events. Stored entities and loaded inputs expose object
IDs; commands, events, source API responses, and export formats remain compatible.

Every consumer resolves the object and validates ID, bucket, external ID, and
byte length before parsing JSON. Retain fixed metadata structs; retailer-specific
payload JSON retains its existing representation.

Use a fresh-data cutover: delete metadata from the six affected stages and drop
their payload tables. Preserve raw sitemaps and bus history. Rollback only
recreates empty payload tables, refusing when any of the six stages has rows.
Deploy the bucket allowlist before restarting sourcing and initiating fresh work.

## Consequences

- All pipeline payload reads and writes require object storage availability.
- Failed local writes and replaced offer payloads leave immutable objects behind.
  No distributed transaction, cleanup API, or durable upload replay is introduced.
- Concurrent offer replacements retain the last committed complete reference.
  Retrying a prepared upload is idempotent; another store call generates a new
  external ID, even for unchanged content.
- Existing data in the six stages is irreversibly removed; queued historical
  commands may reference missing data. The migration does not rewrite bus history.
- Fixture and PostgreSQL tests cover failures, concurrency, migrations, APIs,
  and exports. Opt-in live tests cover all seven stages using a real test-backed
  service, with exact payload and reference comparisons.
