# Item 21: Fuzzy offer title search

Status: completed; database, live, browser and CI acceptance checks passed.

## Goal and confirmed scope

Expose title search in customer-facing and add a centered search form to the
customers header, submitting to a server-rendered `/search` page. The user
confirmed typo-tolerant results ranked by relevance.

Apart from the search text, the endpoint accepts only `limit` and `offset`.
Do not add caller-selected sorting, similarity thresholds, filters, cursors,
total counts or autocomplete. Search uses customer-facing's existing offer
projection, without calling upstream services or running queues. Keep the
customers app HTML/CSS-only.

## Starting behavior

Reviewed against monorepo `ed55cb1eb34740ef1e2ace2a8ba0174a8a078443`, after
items 17–20 were implemented. This document remains an implementation plan;
search has not been added to the service or app.

- The [read service](../../services/customer_facing/src/offer_read.rs) shares
  one bounded read path between REST and GraphQL. It returns `OfferPage`, fetches
  one extra row for `has_more`, and loads ordered images without multiplying
  offer rows. Browse results sort by creation time and ID.
- Its current offer DTO includes `scraped_at`, `localization`, `gtin`, `sku`
  and the nested `external_identifiers` object in addition to the original
  fields and ordered images. The PostgreSQL `Record` currently reconstructs
  that object from three nullable Shopify columns. Search must reuse this
  mapping, including localization validation and exact decimal price strings.
- [GraphQL and route composition](../../services/customer_facing/src/main.rs)
  expose `offers` and health. The only existing browse index is
  [creation time and ID](../../services/customer_facing/migrations/2026-10-24-000003_offer_browsing/up.sql).
- The [customers client](../../apps/customers/src/offers.rs) fetches 30 offers
  with a five-second timeout, preserves configured URL prefixes and validates
  response pagination. The [app](../../apps/customers/src/main.rs) serves `/`
  only; its [page renderer](../../apps/customers/src/page.rs) assumes browse
  URLs and a header containing a brand and a note.
- The app client already validates typed product identifiers and localization;
  it does not consume every service field, such as `scraped_at`.
  Search should use the same `Page`/`Offer` types and validation as browsing,
  without adding new offer facts to cards.
- Stored titles retain their original content. HTML cleanup happens in the
  [display helper](../../apps/customers/src/page/title.rs). This plan searches
  persisted titles; it does not rewrite them or introduce a separately cleaned
  search projection. Returned titles continue through existing display cleanup.

## Proposed API contract

| Surface | Request |
| --- | --- |
| REST | `GET /api/offers/search?q=charger&limit=30&offset=0` |
| GraphQL | `searchOffers(q: String!, limit: Int! = 30, offset: Int! = 0): OfferPage!` |
| Customers | `GET /search?q=charger&offset=0`, using a fixed upstream limit of 30 |

Use the same query validation and search implementation for REST and GraphQL.
Reject NUL and non-whitespace control characters before normalization. Trim
surrounding Unicode whitespace and collapse internal whitespace runs to one
ASCII space. Require 1–200 Unicode scalar values after normalization and at
least one alphanumeric character. Reject missing/blank query text at the service
boundary. Treat quotes, percent signs, underscores and
other punctuation as text, never SQL, wildcard or regular-expression syntax.
Do not translate, stem or silently correct the submitted query.

Reuse browse pagination defaults and bounds: limit 1–100; offset
0–2147483647. Return the existing `items`, `limit`, `offset`, `has_more` shape
(`hasMore` in GraphQL), preserving every current offer field and its type.
Scores are internal. In particular, retain nullable scrape time, localization,
GTIN/SKU and all three nullable external identifier values in REST and GraphQL;
the search addition does not change those contracts. Empty and past-the-end
results return a successful empty page. Invalid input yields REST 400 or a
GraphQL error; database/blocking-task
failures yield sanitized REST 500 or GraphQL errors, never an empty success.

Search ordering is fixed: a case-insensitive full-title equality match first,
then word-similarity score descending, then `created_at DESC, id DESC` for ties.
Fetch `limit + 1` after filtering and ordering; offset applies to that ranked
set. Pagination is deterministic for unchanged data, but concurrent inserts or
title changes can move results between requests. No snapshot guarantee is added.

## Search implementation and database requirements

Use PostgreSQL `pg_trgm` with an index-supported word-similarity predicate.
It supports matching a query against part of a longer title, with GIN/GiST
indexes available for the similarity operators. See the
[PostgreSQL 16 reference](https://www.postgresql.org/docs/16/pgtrgm.html).

Proposed implementation choices:

- Add a GIN index on the same `lower(title)` expression used in the predicate.
  Select candidates using `lower(query) <% lower(title)` and rank them using
  `word_similarity(lower(query), lower(title))`. Bind query, limit and offset
  as parameters. Do not score/load all offers into Rust or paginate before
  applying the relevance order.
- Fix `pg_trgm.word_similarity_threshold` at 0.5 for this first version and
  freeze representative relevance fixtures before shipping. Apply it with
  `SET LOCAL` in the same transaction/connection as the query so pooled
  connection settings cannot leak between requests. Do not rely on server
  defaults or accept a threshold parameter from clients. Record any tuning
  in the plan/ADR and expected-result fixtures.
- Use the existing `spawn_blocking` boundary for Diesel. Select the ranked
  offer page before attaching ordered images; share existing offer-to-DTO
  mapping so current and future fields appear consistently in browse/search.
  Preserve the rank columns through the page subquery and repeat the full
  relevance ordering in the outer image-loading query; SQL subquery ordering
  alone does not define the returned order. Set a three-second transaction-local
  statement timeout for the search SQL, below the customers five-second HTTP
  timeout, and map timeout failures through the normal sanitized error path.
- Short and common queries can return many candidates; test one/two-character
  queries rather than assuming three characters are required. Punctuation is
  not pattern syntax; `pg_trgm` ignores non-word characters when extracting
  trigrams, so punctuation is not a literal-match requirement either. No accent
  folding, language-specific stemming or guarantee of every possible misspelling
  is implied by this initial ranking.

Provision `pg_trgm` once per database in a stable `customer_facing_extensions`
schema before the application migration. Installation requires database `CREATE`
privilege for this trusted extension; the server must have it available.
[Extension documentation](https://www.postgresql.org/docs/16/sql-createextension.html).
Keep extension provisioning separate from per-test-schema migrations.

Use that fixed trusted schema for extension functions/operators/operator classes:
`customer_facing_extensions.word_similarity`,
`OPERATOR(customer_facing_extensions.<%)` and
`customer_facing_extensions.gin_trgm_ops`. Document creating the schema, installing
the extension there and granting schema usage to the runtime/test roles; do not
grant untrusted roles schema creation rights. Check the installed extension's
schema before migration: `CREATE EXTENSION IF NOT EXISTS` does not relocate an
existing installation. A differently located installation needs an explicit
operator-managed provisioning step, not an application startup relocation.
The isolated PostgreSQL tests currently set their search path to the generated
schema only; extension resolution must
work there without moving or reinstalling the extension. Do not interpolate
request-controlled schema names. A missing extension is a clear deployment
failure, not a silent fallback to different search semantics.

The additive migration builds the index over existing rows, so existing offers
become searchable without event replay. Its down migration removes only the
service's index and leaves the database-wide extension intact. Document index
build locking and test a populated table; do not put `CREATE INDEX CONCURRENTLY`
inside the default transactional migration. Preserve projection data, receipts
and queues throughout migration and rollback.

## Customers flow

1. Add an accessible GET form to the shared header, with `action="/search"`,
   an input named `q`, a label and a submit button. Center it within the desktop
   header using a three-column layout rather than centering only the remaining
   space between unequal side elements. On narrow screens allow a full-width
   second row without horizontal overflow. Preserve the brand/home link and
   existing light/dark styles.
2. Register `/search` in `app()`. A missing/blank query renders a 200 search
   prompt without calling the service when offset is valid. Validate offset
   even on this prompt; malformed, negative or overflowing offsets render HTML
   400. Other invalid input also renders HTML 400 with an actionable message.
   Keep entered text in the form; validate on the server even if HTML constraints
   are present.
3. Extend the client with a search method, reusing its HTTP client, timeout,
   response validation and configured base-path handling. Build URLs with
   query-pair encoding. Each valid page makes one server-side request; the
   browser does not contact the API.
4. Reuse offer cards, image selection, cleaned titles, prices and safe new-tab
   links. Give search its own heading and relevance label; do not label ranked
   results “Newest first”. Include the escaped query in the heading/form.
   Provide distinct first-page no-results and past-the-end states.
5. Preserve normalized query text in Previous/Next and retry URLs. Escape HTML
   after URL encoding, including ampersands, quotes, Unicode and plus signs.
   New form submissions omit offset so they restart at page zero. Keep existing
   checked offset arithmetic and current-tab pagination.
6. Service failures/timeouts/malformed responses render HTML 503 with a retry
   URL preserving query and offset. Keep `Cache-Control: no-store`, keyboard
   focus visibility and semantic search/navigation landmarks. Do not inject
   query-based highlights or raw HTML.

## Implementation sequence and documentation

First add the database prerequisite/index and shared search repository path,
then REST/GraphQL adapters and their tests. Next extend the customers client,
page route, reusable rendering and header form. Keep homepage browsing intact.

Expected implementation touch points:

| Component | Change |
| --- | --- |
| `services/customer_facing/src/offer_read.rs` | Extend `Repository` with search, add `Offers::search`, share pagination validation and record/page mapping, and add the OpenAPI route. Ensure the response documentation distinguishes browse and relevance ordering. |
| `services/customer_facing/src/main.rs` | Add `Query::search_offers`, using the same read service and errors as REST. |
| `services/customer_facing/migrations/` | Add a new title GIN index migration after the existing scrape-time, localization and identifier migrations; leave accepted migrations unchanged. |
| `services/customer_facing/src/offer_read/tests.rs` | Update repository doubles; cover input, defaults, REST/GraphQL parity and sanitized failures. Keep PostgreSQL relevance assertions in the database suite. |
| `services/customer_facing/src/postgres_tests.rs` and `src/postgres_tests/` | Register search tests and document/check the extension prerequisite for all migrations run in isolated schemas. |
| `apps/customers/src/offers.rs` and `src/offers/tests.rs` | Add a search endpoint/method while sharing transport and response validation with `fetch`; preserve configured URL prefixes. |
| `apps/customers/src/main.rs`, `src/page.rs`, `src/style.css` and `src/page/tests.rs` | Register the route; parameterize the header, page metadata, headings and pagination URLs; reuse cards; test the form and every route state. |
| `bruno/customer_facing/` and READMEs | Add REST/GraphQL search examples and document extension setup and query semantics. |

Use the existing Diesel, Poem, async-graphql and reqwest dependencies; no new
Cargo dependency is expected. Keep service and app query normalization aligned
with matching boundary fixtures; the service remains authoritative. Do not
move PostgreSQL search into a shared crate solely to reuse these few input rules.

Add an indexed ADR for PostgreSQL fuzzy matching, fixed relevance ordering and
the service/app boundary, extending
[ADR 23](../adr/0023-offer-browsing-and-customers-homepage.md) without editing
accepted ADRs. Update the root requirements for `pg_trgm`, both service
and app READMEs, OpenAPI/GraphQL examples and Bruno search requests. Document
query rules, raw-title matching, ranking, pagination limits and deployment.

Provision the extension and deploy customer-facing before customers uses its
new endpoint. Rolling back the app can leave the additive service route/index
in place. No upstream event, export schema, queue or source data change is
needed. Items 17–20 are already present in the baseline. The remaining locale
normalization and JSON identifier-storage items are not search prerequisites:
preserve their public DTOs and keep search/browse on one mapping. If the identifier
storage work lands first, select its JSON column in both read paths instead of
adding another three-column mapping. Allocate distinct new migration versions
when implementing these items concurrently.

## Acceptance and verification

- Relevance fixtures cover exact title, exact word within a long title,
  partial word, representative misspellings, multiword queries, mixed case,
  Unicode, short terms and unrelated titles. Verify relevant results rank
  ahead of weaker matches and exact-title matches lead. Test tied scores and
  timestamps with stable ID ordering. Document raw markup behavior separately
  from safe cleaned card rendering.
- Test query normalization and length boundaries, missing/blank/punctuation-only
  queries, controls, malformed pagination, SQL-like input and maximum offset.
  REST and GraphQL must share results, errors and defaults.
- PostgreSQL tests verify index up/down over existing data, stable extension
  resolution across isolated schemas, creation and title-update visibility,
  receipt replay, 30/1/empty pagination and ordered images. A changed projected
  title must leave the old search and enter the new one without a search worker.
- Compare browse/search serialization for the same offer, including populated
  and absent scrape time, localization, GTIN/SKU and external identifiers. Test
  legacy empty images/default localization/default identifiers in app responses
  and preserve the existing invalid identifier/localization rejection behavior.
- Prove transaction-local threshold and timeout settings do not leak through
  reused pooled connections, including after a failed search. Verify a missing
  or wrongly located extension prevents migration/startup clearly, rather than
  returning an empty search response or reinstalling it inside a test schema.
- On a representative larger fixture, record `EXPLAIN (ANALYZE, BUFFERS)` and
  latency for selective, common, typo and short queries. Verify the candidate
  predicate can use its index and respects the timeout; do not demand index
  scans on tiny tables where PostgreSQL may reasonably choose a sequential scan.
- Client tests cover encoded query strings, configured path prefixes, fixed
  page size, timeout/HTTP errors and malformed responses. Page tests cover
  `/search` states, query persistence, pagination/retry links, form reset,
  escaping and unchanged homepage/card behavior. Extend route smoke tests.
- Run `make fmt` then `make ci` in changed folders, or at the root, and run
  customer-facing's isolated PostgreSQL suite with the extension provisioned.
  Audit only if Cargo dependencies/lockfiles change. Start service and app with
  `PORT=<port> make serve`; verify health, REST/GraphQL search, `/` and `/search`.
  Use a local fixture to check typo results and pagination end to end. Review
  desktop/tablet/320px layouts, both themes and keyboard-only form navigation.

Record actual revisions, relevance examples, migration/query-plan results,
CI/database checks and live/browser evidence before marking the item complete.

## Implementation verification (2026-10-26)

Implemented migration `2026-10-26-000004_offer_title_search`, shared REST/GraphQL
search, the customers route/form, docs and Bruno requests. No Cargo dependency
or lockfile changes. Threshold remains 0.5.

- Customer-facing `make fmt` and `make ci`: pass (33 tests, 23 opt-in ignored).
- Customers `make fmt`, format check and clippy: pass. Tests: 21 pass; six
  TCP fixture tests fail at listener creation with `Operation not permitted`.
  Rendering, query validation and prompt-route tests pass.
- Root CI was also attempted: existing inbox-client TCP tests hit the same
  listener restriction. No tests were disabled to mask it.
- `make test-postgres`: blocked because `DATABASE_URL_TEST` is not configured;
  Docker socket access is denied. Database relevance expectations, up/down,
  receipt replay and connection-setting tests compile but were not executed.
- Live service/app startup was attempted on ports 39122/39121. Service requires
  DATABASE_URL; app cannot bind in this sandbox. Health/browser verification
  and representative EXPLAIN/latency measurements remain outstanding.

That initial run left the TODO unchecked. The completion run below resolves
those environment restrictions using disposable databases and approved local networking.

## Completion verification (2026-10-26)

Verified the working tree at `cc57884` with PostgreSQL 16 in a disposable
container. Root `make fmt` followed by `make ci` passes, including socket tests.
Customer-facing's `make test-postgres` passes all 29 cases, including relevance,
30/1/empty pagination, raw-title matching, ordered images, title updates, receipt
replay, index up/down and pooled-connection settings after success/failure.
Missing and wrongly located pg_trgm installations both fail with the explicit
provisioning error. No existing development database was changed.

Live servers started with `PORT=<port> make serve` on 39200–39204 and passed
health/page checks. REST/GraphQL and the customers page accepted fresh pipeline
offers; `Ankre` matched Anker titles. Thirty-one additional offers submitted
through the inbox produced 30, 1 and 0 results for `chargre` at offsets 0, 30
and 60. An exclusive table lock forced the three-second search timeout; the API
returned a sanitized 500 and recovered after the lock was released.

Chromium checks passed at 1440, 768 and 320 pixels in light and dark themes:
centered desktop form, no horizontal overflow, keyboard focus/Enter submission,
new-query offset reset, Previous/Next links, no-store responses and no scripts.
Desktop and mobile screenshots were also reviewed. External image requests were
blocked for these layout checks; pipeline image URLs were verified separately.

The production SQL shape was measured with `EXPLAIN (ANALYZE, BUFFERS)` against
50,000 synthetic offers in an isolated schema, with threshold 0.5 and the same
three-second timeout. All five predicates used the lower-title GIN index:

| Query | Fixture match | Execution time |
| --- | --- | ---: |
| Quasar | Selective, 100 candidates | 1.265 ms |
| power | Common, 24,900 candidates | 129.137 ms |
| chargre | Typo, 100 candidates | 0.897 ms |
| p | One character, 24,900 candidates | 99.534 ms |
| po | Two characters, 24,900 candidates | 102.419 ms |

These are local fixture measurements, not production latency guarantees.
No dependencies or lockfiles changed; no audit was required.
