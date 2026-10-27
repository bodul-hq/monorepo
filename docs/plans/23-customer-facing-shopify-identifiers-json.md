# Item 23: Customer-facing Shopify identifiers as JSON

Status: completed; PostgreSQL preservation, live verification and CI passed.

## Goal and confirmed scope

Replace the three Shopify identifier columns in customer-facing's `offers`
table with one JSON column represented by a custom Rust identifier value.
Its three properties remain nullable because some offers have no Shopify
identifiers. The user explicitly confirmed that existing identifier values
must be copied into the JSON field before the old columns are dropped.

This is a customer-facing persistence change. Keep the `gtin` and `sku`
columns, offers service's `read_offers` schema, upstream events and immutable
exports intact. Preserve REST/GraphQL response shapes and customers behavior.
There is no identifier reset, queue reset, event replay requirement, matching,
deduplication, identifier search or new identifier extraction.

The concrete column/type names, JSONB representation, constraints and deployment
procedure below are proposed implementation choices, not additional user
decisions. No unresolved product question remains after the preservation
confirmation.

## Starting behavior

Grounded at monorepo `ed55cb1eb34740ef1e2ace2a8ba0174a8a078443`.

- The [item 20 migration](../../services/customer_facing/migrations/2026-10-26-000003_offer_identifiers/up.sql)
  added five nullable text columns: GTIN, SKU and three Shopify identifiers.
  Add a new migration; do not rewrite this applied migration.
- [Projection writes](../../services/customer_facing/src/offer_projection.rs)
  parse validated `ProductIdentifiers` on creation and a complete decided set
  on `OfferIdentifiersChanged`. Creation inserts all five columns; identifier
  changes replace them. Each mutation and original-message receipt in
  `applied_offer_messages` share one transaction. A missing-parent update
  rolls back the receipt and remains retryable.
- The [read repository](../../services/customer_facing/src/offer_read.rs)
  selects the columns into a `QueryableByName` record, then constructs an API
  `ExternalIdentifiers` object with nullable strings. The object is already
  non-null on REST/GraphQL responses. Browse loads ordered images after selecting
  the limited offer page and orders by creation time and ID.
- [Shared identifiers](../../lib/retailer_sauce/src/identifier.rs) already
  provide `ExternalIdentifiers` with typed optional Shopify members. Its
  deserializer invokes `guard()` to reject empty/NUL-bearing strings; direct
  construction still needs a guard at the persistence boundary. Preserve
  strings exactly, including whitespace, case and full GIDs, without new
  checksum or GID-format restrictions.
- The services workspace already enables Diesel's `postgres` and `serde_json`
  features. A [typed JSONB adapter](../../services/retailer_sourcing/src/platform/db/json.rs)
  exists inside retailer sourcing as a useful pattern, not a public dependency
  customer-facing should import from another service.
- [Item 21](21-offer-title-search.md) is still a plan at this revision. Its
  proposed search query will share the browse offer record and DTO mapping.

## Proposed storage and Rust contract

Use `offers.external_identifiers JSONB NOT NULL`, containing exactly these
three snake_case keys:

```json
{
  "shopify_product_gid": "gid://shopify/Product/1",
  "shopify_product_variation_gid": null,
  "shopify_handle": "usb-charger"
}
```

Every value is a JSON string or JSON null. An offer with no Shopify data has
all three keys set to JSON null. Neither SQL NULL nor top-level JSON null is
valid. Keep a database default of the all-null object so compatible seed or
maintenance inserts that omit Shopify data retain their current behavior.
Projection writes still bind the object explicitly. Do not wrap it in another
`identifiers` or `value` key, stringify the JSON, omit null properties or place
GTIN/SKU inside it.

Add a named CHECK constraint requiring an object, presence of all three keys,
no other keys, and string-or-null values. Ensure the entire predicate is true
rather than relying on SQL UNKNOWN passing a CHECK: explicitly check required
keys and use an `IS TRUE` wrapper where necessary. Keep this a shape/type
constraint; value validation continues through the shared Rust guard. There
is no identifier uniqueness constraint or JSON index for this change.

Introduce a service-local persistence adapter, for example
`Identifier(pub retailer_sauce::identifier::ExternalIdentifiers)`, in a small
module shared by the projector and reader. This fulfills the custom identifier
type without duplicating the three member definitions or adding Diesel to
`retailer_sauce`. Derive `AsExpression` and `FromSqlRow` for `Jsonb`, implement
`FromSql<Jsonb, Pg>` and `ToSql<Jsonb, Pg>`, and use it directly in records and
binds:

```rust
#[diesel(sql_type = diesel::sql_types::Jsonb)]
external_identifiers: Identifier,
// INSERT/UPDATE binds use Jsonb and Identifier, not Nullable<Jsonb>.
```

On decoding, use Diesel's JSONB/`serde_json::Value` support, verify the exact
object shape, then deserialize into shared `ExternalIdentifiers`; reject
arrays, scalars, missing/extra keys and invalid present values. Explicit shape
checks avoid Serde accepting positional struct arrays or ignoring unknown
fields. On encoding, call `guard()`, serialize the shared object with all keys,
and delegate JSONB encoding to Diesel using the output reborrow required for
a temporary JSON value. Return errors instead of `expect`/panic or silently
substituting an empty object. Keep storage strictness local; do not tighten
legacy event deserialization, where omitted identifier fields have established
compatibility semantics.

## Data-preserving migration and deployment

Use a new transactional migration after the existing identifier migration.
Allocate its timestamp against the actual branch, including item 21's migration
if it has landed. Up migration:

1. Add the new JSONB column with the all-null default and NOT NULL constraint.
2. Update every existing row using
   `jsonb_build_object('shopify_product_gid', shopify_product_gid,
   'shopify_product_variation_gid', shopify_product_variation_gid,
   'shopify_handle', shopify_handle)`. SQL nulls become JSON nulls; text remains
   text. Do not use `COALESCE` to empty strings or parse GID suffixes.
3. Add/validate the shape constraint and verify each extracted JSON value is
   `IS NOT DISTINCT FROM` its original column before dropping anything. Fail
   the migration if verification fails, rolling back the whole transaction.
4. Drop only `shopify_product_gid`, `shopify_product_variation_gid` and
   `shopify_handle`. Leave GTIN/SKU, other offer facts, images, receipts, queues,
   indexes and offer identities intact.

The ALTER TABLE lock and transaction cover backfill through column removal;
there is no interval for another writer to update old fields after they were
copied. The update rewrites existing rows and the DDL blocks concurrent table
access. Check row count, disk/WAL headroom and lock duration on a representative
copy before deployment; do not describe this as an online rolling migration.

The down migration is also transactional: add the three nullable TEXT columns,
copy each field back with `external_identifiers ->> '<key>'`, verify equality
including nulls, then remove the JSON constraint and column. It restores the
latest JSON values, including changes written after upgrade, rather than a
stale pre-upgrade snapshot. Use the targeted down migration only: the existing
`make migrate-redo` reverts all migrations and is destructive.

Proposed deployment is a coordinated customer-facing maintenance window:

1. Build/test the new binary and migration together; retain the old binary and
   take a normal database backup. Run a read-only preflight over existing
   Shopify values with shared validation. If manual/legacy invalid values are
   found, report exact affected row IDs and stop deployment for a separate
   data decision; do not normalize or erase them to make the migration pass.
2. Stop routing requests/queue-run calls to customer-facing, wait for active
   batches to finish, and stop all old customer-facing instances. Do not rely
   only on pausing writers: old readers also reference the removed columns.
   Upstream delivery failures during downtime remain retryable; retain queued
   messages and pending outbox entries.
3. Start the new binary, whose embedded migrations execute before serving,
   or run its exact migration bundle explicitly while all instances are down.
   Inspect migrated values and health before reopening traffic. Resume delivery
   and explicit inbox/command batches without changing their message identities.
4. If rollback is needed after a successful migration, stop every new instance,
   apply this migration's down SQL through the migration tooling, then start
   the old binary. If up fails, its transaction leaves the old schema/data
   intact; verify that state before restarting the old binary.

Old and new binaries are not schema-compatible across the drop. If a later
deployment requirement demands uninterrupted mixed-version operation, revise
this procedure to separate expand/backfill and contract releases with an
explicit synchronized-write strategy before implementation; that complexity
is not required by this task.

## Projection, reads and integration

Change creation INSERT and identifier UPDATE SQL to bind the typed JSONB object
alongside the existing GTIN/SKU columns. An identifier-change event remains a
complete replacement, not JSON concatenation or another import enrichment
merge. Null members must replace previously populated values when they occur
in an authoritative complete event. Continue applying legacy creation events
with an all-null object through existing shared defaults.

Keep the original-message claim and all projection writes within the current
transaction. Serialization, constraint or write failure must leave no receipt
and no partial GTIN/SKU/JSON update. Missing parents remain retryable, duplicate
original messages remain no-ops, and no new ordering or stale-event guarantee
is introduced.

Replace the three fields in the read record and SELECT list with the typed
JSONB field, then map its validated inner values to the existing API DTO.
REST retains `external_identifiers.shopify_product_gid`,
`shopify_product_variation_gid` and `shopify_handle`; GraphQL retains its
existing camelCase names and object/member nullability. Do not rename the
public DTO to the persistence adapter's name. Keep sanitized read-error behavior
when stored data cannot decode. No API, event or export version bump is needed.

If item 21 lands first, change its ranked SELECT list and reuse the same
record-to-offer mapping; exercise both browse and search. If this item lands
first, the search implementation must use the JSONB record from the start.
Title indexing, ranking, pagination and ordered-image loading are unaffected.
No search extension dependency is introduced here. Item 22's localization
change does not depend on this schema migration.

Update customer-facing's README, persistence examples and migration/rollback
instructions. Add an indexed ADR narrowly superseding
[ADR 29](../adr/0029-offer-product-identifiers.md)'s requirement for three
customer-facing Shopify TEXT columns; retain its upstream schema, wire types,
validation and identifier update decisions. Do not edit that accepted ADR.
Update the identifier walkthrough only where it describes storage or database
inspection; its public response examples should remain valid.

## Acceptance and verification

- Migration fixtures start with full, partial and all-null Shopify sets, plus
  GTIN/SKU, localization, timestamps, images, receipts and queued commands.
  Compare exact values and row identities before/up/down/up; cover Unicode,
  quotes, backslashes, whitespace and leading-zero strings. Test rollback
  after a new JSON identifier update, not only immediate up/down. Verify only
  the intended columns change and pending legacy/new events remain processable.
- Adapter and PostgreSQL tests cover full/all-null objects, string round trips,
  invalid empty/NUL-bearing strings, SQL NULL, JSON null, arrays/scalars,
  numeric members, missing/extra keys and database default behavior. Confirm
  guarded encoding rejects directly constructed invalid shared values and
  decoding failures use the existing error path.
- Extend projection tests for creation, complete replacement including nullable
  members, unchanged GTIN/SKU behavior, missing-parent retry and replay with
  the same original message ID. Force a database write failure to prove receipt
  and all identifier changes roll back together; retain concurrency/replay tests.
- Update existing `to_jsonb(offers)` snapshot expectations to show the nested
  column and prove the old three keys/columns are absent. Exercise isolated
  migration tests through the actual migration harness and fresh-schema startup.
- REST/GraphQL tests compare identifier values and schema nullability before
  and after the storage change for populated/partial/unknown sets. Preserve
  pagination, image order and smoke tests. If implemented, verify item 21 search
  returns the same identifier objects as browse and retains its title index.
- Run `make fmt` then `make ci` in `services/customer_facing`; run
  `make test-postgres` with a separate `DATABASE_URL_TEST`. Broaden CI only to
  other changed crates. Existing dependencies should suffice; run `make audit`
  only if Cargo dependencies or lockfiles actually change.
- Start customer-facing with `PORT=<port> make serve` against an isolated
  populated database, check `/api/health`, browse REST/GraphQL, then submit a
  creation and identifier-only update through explicit inbox/command batches.
  Observe JSONB values, replay protection and unchanged API responses. Exercise
  search too if available. Record the targeted rollback/re-upgrade results.

Record implementation revision, migration identifier, preservation counts,
CI/PostgreSQL results and live verification before marking the item complete.
This document authorizes no implementation or deployment by itself.

## Implementation verification (2026-10-26)

Implemented migration `2026-10-26-000005_shopify_identifiers_json`, the local
Diesel `Identifier` adapter, complete JSONB projection writes and shared
browse/search mapping. ADR 32 records the maintenance-window deployment and
rollback. No Cargo dependency, upstream contract or public response shape changed.

- Customer-facing `make fmt` then `make ci`: pass, 35 tests passed and 29
  opt-in PostgreSQL tests ignored. Adapter tests validate exact full/partial/
  all-null shapes, exact strings, missing/extra keys and invalid values.
- New PostgreSQL tests compile and cover three populated migration rows plus
  GTIN/SKU, locale/time facts, images, receipts, pending legacy/new commands,
  up/down/up, latest-value rollback, invalid-legacy migration rollback, Diesel
  JSONB bindings, SQL defaults/constraints, missing parents, receipt atomicity,
  replay, and REST/GraphQL browse/search compatibility. They have not run;
  there is no verified database preservation count or measured lock duration.
- `make test-postgres` reports DATABASE_URL_TEST is required. Docker access
  remains denied. `PORT=39125 make serve` builds and then reports DATABASE_URL
  is required; live health, migration and queue demonstrations are unverified.
- `git diff --check` passes. No audit was required because dependencies and
  lockfiles are unchanged. No database migration was applied in this session.

That initial run left the TODO unchecked. The completion run below resolves
the database and local-server restrictions.

## Completion verification (2026-10-26)

Verified the working tree at `cc57884` against disposable PostgreSQL 16.
Root `make fmt` and `make ci` pass. Customer-facing `make test-postgres` passes
all 29 tests. Migration `2026-10-26-000005_shopify_identifiers_json` preserves
all three populated fixture rows (full, partial and all-null identifiers),
including exact strings, GTIN/SKU, images, localization, timestamps, receipts
and queued messages. Up/down/up, latest-value rollback, invalid-legacy rollback,
JSONB constraints and REST/GraphQL browse/search parity all pass.

Live customer-facing started on port 39201 using `make serve`, migrated a fresh
isolated database and passed health checks. The complete sourcing-to-customers
pipeline created offers carrying all three identifiers. A subsequent
OfferIdentifiersChanged inbox message replaced the complete JSON object;
resubmitting its message ID with a changed handle did not alter the projection.
Browse/search and GraphQL retained their public response shape. No existing
development database, dependency or lockfile changed.
