# ADR 7: Domain entities and PostgreSQL records

- Status: Accepted
- Date: 2026-10-15

## Context

ADR 4 keeps persistence within each feature slice. Consolidation already had
separate handler and Diesel structs, but its store calculated domain counts
and represented each count as an integer column. Domain behavior and database
layout need to evolve independently.

## Decision

Within a slice, `entity.rs` holds domain entities and `record.rs` holds
PostgreSQL representations. Every persistence type ends in `PgRecord`, such as
`ProcessedSitemapPgRecord` and `ConsolidatedSitemapPgRecord`. Use a `New` prefix
only when insertion needs a separate type from the read representation.

The store's domain operations accept entities for writes and IDs for reads,
and return entities when loading. Database records never cross that boundary:

- Write: entity → `PgRecord` values → database transaction.
- Read: ID → query → `PgRecord` values → entity.

Explicit, fallible conversions live alongside records. They handle database
representations, serialization, and storage bounds. Entities have no Diesel
annotations or knowledge of JSONB columns. Queries stay in `store.rs`, and
records remain private to the slice; this adds no shared repository.

The first application is `consolidate_sitemap`, including its processed-sitemap
read. `ConsolidatedSitemap` owns five counts as direct Rust fields, calculated
from grouped links at construction and accessible through read-only methods.
Events and logs use those values. The PostgreSQL record groups them in
`stats JSONB` under `product_count`, `catalog_count`, `content_count`,
`not_interested_count`, and `unknown_count`. `content_size` remains a column,
and grouped links remain in `consolidated_sitemaps_data`.

## Consequences

- Domain counts and the PostgreSQL layout can change independently through
  explicit mappings. Existing event fields and payload consumers are unchanged.
- Counts retain the previous checked `i32` storage range so the schema can be
  downgraded. The domain continues using `usize`.
- A new transactional migration backfills stats without losing existing rows;
  its down migration restores the previous columns. Old and new binaries cannot
  share the changed schema during deployment.
- Unit tests cover entities and conversions. Opt-in PostgreSQL tests cover the
  store boundary, rollback, and populated migration round trips in isolated schemas.
- This complements ADR 4. Other slices are unchanged and can adopt the same
  convention when their persistence is next changed.
