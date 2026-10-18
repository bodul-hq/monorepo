# ADR 11: Raw sitemap bodies in object storage

- Status: Accepted
- Date: 2026-10-15
- Supersedes: ADRs 7 and 9's local raw sitemap payload layout and ADR 10's raw sitemap storage metadata

## Context

Raw sitemap bodies should use the common object storage service while local
records retain crawl metadata and a typed reference to each stored body.

## Decision

Use `object_storage_client` over REST with bucket `sitemaps_raw`. Allocate the
local raw sitemap UUID before uploading and send it as the idempotency key,
which object storage persists as `external_id`. Save the returned record UUID
and the body's UTF-8 byte length in `sitemaps_raw.storage`, through the typed
`ObjectStorageMetadata { content_size: u64, object_storage_id: Uuid }` struct.
Expose the object ID in retrieval and processing entities.

Upload all bodies before inserting the batch of metadata in one PostgreSQL
transaction. Publish the retrieved event only after this succeeds. Do not hold
database connections during HTTP calls. The synchronous bus uses a Tokio
runtime handle to drive the async client from blocking workers. Require
`OBJECT_STORAGE_URL` and `OBJECT_STORAGE_TOKEN` whenever the pipeline database
is configured; HTTP-only startup remains available without them.

Processing, source inspection, and sitemap export resolve bodies through the
client. Validate returned record ID, bucket, external ID, and byte length
against the local reference. Missing or invalid objects fail the operation.
Keep the existing API and export formats. Preserve the legacy `sitemap_raw`
bucket in object storage while allowing `sitemaps_raw`.

Use a fresh-data migration: delete existing raw sitemap rows and drop
`raw_sitemap_documents_data`. There is no backfill or legacy fallback.
Processed, consolidated, and offer data remain intact. Rollback recreates an
empty payload table only when no new raw sitemap rows exist.

## Consequences

- Raw sitemap retrieval and reads now depend on object storage availability.
- Failed uploads or local commits can leave immutable unreferenced objects.
  No distributed transaction or automatic cleanup is introduced.
- Retrying an upload with the same local ID is idempotent. New retrievals
  allocate new IDs; this change does not add durable replay of upload batches.
- Old source bodies are irreversibly removed by migration. Fresh retrievals
  are required before inspecting or exporting sources for existing crawls.
- Fixture tests cover failure boundaries and metadata validation. An opt-in
  integration test checks bodies and identifiers against the real service.
