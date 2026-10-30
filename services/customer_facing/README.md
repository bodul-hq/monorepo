# Customer facing

Poem service with durable Mulac inbox acceptance and explicit queue batch APIs.
REST and GraphQL health report liveness. Incoming offers events maintain the
service-owned `offers` table during command processing. Paginated REST and
GraphQL queries read this projection; no projection events or outgoing
messages are generated.

## Setup and startup

Use the repository's pinned Rust toolchain, GNU Make, libpq, and PostgreSQL 16
or later. The sibling Mulac checkout must be at `../mulac` beside `monorepo`.
The queue adapters were integrated against Mulac revision
`9474319fff2237d81828a23d97df5d2f97741858`.

Configure this service's own UTF8 development database and a separate test
database. The development user needs migration DDL permissions; the test user
also needs `CREATE SCHEMA`. The parent deployment's `customer_facing-db`
container provides `customer_facing_dev` and `customer_facing_test`; use its
configured credentials and reachable address.

```sh
make .env
# Edit .env to set DATABASE_URL and DATABASE_URL_TEST.
PORT=3003 make serve
curl http://localhost:3003/api/health
```

`DATABASE_URL` is required. Missing, blank, invalid-Unicode, inaccessible, or
invalid database configuration stops startup. Embedded migrations run before
HTTP begins serving; migration failure also stops startup. `PORT` defaults to
3000, with invalid values rejected. The Makefile reads the git-ignored `.env`
and preserves shell and command-line overrides, including empty values. Direct
`cargo run` requires exported variables.

Use `make migrate` for optional explicit migration runs with Diesel CLI.
`make migrate-redo` deletes queue data and is only for an intentional development
reset. No other service's tables or database are needed.

## Routes and manual queue processing

| Method and path | Behavior |
| --- | --- |
| `GET /api/health` | `200`, text `ok`; does not probe PostgreSQL |
| `GET /api/reporting/database-tables` | Authenticated table storage and estimated row counts |
| `GET /api/reporting/entry-statuses` | Authenticated exact inbox/command/event/outbox counts by status |
| `GET /api/offers` | Paginated offers, with `limit` and `offset` query parameters |
| `POST /api/inbox` | Store a message and return `202`, `{"message_id":"<uuid>"}` |
| `POST /api/inbox/run` | Convert one batch of due inbox messages into queued commands |
| `POST /api/commands/run` | Execute one batch of due commands |
| `POST /api/events/run` | Dispatch one batch of due events |
| `POST /api/outbox/run` | `503`, `outbox publisher is not configured`; rows remain untouched |
| `GET /api/openapi.json` | OpenAPI description |
| `GET /api/docs/` | Swagger UI |
| `GET /graphql`, `POST /graphql` | GraphQL `health` and paginated `offers` queries |
| `GET /graphql/ide` | GraphiQL in debug builds, `404` in release |

Submit the agreed Mulac envelope directly; no upstream service is required:

```sh
curl http://localhost:3003/api/inbox \
  -H 'content-type: application/json' \
  -d '{"routing_key":"OfferCreated","message_id":"0191d6c4-8f5e-7a3b-9c1d-2e3f4a5b6c7d","correlation_id":null,"source":"offers","payload":{"id":"0191d6c4-8f5e-7a3b-9c1d-2e3f4a5b6c7e","retailer_code":"AnkerAu","title":"USB charger","price":{"amount":1999,"currency":"AUD"},"url":"https://shop.example/Charger?variant=123#details","created_at":"2026-10-24T12:00:00Z","images":["https://shop.example/front.jpg"]}}'
curl http://localhost:3003/api/inbox/run \
  -H 'content-type: application/json' -d '{"limit":10}'
curl http://localhost:3003/api/commands/run \
  -H 'content-type: application/json' -d '{"limit":10}'
curl http://localhost:3003/api/events/run \
  -H 'content-type: application/json' -d '{"limit":10}'
```

Submission returns `202` after durable receipt, with no offer yet. The inbox
runner queues commands and still does not apply the offer. The command runner
commits the offer and its replay receipt together. The event runner has nothing
to do for these messages (normally `processed: 0`); it remains available for
other queued events. Inspect this service's database after command success:

```sql
SELECT id, retailer_code, title, price_amount, currency, url, created_at
FROM offers WHERE id = '0191d6c4-8f5e-7a3b-9c1d-2e3f4a5b6c7e';
```

Use the same envelope with a new message UUID for each distinct update:

| Exact routing key | Payload |
| --- | --- |
| `OfferCreated` | `id`, `retailer_code`, `title`, `price`, `url`, `created_at`, ordered `images` (missing in legacy messages means `[]`) |
| `OfferTitleChanged` | `{"id":"<offer UUID>","title":"New title"}` |
| `OfferPriceChangedEvent` | `{"id":"<offer UUID>","price":{"amount":0,"currency":"EUR"}}` |
| `OfferPricingChanged` | `{"id":"<offer UUID>","pricing":{"price":{"amount":1999,"currency":"AUD"},"was_price":{"amount":2999,"currency":"AUD"}}}` |
| `OfferAvailabilityChanged` | `{"id":"<offer UUID>","availability":"in_stock"}` |
| `OfferItemConditionChanged` | `{"id":"<offer UUID>","item_condition":"new"}` |
| `OfferContentsChanged` | `{"id":"<offer UUID>","content":[{"language":"en","title":"USB charger","description":null}],"language":"en"}` |
| `OfferContentChanged` (legacy) | `{"id":"<offer UUID>","content":{"description":"<p>Source description</p>"}}` |
| `OfferImageAdded` | `{"id":"<offer UUID>","image_url":"https://shop.example/front.jpg","position":0}` |

Payloads must be JSON objects without an event wrapper or JSON string encoding.
The price event's `Event` suffix is required. Retailer enum values such as
`AnkerAu` persist as `ankerau`. `Money` validates signed i64 minor units and
supported currencies; no floating-point conversion occurs. UUID, Unicode and
whitespace, exact URL (up to 2048 bytes), and the original creation instant at
PostgreSQL microsecond precision are preserved. NUL in text is rejected.
Creation enforces both offer-ID uniqueness and the retailer/exact-URL identity;
URL comparison is case-sensitive. A conflicting creation under another message
identity fails, without overwriting the existing row. Updates require creation
and change only the named properties (price amount and currency together).

`applied_offer_messages` records the original inbox message UUID from command
causation metadata, not the newly generated command UUID. Its unique claim and
the business write share one transaction: concurrent duplicates have one effect,
failed writes roll back the claim, and replay after later updates or restart
cannot restore older values. Queue completion happens separately; a crash after
commit can safely retry the command. Keep receipts for as long as messages may
be replayed. Never reuse a message UUID for different content.

Distinct messages apply in processing order. There is no offer revision, source
ordering, or stale-update protection: a delayed distinct update can overwrite a
newer source value. Batches and concurrent runners do not promise source order.

The optional `message_id` and `correlation_id` must be UUID strings. Omitting the
message ID generates a UUIDv7. Repeating an existing message ID succeeds and
preserves the original row, even if the new payload differs. Unknown routing
keys are accepted. An accepted message is durable but has not been processed.
The sender must supply a nonblank `source`; offers uses `offers` when its own
source metadata is absent. Inbox validation still rejects missing or blank sources.

Every runner requires an integer `limit` between 1 and 1000. Successful batches
return `{"processed":N,"errors":[]}`. Batch failures, including reservation or
storage failures, return `200` with null `processed` and error messages.
Successful items in a partially failed batch remain completed. Inspect
`status`, `attempts`, `scheduled_at` and `extra_info` in the corresponding
queue; retries use Mulac's backoff and six-attempt default.

Invalid inputs return `400`, missing/non-JSON content types return `415`, and
inbox storage or blocking-task failures return sanitized `500` responses.
Shared Mulac schema wrappers document request and success-response bodies;
adapter errors use plain text. No authentication is added to these development
queue endpoints.

There are no background workers, broker connections, outbound publisher, or
automatic stale-reservation sweeper. Each API call advances only its own queue.
A process killed during a reservation may leave work reserved; an operator must
inspect and recover stale reservations before they can run again (see below).
Do not use migration resets as recovery. Ctrl+C/SIGTERM waits for
in-flight HTTP requests and then shuts down the kernel.

The [Bruno collection](../../bruno/customer_facing/README.md) provides the same
manual requests.

## Browsing offers

```sh
curl 'http://localhost:3003/api/offers?limit=30&offset=0'
curl http://localhost:3003/graphql -H 'content-type: application/json' \
  -d '{"query":"{ offers(limit: 30, offset: 0) { items { id retailerCode title price { amount currency } url images createdAt } limit offset hasMore } }"}'
```

REST returns this page shape (GraphQL uses `retailerCode`, `createdAt`, and
`hasMore`; `id` is GraphQL `ID`):

```json
{
  "items": [{
    "id": "0191d6c4-8f5e-7a3b-9c1d-2e3f4a5b6c7e",
    "retailer_code": "ankerau",
    "title": "USB charger",
    "price": {"amount": "1999", "currency": "AUD"},
    "url": "https://shop.example/Charger?variant=123#details",
    "images": ["https://shop.example/front.jpg"],
    "created_at": "2026-10-24T12:00:00Z"
  }],
  "limit": 30,
  "offset": 0,
  "has_more": false
}
```

Both APIs default to limit 30 and offset 0. Limit must be an integer from 1
through 100; offset must be an integer from 0 through 2147483647. GraphQL
arguments are non-null with defaults. Invalid inputs return REST 400 or
GraphQL errors. Database or blocking-task failures return sanitized REST 500
or GraphQL errors; they never masquerade as an empty result.

Offers sort by `created_at DESC, id DESC`. A query fetches one additional row
to determine `has_more`; it returns at most the requested limit and no total
count. Empty and past-the-end pages return empty `items` with `has_more: false`.
New offers can shift pages between requests; this is not snapshot pagination.

Read API amounts are **decimal strings of signed i64 minor units**, preserving
the full range through REST and GraphQL clients. Inbox event amounts remain
JSON integers. Currency codes are uppercase, retailer codes are lowercase,
and original timestamps are represented in UTC RFC3339 at stored precision.
Titles and URLs retain their original values. Reading does not process queues:
the submission and inbox steps alone do not make an offer visible. Run
`/api/commands/run` after `/api/inbox/run` to apply it. Later processed title
and price changes appear on the next read.

The [customers app](../../apps/customers/README.md) renders these offers with
30 per page. See [ADR 23](../../docs/adr/0023-offer-browsing-and-customers-homepage.md)
for the service boundary and representation choices.

## Failure inspection and recovery

Malformed payloads, unsupported routing keys, conflicting creation, missing
offers, and database errors appear in the command runner's `errors` array with
`processed: null`. The HTTP response is still `200`; it does not mean every
command succeeded. Inspect the affected rows:

```sql
SELECT id, command_type, meta->>'causation_id' AS message_id,
       status, attempts, scheduled_at, reserved_at, extra_info
FROM command_entries WHERE status <> 5 ORDER BY received_at;
```

Statuses are received `0`, reserved `2`, failed `4`, completed `5`, dead `7`.
For updates arriving before creation, first submit and process creation, then
run commands again when `scheduled_at` is due. Failed commands are retried with
Mulac backoff, up to six attempts. Exhaustion requires operator intervention;
reposting an already stored inbox ID alone does not requeue it.

There is no sweep HTTP endpoint or automatic sweeper. Mulac exposes
`commanding::io::CommandSweeper` with `CommandConsumerStorage` and
`StaleCommandSpec`, plus the inbox `ReservationSweeper`. Its current sweep
releases reservations into failed status without resetting attempts. Rows at
six attempts remain ineligible even if a sweep leaves status `4`.

For targeted recovery with `psql`, stop active runners, inspect the selected row
and its error history, and set `command_id` to that row's UUID. After verifying
that the reservation owner is gone, this example releases a reservation older
than 15 minutes without changing message identity or retry count:

```sql
BEGIN;
SELECT * FROM command_entries WHERE id = :'command_id'::uuid FOR UPDATE;
UPDATE command_entries
SET status = CASE WHEN attempts >= 6 THEN 7 ELSE 4 END,
    reservation_id = NULL, reserved_at = NULL,
    scheduled_at = now(), updated_at = now(),
    extra_info = jsonb_set(COALESCE(extra_info, '{}'::jsonb), '{errors}',
        COALESCE(extra_info->'errors', '[]'::jsonb)
        || jsonb_build_array('operator released confirmed stale reservation'))
WHERE id = :'command_id'::uuid AND status = 2
  AND reserved_at < now() - interval '15 minutes';
COMMIT;
```

The same targeted recovery applies to `inbox_entries` with the inbox row UUID;
inbox replay may queue a new command UUID, which remains protected by the
original message receipt. An operator may deliberately retry an exhausted
command **after fixing its cause** with the following selected-row operation.
Keep payload, metadata, receipts, and error history intact:

```sql
UPDATE command_entries
SET status = 4, attempts = 0, scheduled_at = now(), updated_at = now()
WHERE id = :'command_id'::uuid AND status IN (4, 7)
  AND attempts >= 6 AND reservation_id IS NULL;
```

Resume the relevant manual runners. Do not truncate queues, delete replay
receipts, or run `migrate-redo` for recovery.

## Checks

```sh
make fmt
make ci
make test-postgres
# Only when dependencies or Cargo.lock changed:
make -C .. audit
PORT=3003 make serve
```

`make ci` runs configuration and HTTP tests without PostgreSQL. The opt-in
PostgreSQL suite requires `DATABASE_URL_TEST`; each case creates and drops only
its own generated schema. It verifies migration round trips and failures,
durable acceptance, duplicate IDs, metadata, separate queue stages, batch
limits, retries and exhaustion, and unchanged unavailable-outbox rows. Production
projection tests cover value fidelity, independent updates, concurrent replay,
restart, business-write rollback, conflicting identities, malformed messages,
and additive migration with existing queued work. Tests submit directly to the
real inbox routes; no offers publisher or cross-service database is needed.
Browse tests cover REST/GraphQL pagination, deterministic tie ordering, exact
string amounts and timestamps, projection visibility and updates, and an index
migration round trip that preserves existing offers. Database-independent HTTP
tests verify defaults, bounds, documentation and sanitized failures.

Run `cargo test --release graphiql_is_served_only_in_debug_builds` to check the
release GraphiQL route. After serving, check health, GraphQL health, OpenAPI and
each runner. Run the audit when dependencies or the lockfile change and report
new advisories. See [ADR 18](../../docs/adr/0018-customer-facing-mulac-queues.md)
for the runtime and startup decision.

See [ADR 22](../../docs/adr/0022-customer-facing-offer-projection.md) for the
projection transaction and metadata-preserving consumer decision.

## Development dashboard reports

The [development dashboard](../../apps/development/README.md#database-dashboard)
reads two service-owned REST reports. Both require bearer authentication;
`CUSTOMER_FACING_REPORTING_TOKEN` defaults to `customer-facing-development-token`.
An explicitly empty token authorizes no requests. The Makefile loads this
setting from `.env`, preserving shell and command-line overrides.

```sh
curl http://localhost:3003/api/reporting/database-tables \
  -H 'Authorization: Bearer customer-facing-development-token'
curl http://localhost:3003/api/reporting/entry-statuses \
  -H 'Authorization: Bearer customer-facing-development-token'
```

`database-tables` returns `schema_name`, UTC `generated_at`,
`row_count_kind: "estimated"`, and `tables`. Each table has `table_name`,
`row_count`, `table_bytes`, `index_bytes`, and `total_bytes`. Ordinary tables in
the active schema include offers, queues, and migration records; views and
system schemas are excluded. Tables are sorted alphabetically. Row counts use
PostgreSQL live-row estimates and can lag recent writes; zero does not prove a
table is empty. Table bytes include TOAST and auxiliary storage; total bytes
are table plus index bytes. Reporting never runs `ANALYZE` or scans application
tables to calculate row estimates.

`entry-statuses` returns UTC `generated_at`, `command_entries`, and
`event_entries`. Each entry has numeric `status`, `status_text`, and exact
`count`, sorted by status. Both queues are counted in one SQL statement.
Labels are `0 received`, `2 reserved`, `4 failed`, `5 completed`, `7 dead`,
and `8 archive`; other codes remain visible as `unknown`. Empty queues return
empty arrays. Inbox/outbox status breakdowns are not included.

All report responses use `Cache-Control: no-store`. Missing or incorrect
credentials return `401`, database unavailability returns `503`, and query or
blocking-task failures return sanitized `500` errors. Reporting does not run
queues or change their rows. Other routes keep their existing access rules.
Bruno includes **Report database tables** and **Report command and event
statuses**; adjust their development bearer token if overriding the service.

The development app defaults `CUSTOMER_FACING_URL` to
`http://172.28.100.42:3000`. Its `CUSTOMER_FACING_TOKEN` must match this service's
reporting token. Use `CUSTOMER_FACING_URL=http://localhost:3003` when running
customer-facing locally on port 3003. Restart each process after configuration
changes; recreate containers when changing their environment.

## Ordered image projection

Migration `2026-10-25-000001_offer_images` adds `offer_images` without rewriting
old offers or receipts. `OfferImageAdded` requires a UUID, a valid exact HTTP(S)
URL without credentials/padding/controls (at most 2048 bytes), and a u32 position.
Invalid events fail; only import producers filter invalid strings.

An image addition and its original-message receipt commit together. Missing
parents and conflicting position/URL pairs roll back the receipt and retry.
Identical additions under different message IDs succeed without extra images.
Out-of-order messages may leave temporary gaps; REST and GraphQL always return
URLs sorted by stored position. Old rows return `images: []`. A single SQL page
query includes the image lists without duplicating offers or changing pagination.
Title/price changes retain all images. Deploy this consumer before offers starts
publishing images. See [ADR 24](../../docs/adr/0024-append-only-offer-images.md).

Initial images now arrive in `OfferCreated`. The parent, all initial images and
its original-message receipt commit atomically; a failed image insert rolls back
the entire creation. Missing images in old events default to `[]`; malformed,
invalid or duplicate event images are rejected. Separate `OfferImageAdded` events
are emitted only by updates, while already queued legacy additions remain
supported. Deploy this consumer before the new offers producer; see
[ADR 25](../../docs/adr/0025-initial-images-in-offer-created.md).

## Scrape time

The successful download instant travels with immutable artifact references through
v3 and newer exports to `OfferCreated` and `OfferScrapedAtUpdated`. Old imports/events retain
unknown time. Newer times advance the aggregate and both projections even when
content is unchanged; absent, equal and older times never erase or regress it.
Customer-facing returns nullable `scraped_at` (REST) and `scrapedAt` (GraphQL),
independent of creation time. See [ADR 26](../../docs/adr/0026-offer-scrape-provenance.md)
for provenance, compatibility and deployment order.

## Localization, identifiers and revisions

New sourcing exports use schema v5: v3 added scrape time, v4 added a localization
observation, and v5 adds GTIN, SKU and the three Shopify identifiers. Prepared
v1–v4 snapshots keep their original payload shapes and IDs. Missing localization
means no observation; an explicit object replaces both facts, including unknowns.
Missing/null identifiers retain known values; supplied fields enrich or correct
them. Historical `OfferCreated.v5` and earlier records remain readable; current
creation history uses `OfferCreated.v9` with localized content.
Localization-only and identifier-only changes have their own domain events.

Offers preserves each Testimony revision in Mulac event and outbox metadata;
legacy metadata can omit it. Use the sibling Mulac optional-sequence API before
building/deploying this version. This does not change HTTP inbox metadata or add
source-order guarantees. See the [verification walkthrough](../../docs/offer-facts-walkthrough.md)
and ADRs [27](../../docs/adr/0027-offer-revision-metadata.md),
[28](../../docs/adr/0028-source-backed-offer-localization.md) and
[29](../../docs/adr/0029-offer-product-identifiers.md).

## Pricing, availability, condition and description

Creation also accepts nullable `was_price`, `availability`, `item_condition`,
and the localized content list described below. Legacy creation messages may
omit those facts or supply `content: {"description": null}`; content is adapted
to a singleton using the original title and language. Each changed event above replaces its
complete decided field atomically with the inbox receipt. `pricing.was_price`,
`availability`, `item_condition`, and `content.description` must be present in
their change events; explicit null clears that fact. A legacy
`OfferPriceChangedEvent` still changes only the current price.
New `OfferPricingChanged` events require a greater was-price in the same
currency. Historical creation values are retained as recorded and filtered
only when read.

Availability event values are `available`, `unavailable`, `in_stock`,
`out_of_stock`, `sold_out`, or `{"other":"original source text"}`. Item
condition is `new` or the same `other` object shape. Unknown source text is
preserved exactly; blank/NUL statuses are rejected. Description keeps the
normalized source HTML or text unchanged, rejects NUL, and is never rendered
by the customers app. Title and images remain separate fields.

REST browse and search return nullable `was_price` using the existing price
object and decimal-string minor units. GraphQL calls this `wasPrice`. It is
visible only when its currency matches the current price and its amount is
greater; incomparable values remain stored. Both transports expose nullable
typed availability/condition objects (`kind`, `raw`), where `raw` is populated
only for `other`. REST kind values use the event spellings; GraphQL enum values
are uppercase. REST `item_condition` becomes GraphQL `itemCondition`.
`content` is a nonempty list of entries with nullable `language`, required
`title`, and nullable `description`; see the localized-content rollout below.

Migrations `2026-10-27-000006` through `000008` add paired nullable was-price
columns, validated nullable status JSONB, and nullable description TEXT. They
preserve existing rows, images, receipts and queues. Deploy this consumer and
its migrations before offers publishes the new events. Reverting these three
migrations drops the new facts; it retains the prior projection columns and
receipts. Retain a backup or replay source before such a rollback. The isolated
PostgreSQL suite covers populated migration round trips, atomic changes,
explicit clearing, receipt replay, retry after creation, and transport parity.

## Title search

`GET /api/offers/search?q=charger&limit=30&offset=0` and GraphQL
`searchOffers(q: "charger", limit: 30, offset: 0) { items { id title } hasMore }`
return the same offer fields as browsing. Defaults and bounds are unchanged:
limit 30 (1–100), offset 0 (0–2147483647). Only text, limit and offset are
search options. Missing/invalid text returns REST 400 or a GraphQL error;
read failures return sanitized 500/errors, not an empty result.

Text is trimmed and whitespace collapsed. Use 1–200 Unicode scalar values,
including at least one letter or number; non-whitespace controls are invalid.
Search matches persisted titles, including any raw markup, before card display
cleanup. It ranks case-insensitive exact titles first, then word similarity,
then creation time and ID descending. The fixed similarity threshold is 0.5.
Punctuation is never SQL/wildcard syntax; trigram matching ignores non-word
characters. There is no stemming, accent folding, total count or snapshot
pagination. Empty/past-end results are successful empty pages.

Before migrating **each development, test or production database**, a database
owner must provision the trusted extension in its fixed schema:

```sql
CREATE SCHEMA customer_facing_extensions;
REVOKE CREATE ON SCHEMA customer_facing_extensions FROM PUBLIC;
CREATE EXTENSION pg_trgm WITH SCHEMA customer_facing_extensions;
-- Replace customer_facing with your runtime and test roles:
GRANT USAGE ON SCHEMA customer_facing_extensions TO customer_facing;
SELECT n.nspname FROM pg_extension e
JOIN pg_namespace n ON n.oid = e.extnamespace WHERE e.extname = 'pg_trgm';
```

The server must provide `pg_trgm`; installation needs database CREATE privilege.
If it already exists elsewhere, stop and arrange an explicit operator-managed
relocation after checking other consumers. `IF NOT EXISTS` does not relocate it.
Never grant untrusted roles CREATE on the extension schema. Startup checks the
prerequisite and does not install extensions in isolated test schemas.

Migration `2026-10-26-000004_offer_title_search` adds a GIN index on `lower(title)`
over existing data. Its transactional build blocks writes: budget lock time
against a representative populated copy. Its targeted down migration removes
only that index. Deploy customer-facing before the customers app. Search sets a
three-second statement timeout and threshold inside its transaction, leaving
pooled connections' settings unchanged. The isolated `make test-postgres` suite
requires the provisioned extension and tests ranking, page mapping, title
updates, replay, index reversal and setting isolation. See [ADR 30](../../docs/adr/0030-offer-title-search.md).

## Shopify identifier JSONB storage and migration

Customer-facing now stores `offers.external_identifiers` as a validated JSONB
object; offers service's `read_offers` columns and all public contracts stay the
same. GTIN and SKU retain separate columns. Example storage:

```json
{"shopify_product_gid":"gid://shopify/Product/001","shopify_product_variation_gid":null,"shopify_handle":"usb-charger"}
```

All three keys are required and nullable; the column itself is NOT NULL. The
all-null object is the database default. `offers_external_identifiers_shape`
rejects missing/extra keys and non-string/non-null values. The local typed
`Identifier` adapter additionally validates present strings through shared
`ExternalIdentifiers`, preserving whitespace, Unicode, case and leading zeros.
Legacy wire defaults remain compatible. Projection creation binds the whole
object; an identifier-change event replaces the complete decided set, including
null members, atomically with its original-message receipt. There is no JSON
merge or identifier reset. REST/GraphQL browse and search reuse that typed value.

Migration `2026-10-26-000005_shopify_identifiers_json` copies all three old TEXT
values, checks null-aware equality, then drops those columns in one transaction.
It retains GTIN/SKU, localization, times, images, receipts, queue entries and
indexes. Its down migration restores **latest** JSON values to the old columns.

Deployment procedure:

1. Build and test the binary/migration together and retain the previous binary
   and a database backup. Check row count, disk/WAL headroom and lock/backfill
   duration on a representative populated copy. Run this read-only preflight:

   ```sql
   SELECT id FROM offers
   WHERE shopify_product_gid = '' OR shopify_product_variation_gid = ''
      OR shopify_handle = '';
   ```

   These are the shared guard's invalid PostgreSQL text values; PostgreSQL TEXT
   already excludes NUL. Whitespace-only strings are valid and stay unchanged.
   If rows are returned, stop and resolve the affected IDs through a separate
   data decision. The migration also rejects such rows, without erasing values.
2. Stop routing requests and queue-run calls to customer-facing, wait for active
   batches, and stop **all** old instances. Old readers reference the removed
   columns too. Delivery failures during maintenance remain retryable.
3. Start the new binary to apply embedded migrations before serving, or run
   `make migrate` while all instances are down. Inspect the migrated objects:

   ```sql
   SELECT id, gtin, sku, external_identifiers FROM offers ORDER BY id LIMIT 30;
   ```

   Check health, REST/GraphQL browse/search, a creation and identifier-only update,
   then resume inbox/command runs and delivery with original message identities.
4. For rollback, stop every new instance. With this migration still the latest,
   use `diesel migration revert` from this service to revert only
   `2026-10-26-000005_shopify_identifiers_json`; verify the migration version
   before running it. Verify restored TEXT values, then restart the old binary.
   If later migrations exist, plan their rollback first. Do not use
   `make migrate-redo`, which reverts all migrations and deletes data.

This is a maintenance migration, not an online mixed-version deployment. The
isolated PostgreSQL suite includes populated up/down/up preservation, updates
before rollback, pending legacy/new messages, strict JSONB/default checks,
receipt rollback/retry and API compatibility. See [ADR 32](../../docs/adr/0032-customer-facing-shopify-identifiers-json.md).

## Localized offer content

Browse, search and GraphQL return a nonempty `content` list:

```json
[{"language":null,"title":"Charger","description":null},{"language":"en","title":"USB charger","description":"<p>Source HTML</p>"}]
```

Keys retain their exact historical spelling, with the unknown (`null`) bucket
first and other keys sorted lexically. `localization.language` selects the entry
whose title and description are also stored in the scalar columns; the flat API
`title` and existing title search continue to use that selection. Descriptions
preserve source text verbatim and are data only. A complete snapshot arrives as
`OfferContentsChanged {id, content, language}`; nullable `language` is required.
The projection validates the entire list and updates all selected fields in one
statement within the original-message receipt transaction. Historical creation
messages may omit content or contain the old description object. Legacy title
and description events update the selected bucket. Legacy language changes move
that bucket, replace a destination collision, and retain the other buckets.
Those read/modify/write operations lock the offer row within the receipt
transaction to avoid losing concurrent updates.

Migration `2026-10-27-000009_localized_offer_content` transactionally backfills
`offers.content` from the exact `(language, title, description)` scalar tuple,
then requires a nonempty JSONB array. It retains scalar columns, indexes, queues,
images and receipts. Deploy the customers compatibility reader first, stop all
customer-facing projection writers and apply this migration/readers, then deploy
the offers writer. Resume processing original queued message identities after
checking browse/search and selected fields. For rollback, stop writers and
retain a backup or replay source before reverting this migration: its down
migration drops only the list, keeping the selected scalar fields, but loses all
unselected languages. Do not use `make migrate-redo` against operational data.

`make test-postgres` covers populated up/down/up preservation with pending work,
mixed legacy/snapshot replay, receipt rollback, row locking, and REST/GraphQL
browse/search parity. It requires a separate `DATABASE_URL_TEST` with `pg_trgm`
provisioned as described above.

Command and event counts from `/api/reporting/entry-statuses` include only entries
whose current `scheduled_at` falls within today in UTC (midnight inclusive, next
midnight exclusive), grouped by current status. Later today is included; retries
can move entries between days. These counts are not historical throughput.

The entry-status report includes `inbox_entries`, `command_entries`,
`event_entries`, and `outbox_entries` arrays, grouped by current status for
entries scheduled today in UTC. All four queues share one query snapshot.
Reports include `window_start` (inclusive) and `window_end` (exclusive), the
UTC midnight bounds used by the query. The dashboard derives its report date
from these bounds. Reports without bounds show an upgrade message for the
entire panel; their potentially all-time counts are not displayed as daily totals.
Invalid or incomplete windows are rejected. Missing inbox/outbox arrays in a
windowed report still show an upgrade message for those queues.

### Daily reporting indexes

The `2026-10-30-000001_queue_scheduled_at_indexes` migration adds a B-tree
`scheduled_at` index to each of the four queue tables. Existing worker indexes
remain in place. It uses the existing transactional migration runner:
**index creation blocks writes**, so schedule deployment in a maintenance
window appropriate to the queue sizes. The down migration drops only these
four indexes.

The ignored `postgres_scheduled_indexes_round_trip_and_explain` test uses
an isolated schema to verify up/down/reapply, unchanged reports and preserved
worker indexes. It also prints PostgreSQL version, baseline and indexed
`EXPLAIN (ANALYZE, BUFFERS)` plans, and index sizes for historical-heavy data:

```sh
cargo test --bin customer_facing postgres_scheduled_indexes_round_trip_and_explain -- --ignored --nocapture
```

Set `DATABASE_URL_TEST` as for `make test-postgres`.
