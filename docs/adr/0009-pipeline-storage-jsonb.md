# ADR 9: Pipeline storage metadata in JSONB

- Status: Accepted
- Date: 2026-10-15
- Supersedes: ADR 7's standalone size columns for raw and processed artifacts

## Context

ADR 8 groups consolidated sitemap content size in `storage`. The other sitemap
and offer-detail stages need the same representation so artifact metadata has
a consistent location throughout the pipeline.

## Decision

All seven main pipeline tables have a non-null `storage JSONB` column:
`sitemaps_raw`, `sitemaps_processed`, `sitemaps_consolidated`,
`offer_details_raw`, `offer_details_extracted`, `offer_details_normalized`, and
`offer_details_structuralized`. Their write records contain `storage` as a JSON
value, with `content_size` as an integer byte count.

Raw sizes move from `body_size` to `storage.content_size`. Processed sitemap
sizes move from `document_size` to the same JSON key. The standalone columns
are removed. Consolidated sitemap storage retains its existing representation.
Separate payload tables and domain/event fields remain unchanged, including
the sitemap library's `RawSitemapDocument.body_size` field.

New writes count UTF-8 bytes of raw text or compact serialized JSON. Extraction,
normalization, and structuralization now calculate sizes as part of their
record conversions. Upserts update storage metadata in the same transaction as
the payload, preserving the existing record identity and atomic rollback.
Existing size bounds remain for stages that previously used an `INT` column;
the three offer JSON stages use the serialized document's byte length without
introducing an integer-column limit. Raw sitemap reads reject missing,
non-integer, negative, and out-of-range content sizes.

## Consequences

- Migration 10 copies existing size values exactly. For offer JSON stages that
  previously had no size, it measures compact JSON from the stored JSONB value,
  preserving whitespace inside strings and PostgreSQL's number representation.
  Missing payloads fail the migration transactionally rather than receiving a
  fabricated size. Payloads, IDs, timestamps, and relationships are preserved.
- Stop old service instances before migrating; their queries require the
  removed columns. Startup applies pending migrations, or use `make migrate`.
- Downgrade restores the previous integer columns and drops the six new storage
  columns. Missing, malformed, or overflowing sizes abort it transactionally.
  Storage metadata introduced in the three offer JSON stages is discarded on
  downgrade and recalculated on upgrade; payloads are retained.
- PostgreSQL tests cover populated upgrade/downgrade/re-upgrade, failed upgrade
  and downgrade transactions, UTF-8 sizes, and synchronized upserts with rollback.
