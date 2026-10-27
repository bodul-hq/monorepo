# ADR 19: Immutable offer export preparation

- Status: Accepted
- Date: 2026-10-24
- Supersedes: ADR 12's unchanged structuralization event shape, and its
  upload-before-local-write and no-durable-upload-replay policy for exports only

## Context

Structuralized offer metadata points to the latest immutable object. A delayed
event containing only an offer ID cannot identify the document that produced it.
Export delivery also needs stable entry IDs when commands or event publication
are retried. Allocating a fresh upload key on each attempt cannot provide that.

## Decision

New `OfferWasStructuralized` events carry the complete typed immutable storage
reference. The new `ProcessOfferForExport` command uses that reference, never a
lookup of the latest offer row. Existing events still deserialize, but export
commands without a reference fail with an instruction to run fresh
structuralization. No historical snapshot is inferred from current metadata.

The `processing_for_export` feature converts each priced variation into the
existing offers `ImportOfferDetails` payload. It preserves retailer enum
serialization, title, exact URL and integer money. Missing prices are recorded
as exclusions. Malformed required values, blank titles, negative prices,
invalid HTTP(S) URLs and repeated exact variation URLs fail preparation.
Availability, images and retailer identifiers do not change the import contract.

Persist an `offer_export` preparation before HTTP upload, uniquely keyed by
sourcing offer ID, source object ID and schema version. It contains stable
UUIDv7 snapshot/entry IDs and the exact pending UTF-8 body. Concurrent prepares
use the winning row. Upload to the `offer_export` bucket with the snapshot ID
as idempotency key, then atomically replace pending bytes with a typed object
reference and completion time. Retry reuses persisted bytes and IDs; completed
snapshots are read and validated from object storage.

Only completed storage returns `OfferExportWasPrepared`, containing the export
ID, source identity/reference and schema version. Event envelopes can still
duplicate after a queue failure; later outbox subscribers must deduplicate by
stable export entry ID. This change adds neither a subscriber nor a publisher.
An empty or wholly excluded document is a completed zero-entry export.

## Consequences

The additive migration preserves pipeline and queue history. Deploy object
storage's new bucket allowlist before sourcing. Pending export bodies temporarily
live in PostgreSQL so upload acknowledgement or completion failures are
recoverable; completed bodies live only in object storage. Connections are
released before HTTP. Rolling back removes export metadata but leaves immutable
objects. Other pipeline stages retain ADR 12's existing replacement semantics.

Version 1 is a fixed conversion contract. Changing its output requires a new
schema version. Older pending rows retain their exact bytes, and later source
objects create distinct exports. No atomic transaction spans PostgreSQL, object
storage or Mulac's event publication.
