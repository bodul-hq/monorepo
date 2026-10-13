# ADR 4: Feature slices for retailer_sourcing

- Status: Accepted
- Date: 2026-10-13

## Context

`retailer_sourcing` gained a sitemap discovery pipeline ported from an earlier
MVP. The port kept the MVP's layout: three large files with inline modules, a
repository per table shared by every stage, error types in `main.rs`, and two
central enums (`MvpEvent`, `AppCommand`) that every stage edited. Nothing
started the pipeline, there were no migrations, and the unreachable code kept
`make ci` from passing.

The design note `services/feature-slices-architecture.md` describes a
feature-sliced structure: each use case is a folder that follows command →
handler → event.

## Decision

`retailer_sourcing` follows that note, adapted to the stack it already uses:

- `src/sitemap_discovery/` is a bounded context. Its `mod.rs` holds the domain
  shared by its slices (sitemap documents, the parser, the table
  definitions), and its prelude has a `register()` that registers every slice.
- Each use case is a slice: `retrieve_sitemap`, `process_sitemap` and
  `consolidate_sitemap`. A slice has `command.rs`, `handler.rs`, `event.rs`,
  `store.rs`, and `reacts_to.rs` when an event from another slice starts it.
- A slice owns its queries in `store.rs`. There is no repository shared between
  slices.
- A handler depends on traits defined in its own slice, shaped to what it needs
  (storage, fetching a body, the retailer registry), so it can be tested with
  in-memory fakes. Handlers return events and don't know about the bus.
- A slice may import another slice's events, but never its store or handler.
- The context and each slice expose one public module, `prelude`, and nothing
  else; everything else sits in private modules. It holds `register()`, any HTTP `routes()`, and the events others may
  use. A slice's prelude serves the other slices; the context's prelude is
  what `main.rs` and other contexts use, and exports only the events other
  contexts may react to.
- `src/platform/` holds plumbing that knows nothing about sitemaps: `db.rs`
  (the Postgres pool and embedded migrations), `http.rs` (the fetch client) and
  `bus.rs`.
- The bus is a thin layer over the Mulac kernel, which the pipeline already
  used, rather than the in-process channel the note suggests. Mulac stores
  every command and event in Postgres and delivers each one at least once.
  Handlers stay synchronous and use Diesel, not sqlx.
- `main.rs` boots the bus and runs the pending migrations only when
  `DATABASE_URL` is set. Without it the service serves HTTP only, as before.

## Consequences

- Adding a use case means adding a slice folder and one line in the context's
  `register()`. No central enum or shared repository changes.
- Handlers have unit tests that need no database. Store queries are only
  covered by running the service against Postgres.
- Commands and events are stored as each slice's own struct, under the names
  `RetrieveSitemap`, `SitemapRetrieved`, `ProcessSitemap`, `SitemapProcessed`,
  `ConsolidateSitemap` and `SitemapConsolidated`. Renaming one later orphans
  queued entries, so a rename needs a migration of the stored rows.
- The service now owns Mulac's tables through its own migrations. Upgrading
  Mulac may need a new migration here.
- The note's cross-cutting rules aren't applied yet: idempotent commands with
  deterministic ids, retryable versus permanent errors, transactional writes
  and a tracing span per handler. Every handler error is still reported to
  Mulac as a storage error and retried.
- Nothing starts a retrieval yet. A trigger, such as a GraphQL mutation or a
  schedule, is still to be decided.
