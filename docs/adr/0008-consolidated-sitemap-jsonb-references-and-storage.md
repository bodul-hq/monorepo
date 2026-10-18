# ADR 8: Consolidated sitemap JSONB references and storage

- Status: Accepted
- Date: 2026-10-15
- Supersedes: ADR 7's separate content-size column and retained processed-sitemap foreign key

## Context

ADR 7 separates domain entities from PostgreSQL records and groups counts in
`stats`. Consolidated sitemap records also need grouped source references and
storage information, without duplicating those values in standalone columns.

## Decision

`ConsolidatedSitemapPgRecord` stores three JSONB objects:

- `references`: `processed_sitemap_id`, `retrieval_id`, and `retailer_code`.
- `storage`: `content_size`, the byte length of the serialized grouped links.
- `stats`: the five existing counts.

UUID references are JSON strings. Size and counts remain JSON integers. The
entity and event fields remain unchanged; conversions at the store boundary
create the JSON objects. `id` and `consolidated_at` remain regular columns.

References exist only in JSONB, with no generated columns. A unique expression
index on the processed-sitemap reference cast to UUID retains one consolidated
record per processed sitemap. The previous foreign key to `processed_sitemaps`
and its cascading deletion are removed. The payload table's foreign key to
`consolidated_sitemaps` remains intact.

The sitemap export joins processed and consolidated records using the UUID
extracted from `references`. Its output format remains unchanged.

## Consequences

- Deleting a processed sitemap leaves the consolidated record and its payload
  intact. PostgreSQL no longer requires that the referenced processed row exists.
- A new transactional migration backfills both JSONB objects and drops the four
  original columns without discarding records, stats, timestamps, or payloads.
- The down migration restores those columns, uniqueness, and the original
  cascading foreign key. Missing or invalid values, out-of-range sizes, or
  dangling processed references fail rollback transactionally; operators must
  resolve such data before downgrading.
- Deployment requires stopping the old service before migration because its
  insert and export queries expect the old columns. No API or event changes
  are needed.
