# retailer_sourcing

Finds what retailers sell. It crawls each retailer's sitemaps, sorts the page
URLs into products, catalogs and other content, records every product URL as an
offer, downloads the offer pages, and reads the offer data out of them: it
extracts each page into JSON, normalizes and structuralizes it, then prepares an
immutable export of its priced variations.

The code is organised as feature slices
([ADR 4](../../docs/adr/0004-feature-slices-for-retailer-sourcing.md)). Each
slice handles one command and returns events; a reaction in the next slice turns
an event into the next command. Commands and events are stored in Postgres by
the [Mulac](https://github.com/nulllabsdev/mulac) kernel.

## Workflow

```
POST /api/sitemap-retrievals
  └─▶ RetrieveSitemap ──▶ retrieve_sitemap handler ──▶ SitemapRetrieved
        └─▶ process_sitemap reaction ──▶ ProcessSitemap ──▶ process_sitemap handler ──▶ SitemapProcessed
              └─▶ consolidate_sitemap reaction ──▶ ConsolidateSitemap ──▶ consolidate_sitemap handler ──▶ SitemapConsolidated
                    └─▶ discover_offers reaction ──▶ DiscoverOffers ──▶ discover_offers handler ──▶ OfferDiscovered × N
                          └─▶ download_offer_page reaction ──▶ DownloadOfferPage × N ──▶ download_offer_page handler
                                ──▶ OfferPageWasDownloaded | no event (download failed)
                                  └─▶ extract_offer_page reaction ──▶ ExtractOfferPage ──▶ extract_offer_page handler ──▶ OfferPageWasExtracted
                                        └─▶ normalize_offer reaction ──▶ NormalizeOffer ──▶ normalize_offer handler ──▶ OfferWasNormalized
                                              └─▶ structuralize_offer reaction ──▶ StructuralizeOffer ──▶ structuralize_offer handler ──▶ OfferWasStructuralized
```

| Step | Command              | Handler does                                                                                                                                                                                                                               | Event                                                    | Reaction that follows                                                 |
| ---- | -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------- | --------------------------------------------------------------------- |
| 1    | `RetrieveSitemap`    | Fetches the retailer's root sitemaps and every child sitemap and uploads the files to object storage, with references in `sitemaps_raw`. A file that can't be fetched or parsed is skipped with a warning. Fails without a sitemap config. | `SitemapRetrieved`                                       | `process_sitemap`: queue `ProcessSitemap`                             |
| 2    | `ProcessSitemap`     | Parses the stored files into one sitemap tree, stores it in `sitemaps_processed`.                                                                                                                                                          | `SitemapProcessed`                                       | `consolidate_sitemap`: queue `ConsolidateSitemap`                     |
| 3    | `ConsolidateSitemap` | Sorts every URL of the tree into product, catalog, content, not interested and unknown, using the retailer's link rules; stores the groups in `sitemaps_consolidated`.                                                                     | `SitemapConsolidated`                                    | `discover_offers`: queue `DiscoverOffers`, unless it found 0 products |
| 4    | `DiscoverOffers`     | Reads the product URLs and gives each its own `offer_id`. Writes nothing.                                                                                                                                                                  | `OfferDiscovered`, one per URL                           | `download_offer_page`: queue one `DownloadOfferPage` per offer        |
| 5    | `DownloadOfferPage`  | Downloads the page and stores it in `offer_details_raw` under the `offer_id`, with its retailer code. A failed download is logged, and the command still completes.                                                                        | `OfferPageWasDownloaded`; none when the download failed  | `extract_offer_page`: queue `ExtractOfferPage`                        |
| 6    | `ExtractOfferPage`   | Extracts the offer's page from `offer_details_raw` into JSON with the retailer's architecture and stores it in `offer_details_extracted`.                                                                                                  | `OfferPageWasExtracted`                                  | `normalize_offer`: queue `NormalizeOffer`                             |
| 7    | `NormalizeOffer`     | Normalizes the extracted JSON with the retailer's normalizer and stores it in `offer_details_normalized`. When the retailer produces no document for the page, stores nothing.                                                             | `OfferWasNormalized`; none when there is no document     | `structuralize_offer`: queue `StructuralizeOffer`                     |
| 8    | `StructuralizeOffer` | Structuralizes the normalized document with the retailer's structuralizer and stores it in `offer_details_structuralized`. When the retailer produces no document, stores nothing.                                                         | `OfferWasStructuralized`; none when there is no document | `processing_for_export`: queue `ProcessOfferForExport`                                                              |
| 9 | `ProcessOfferForExport` | Prepares a pinned immutable export of priced variations, recording missing-price exclusions. | `OfferExportWasPrepared` | none yet |

Steps 6 to 8 fail the command when the input row is missing, when the retailer
has no such step, or when the step fails, including a retailer whose step is
still a `todo!()` stub; the bus retries a failed command on a later run. Only
`ankerau` has every step implemented so far. Each of their tables holds one row
per offer, replaced when the step runs again.

All seven pipeline tables keep metadata in PostgreSQL and payloads in object
storage. Raw bodies are uploaded unchanged; JSON documents are uploaded as
compact JSON. Writes upload first, then atomically insert or update the local
reference. A failed local write can leave an unreferenced immutable object.

Every row a crawl writes in steps 1 to 3 carries its `retrieval_id`. Each step
logs its outcome (see [Logging](#logging)); `command_entries` and
`event_entries` show what is queued, done or failed.

## Code layout

```
src/
  main.rs                    routes, startup; registers each context
  platform/                  plumbing that knows nothing about the domain
    bus.rs                   command and event bus over Mulac
    bus_api.rs               POST /api/commands/run and /api/events/run
    db.rs                    Postgres pool and embedded migrations
    http.rs                  blocking HTTP client
  sitemap_discovery/         context: sitemaps → sorted URLs
    retrieve_sitemap/        step 1, plus POST /api/sitemap-retrievals
    process_sitemap/         step 2
    consolidate_sitemap/     step 3
  offer_discovery/           context: product URLs → offer data
    discover_offers/         step 4
    download_offer_page/     step 5
    extract_offer_page/      step 6
    normalize_offer/         step 7
    structuralize_offer/     step 8
    processing_for_export/   step 9
migrations/                  Mulac's tables, then each context's tables
```

A slice holds `command.rs`, `handler.rs`, `event.rs`, `entity.rs` (domain
values), `store/record.rs` (PostgreSQL representations and conversions),
`store.rs` (its own queries), and `reacts_to.rs` when a reaction starts it. Each
context and slice exposes only its `prelude` module, with `register()`, any
`routes()`, and the events others may use. Contexts talk to each other only
through events.

## Running it

The services workspace uses the local Mulac checkout next to `monorepo`, at
`../mulac` relative to the repository root. Revision
`9474319fff2237d81828a23d97df5d2f97741858` was verified for this integration.
Cargo path dependencies compile the current checkout; they do not pin a Git
revision. Keep that checkout available in development and build environments.
See [ADR 17](../../docs/adr/0017-retailer-sourcing-local-mulac.md).

The service needs Postgres for the workflow; without `DATABASE_URL` it serves
HTTP only and the routes below return `503`. On startup it applies pending
migrations. Configure `DATABASE_URL` for the service and `DATABASE_URL_TEST` for
the separate test database. The Make migration targets use `DATABASE_URL`
directly with Diesel CLI.

When `DATABASE_URL` is configured, `OBJECT_STORAGE_URL` and
`OBJECT_STORAGE_TOKEN` are required. Configuration is checked before connecting
to PostgreSQL or applying migrations. The client uses REST, a 30-second request
timeout, and no automatic retries or redirects. Start an object storage service
that allows the seven existing pipeline buckets and `offer_export`. Without
`DATABASE_URL`, the HTTP-only mode needs no object storage configuration.

```sh
make .env          # once: database and object storage configuration
make migrate       # optional; startup also migrates
PORT=3002 make serve
```

`make migrate-redo` reverts and reapplies every migration, which deletes the
data. With migration 11 applied, reverting requires removing new raw sitemap
rows first; see the cutover instructions below.

When the service runs in the workspace's `compose-apps.yaml` container, rebuild
and restart it after code changes, from the workspace folder:

```sh
make services-restart
```

Events are delivered to the reactions registered in the running binary. An event
processed by an older build that had no reaction for it is marked done and isn't
delivered again.

## Driving the workflow

Nothing processes the queues in the background. `Bus::start` assembles the
kernel with zero inline drain rounds and passes its existing consumers to
Mulac's Axum command/event adapters. Queue retrievals, then run commands and
events in turns:

| Route                          | Body                          | Does                                                                      |
| ------------------------------ | ----------------------------- | ------------------------------------------------------------------------- |
| `POST /api/sitemap-retrievals` | `{"retailers": ["ugreenca"]}` | Queues a `RetrieveSitemap` per retailer, returns each `retrieval_id`      |
| `POST /api/commands/run`       | `{"limit": 10}`               | Runs up to `limit` queued commands and waits for them                     |
| `POST /api/events/run`         | `{"limit": 10}`               | Runs up to `limit` queued events; their reactions queue the next commands |

One retailer from start to finish takes alternating runs:

1. `sitemap-retrievals` with the retailer.
2. `commands/run`: retrieves the sitemaps (can take minutes; keep the limit small).
3. `events/run`, then `commands/run`: processes the sitemaps.
4. `events/run`, then `commands/run`: consolidates them.
5. `events/run`, then `commands/run`: discovers the offers.
6. `events/run`, then `commands/run` until it reports `"processed": 0`: downloads the offer pages.
7. `events/run`, then `commands/run`, three more times: extracts, normalizes and structuralizes the offers.
8. `events/run`, then `commands/run`: prepares immutable `offer_export` snapshots.
9. `events/run`: dispatches `OfferExportWasPrepared`; no outbox listener is registered yet.

Run responses look like `{"processed": 3, "errors": []}`. When something fails,
`processed` is `null` and `errors` lists the failures; failed commands are
retried on a later run.

Both runners require a JSON object with an integer `limit` from 1 through 1000;
unknown fields are ignored. They now use Mulac's shared validation and error
contract:

| Response | Meaning |
| --- | --- |
| `200`, JSON `RunResponse` | Batch finished. Reservation, storage, handler, and subscriber failures appear in `errors`; successful items remain committed. |
| `400`, plain text | Invalid JSON or limit, including missing, null, negative, fractional, or wrongly typed values. |
| `413`, plain text | Body exceeds Axum's configured size limit. |
| `415`, plain text | Missing or non-JSON content type. |
| `500`, plain text `queue processing task failed` | The blocking task failed; internal panic/join details are hidden. |
| `503`, JSON `{ "error": "the command bus is not running; start the service with DATABASE_URL" }` | A valid request reached an unconfigured bus. |

Request validation runs before the unavailable-bus check. This changes the old
JSON range-error body to plain text and typed-JSON `422` errors to `400`.
Successful batch responses, route paths, and authentication are unchanged.
OpenAPI publishes one shared `RunRequest`/`RunResponse` pair for both queues.

No queue schema migration is required for this upgrade. Existing command/event
names, payloads, metadata, and retry state remain compatible. Queue progress
still requires explicit runs; neither an automatic stale-reservation sweep nor
a worker is started. Rows left reserved by an interrupted process require
operator recovery before they can run again. Graceful shutdown drains HTTP
requests before cancelling the kernel.

The Bruno collection in `../../bruno/retailer_sourcing` has a request for each
route.

## Logging

Logs go to stderr and to daily files under `logs/` (set `LOG_DIR` to change the
folder; it's git-ignored):

| File                               | Contains                                                                                                                       |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `retailer_sourcing.log.YYYY-MM-DD` | The same lines as stderr: each step's outcome, skipped sitemap files, failed downloads and commands, one line per HTTP request |
| `timing.log.YYYY-MM-DD`            | How long each fetch, command and reaction took, and whether it succeeded                                                       |

`RUST_LOG` sets the level, for example `RUST_LOG=debug`; the default is `info`.
In the compose container the files are written inside the container, so mount a
volume on the log folder to keep them.

## Tests

`make ci` runs formatting, clippy and the tests. Handler tests use in-memory
fakes, so they need no database or network. `make test-postgres` checks store
queries and transactions in isolated schemas; see the database test setup below.
Queue HTTP tests cover shared validation, schema reuse, batch isolation, partial
failure, and sanitized task errors. PostgreSQL queue tests insert the original
wire format directly, including null metadata and prior errors, then verify
command → event → next-command progression, metadata preservation, and retries
through the actual HTTP routes.

## Domain entities and PostgreSQL records

All three sitemap discovery slices and all five offer discovery slices follow
[ADR 7](../../docs/adr/0007-domain-entities-and-postgres-records.md). Each
slice's `entity.rs` contains domain types; `store/record.rs` contains private
Diesel types ending in `PgRecord` and their explicit conversions. Conversions
that parse stored JSON or check integer bounds return errors. `PgStore` takes
entities when writing and IDs when reading, and returns entities when loading.
Each `store.rs` declares its private `record` module with `mod record;`. Only
the store handles PostgreSQL records. Queries remain owned by the slice;
entities and records are not imported from another slice.

- Retrieval passes a batch of `RetrievedSitemapDocument` entities, each carrying
  its ID, retrieval ID, timestamp, and the sitemap library's raw document. The
  store returns `StoredSitemapDocument` entities exposing `object_storage_id`.
- Processing reads slice-local `RawSitemapDocument` entities exposing the local
  ID, `object_storage_id`, and the library's raw document, and writes a
  `ProcessedSitemap`. Its URL count is a `usize`; conversion to the stored `i32`
  happens in `store/record.rs`. Negative stored raw sizes are rejected on reads.
- Offer discovery reads a `ConsolidatedSitemap` entity containing product URLs.
- Downloading writes a `RawOffer` entity containing the offer ID, retailer, URL,
  and downloaded body.
- Extraction reads `RawOffer` and writes `ExtractedOffer`; normalization reads
  `ExtractedOffer` and writes `NormalizedOffer`; structuralization reads
  `NormalizedOffer` and writes `StructuralizedOffer`. These are slice-local
  projections identified by the domain offer ID. JSON documents remain typed
  JSON values until the handler calls a retailer step that expects text.

Offer row IDs remain stable across upserts. Each replacement uploads a new
immutable object with its own external ID, then updates the row's complete
storage reference. Stores return `Stored<T>` with the entity and
`object_storage_id`; input entities also expose the loaded object ID. Command
and event payloads remain unchanged, and writes must succeed before events are
returned.

`ConsolidatedSitemap` owns five count fields, calculated from its grouped links
at construction and exposed through read-only accessors. Its PostgreSQL record
stores them in `sitemaps_consolidated.stats` as a JSONB object:

```json
{"product_count":12,"catalog_count":3,"content_count":7,"not_interested_count":2,"unknown_count":0}
```

The same record stores source references and storage information in two more
non-null JSONB columns, as recorded in [ADR 8](../../docs/adr/0008-consolidated-sitemap-jsonb-references-and-storage.md):

```json
{
  "references": {
    "processed_sitemap_id": "019a1234-5678-7000-8000-000000000001",
    "retrieval_id": "019a1234-5678-7000-8000-000000000002",
    "retailer_code": "ugreenca"
  },
  "storage": {"content_size": 123, "object_storage_id": "019a1234-5678-7000-8000-000000000003"}
}
```

These four fields exist only inside JSONB, with no standalone or generated
columns. A unique expression index on the processed UUID in `references` keeps
one consolidated record per processed sitemap. There is no foreign key to the
processed row: deleting it now leaves the consolidated record and payload
intact. Deleting local records does not delete immutable objects.

Grouped links live in object storage. Event fields retain their existing names
and types. Counts and content size remain limited to the previous `i32` storage
range for downgrade support. In SQL, quote the reserved column name as
`"references"`.

### Storage metadata

All seven main `sitemaps_*` and `offer_details_*` tables use typed `storage
JSONB`. Sitemaps use `ObjectStorageMetadata { content_size: u64,
object_storage_id: Uuid }`. Offers use `OfferStorageMetadata` with the same
fields plus `external_id: Uuid`:

```json
{"content_size": 123, "object_storage_id": "019a1234-5678-7000-8000-000000000001", "external_id": "019a1234-5678-7000-8000-000000000002"}
```

`content_size` measures the exact uploaded UTF-8 bytes. The old `body_size` and
`document_size` columns and all seven payload tables are removed. JSONB uses
`Json<T>` with required fields and concrete types; unknown fields are rejected.
Consolidation references and stats retain their typed structures. Existing
stage-specific signed-integer limits still apply. Retailer-specific payload JSON
keeps its existing representation.

Buckets match table names: `sitemaps_raw`, `sitemaps_processed`,
`sitemaps_consolidated`, `offer_details_raw`, `offer_details_extracted`,
`offer_details_normalized`, and `offer_details_structuralized`. Legacy singular
sitemap buckets remain available in object storage.

Sitemaps use the local row UUID as the object external ID. Offer writes generate
a fresh external UUIDv7 each time, preserving the local row ID and `offer_id`.
Even an unchanged repeated offer write creates a new object. Concurrent writes
retain the last committed complete reference. HTTP retries of a prepared upload
use its existing external ID; durable replay across command attempts is not
added.

All downstream readers fetch bodies through the client and validate record ID,
bucket, external ID, and byte length. JSON payloads are then parsed. Database
connections are released before HTTP. Missing or invalid objects fail the
operation; missing local inputs retain their previous not-found behavior.
Responses and export formats are unchanged. See [ADR 12](../../docs/adr/0012-pipeline-object-storage.md).

### Raw sitemap objects

Raw sitemap metadata uses `Json<ObjectStorageMetadata>`, with required typed
`content_size: u64` and `object_storage_id: Uuid` fields:

```json
{"content_size": 123, "object_storage_id": "019a1234-5678-7000-8000-000000000001"}
```

Retrieval uploads the exact UTF-8 body through `object_storage_client` to bucket
`sitemaps_raw`. The preallocated local `sitemaps_raw.id` is the client's
`idempotency_key` (the object's `external_id`). The returned UUID is stored in
`storage.object_storage_id` and exposed in retrieval and processing entities.
After every upload succeeds, all local metadata rows are inserted in one
PostgreSQL transaction. A failed upload or commit can leave immutable objects
without local references; there is no distributed transaction or cleanup API.
Only a successful local commit produces `SitemapRetrieved`. Retrying the same
IDs is idempotent remotely; a fresh retrieval generates new IDs.

Downstream sitemap readers retrieve bodies by object ID and validate the
returned ID, bucket, external ID, and byte length. They release database
connections before HTTP requests. Missing or invalid objects fail the operation.
The API and export formats are unchanged. See [ADR 11](../../docs/adr/0011-raw-sitemap-object-storage.md).

### Applying the migrations

Migration `2026-10-16-000012_pipeline_object_storage` deletes existing processed
and consolidated sitemap rows and all four offer-stage rows, then drops their
six payload tables. It preserves raw sitemap rows, their object references, and
bus history. There is no backfill or legacy fallback.

Deploy the new object storage bucket allowlist first. Stop sourcing workers,
apply the migration (or start the new binary, which migrates), restart, and
initiate fresh retrievals to rebuild the pipeline. Old queued work referencing
deleted records may fail; the migration does not remove or rewrite bus entries.
Rollback recreates empty payload tables only when all six affected metadata
tables are empty. Deleted data cannot be recovered. Revert migration 12 before
following the older instructions below.


Migration `2026-10-15-000011_raw_sitemap_object_storage` is a destructive
fresh-data cutover: it deletes existing `sitemaps_raw` rows and drops
`raw_sitemap_documents_data`, without backfilling objects. Processed,
consolidated, and offer records remain intact. Stop old service instances before
applying it or starting the new binary, then retrieve fresh sitemaps. Existing
crawls no longer have source bodies; exporting them requires a fresh retrieval.
Rollback recreates only the empty payload table and refuses to run while new raw
rows exist. It cannot recover deleted bodies. Revert migration 11 before
following the older rollback instructions below.

Migration `2026-10-15-000010_pipeline_storage` preserves the existing raw and
processed size values and calculates previously unrecorded offer-stage sizes
from compact stored JSONB, retaining whitespace in strings and PostgreSQL's
number representation. It preserves IDs, timestamps, relationships, and
payloads. Missing offer-stage payloads abort the migration transactionally. Stop
old service instances before applying it with `make migrate` or starting the new
binary. To roll back, stop the new service and run `diesel migration revert`
while migration 10 is latest. It restores the previous size columns; missing,
non-integer, or overflowing JSON sizes cause rollback to fail transactionally.
The three offer JSON stages lose their new size metadata on downgrade, and
regain it on upgrade. Revert migration 10 before following the older rollback
instructions below.

Migration `2026-10-15-000009_rename_pipeline_tables` renames the seven pipeline
tables to `offer_details_raw`, `offer_details_extracted`,
`offer_details_normalized`, `offer_details_structuralized`, `sitemaps_raw`,
`sitemaps_processed`, and `sitemaps_consolidated`. It preserves rows, IDs,
indexes, and foreign keys; companion `_data` tables retain their existing names.
Stop old service instances before applying it with `make migrate` or starting
the new binary. To roll back the renames, stop the new service and run `diesel
migration revert` while migration 9 is latest, then start the old binary. Revert
migration 9 before following the older rollback instructions below.

Stop existing service instances before running `make migrate` or starting the
new binary (which applies pending migrations). Migration
`2026-10-15-000007_consolidated_sitemap_stats` backfills stats from the count
columns. Migration `2026-10-15-000008_consolidated_sitemap_references_storage`
backfills the references and storage objects before removing the four columns.
Both preserve existing values, IDs, timestamps, and payloads. The second
migration intentionally removes the processed-sitemap foreign key. Old binaries
cannot run against the new schema.

For rollback to the previous stats-only layout, stop the new service and, while
migration 8 is the latest applied migration, run `diesel migration revert` from
this directory with `DATABASE_URL` configured. Then start the previous binary.
Reverting migration 7 as well restores the original count columns. Do not use
`make migrate-redo` for rollback: that target reverts all migrations and deletes
stored data. Missing, malformed, or overflowing values cause rollback to fail
transactionally. Dangling processed references must be resolved before reverting
migration 8, because rollback restores the original foreign key.

### PostgreSQL tests

```sh
make fmt
make ci
DATABASE_URL_TEST=postgres://object_storing:object_storing@172.28.100.31:5432/object_storing_test RUST_TEST_THREADS=4 make test-postgres
```

The test database must already exist, and its role must be able to create/drop
schemas. Each test creates a unique schema and removes it afterward, including
on assertion failure; it never resets existing tables. The target fails when
configuration or connectivity is missing. Ordinary `make test` leaves these
PostgreSQL tests ignored. Tests cover entity-facing reads and writes in every
sitemap and offer slice, missing inputs, conversion failures, retrieval
ordering, Unicode byte sizes, storage metadata on insert/update, batch rollback,
stable upsert IDs, failed insert/update rollback, fresh migrations, populated
upgrade/downgrade/re-upgrade, failed downgrades, the export query, reference
uniqueness, and deletion behavior.

### Real object storage integration test

With a running object storage service and the separate test database:

```sh
DATABASE_URL_TEST=postgres://object_storing:object_storing@172.28.100.31:5432/object_storing_test \
OBJECT_STORAGE_URL=http://localhost:3108 OBJECT_STORAGE_TOKEN=<token> \
make test-object-storage
```

These opt-in tests cover all seven stages, compare input bodies/documents with
real service output, and check JSONB references, exposed IDs, and stable offer
row IDs across replacements. Local schemas are removed afterward; immutable
remote test objects remain. Use a test-backed object storage instance: the
normal service at `172.28.100.43:3000` uses the development database. A
temporary instance can run against an isolated schema in `object_storing_test`.

Ordinary PostgreSQL tests use a local HTTP fixture for upload and read failures,
failed local commits, concurrent offer replacements, migration guarantees, and
source API responses. Limit test concurrency with `RUST_TEST_THREADS=4` on the
shared test server. `make ci` also checks exported content and event suppression
when storage fails.

If queuing a sitemap request fails partway through, its HTTP 500 response
includes `error` and the `retrievals` already queued. These remain queued; retry
only retailers not listed in the response.

### Migration review limitations

The combined migration batch is retained for review, not a promise of a safe
upgrade from populated legacy databases. Migration 000005 adds a required
retailer column without a backfill, so existing raw offers cause it to fail.
Migration 000006 drops inline payloads without copying them. Migrations 000011
and 000012 deliberately delete pipeline rows. Review the data transition before
deploying; ordinary CI and fresh-schema tests do not establish populated-upgrade
safety.

## Downloaded source APIs

`GET /api/development/sitemaps/sources` returns saved source bodies resolved
through object storage. `GET /api/development/offer-details/sources` returns
only database metadata, newest first: `id`, `offer_id`, `url`, `retailer_code`,
and `storage` (`content_size`, `object_storage_id`, `external_id`). It does not
contact object storage or return a `body`. Clients retrieve the HTML with `GET
/api/buckets/offer_details_raw/records/{object_storage_id}` on object storage
using that service's bearer token. Both sourcing routes require the service
bearer token and accept `limit` and `offset`. Limits must be 1–100 for sitemaps
and 1–1000 for offer pages. Missing or invalid credentials return 401; invalid
limits return 400; without a database the endpoints return 503. OpenAPI
documents the bearer scheme.

## Exporting offer pages

`make export-offer-sources` writes every page in `offer_details_raw` to
`<DATA_DIR>/<retailer>/offer_details/sources/<url path>.html`, the layout of the
per-retailer data repos. `DATA_DIR` defaults to `../../../data`, the `data`
folder next to the monorepo. A page's file is named after its decoded URL path
(`au/products/a1256` becomes `au-products-a1256.html`) and is overwritten on the
next export.

Both `DATABASE_URL` and the object-storage connection settings are required.

## Exporting sitemap crawls

`make export-sitemaps` writes each retailer's most recently processed sitemap
crawl to `<DATA_DIR>/<retailer>/sitemap`, one folder per stage, each named for
the crawl's retrieval id:

- `sources/<retrieval id>/<url path>.xml`: every sitemap file as fetched.
  Names follow the offer pages' rule, plus the query with `&` replaced by `_`,
  since Shopify pages its sitemaps by query (`sitemap_products_1.xml?from=1&to=2` becomes
  `sitemap_products_1_from=1_to=2.xml`).
- `processed/<retrieval id>.json`: the files parsed into one sitemap tree.
- `consolidated/<retrieval id>.json`: the tree's URLs grouped by kind of page,
  once the crawl has been consolidated.

Exporting the same crawl again overwrites its files. A newer crawl gets its own
folder and files, so earlier exports stay alongside it.

Both `DATABASE_URL` and the object-storage connection settings are required.

## Database reporting

`GET /api/reporting/database-tables` requires the existing service bearer token.
It reports ordinary tables in the connection's active schema
(`current_schema()`), including migration and bus tables where present; other
schemas and views are excluded. The typed response has this shape:

```json
{
  "schema_name": "public",
  "generated_at": "2026-10-17T12:00:00Z",
  "row_count_kind": "estimated",
  "tables": [
    {
      "table_name": "example",
      "row_count": 120,
      "table_bytes": 32768,
      "index_bytes": 16384,
      "total_bytes": 49152
    }
  ]
}
```

Tables are returned in name order. Row counts use PostgreSQL's estimated live
rows and can lag recent writes; this report performs no row scans or ANALYZE.
Table bytes include TOAST and auxiliary storage; index bytes cover attached
indexes. Total bytes equal table plus index bytes. Successful responses use
`Cache-Control: no-store`. Unavailable databases return 503; unexpected failures
return 500, without database diagnostics.

The [development dashboard](../../apps/development/README.md) consumes this API
through private server-side clients. No reporting client is exported by a shared
library. The route is covered by normal HTTP tests and `make test-postgres`.

`GET /api/reporting/entry-statuses` uses the same bearer authentication and
error responses. It returns exact counts from `command_entries` and
`event_entries`, grouped by numeric status and sorted by status:

```json
{
  "generated_at": "2026-10-18T12:00:00Z",
  "command_entries": [{"status": 0, "status_text": "received", "count": 12}],
  "event_entries": [{"status": 5, "status_text": "completed", "count": 8}]
}
```

Both queues are counted in one database statement, using the same snapshot. Only
statuses with entries are returned; empty queues return empty arrays. Mulac
status labels are `received` (0), `reserved` (2), `failed` (4), `completed` (5),
`dead` (7), and `archive` (8). Unrecognized codes retain their numeric value
with `status_text: "unknown"`. Counts scan the queues and may cost more than the
estimated database table report. Responses use `Cache-Control: no-store`; the
dashboard refreshes them on each page load.

## Processing for export

`processing_for_export` consumes the immutable reference included in new
`OfferWasStructuralized` events. It never resolves a delayed event through the
latest `offer_details_structuralized` row. Older events still deserialize, but
the resulting export command fails with `run fresh structuralization before
exporting`; start a fresh retrieval to emit new pinned source references.
No latest-snapshot fallback is performed.

Each priced variation becomes an `ImportOfferDetails`-compatible payload with
retailer code, title, exact variant URL and integer minor-unit price/currency.
Missing prices are recorded under `exclusions` with the variation index, URL
and `missing_price` reason. Unavailable stock is included when priced. Blank
titles, malformed money, negative prices, invalid HTTP(S) URLs and repeated
exact variation URLs fail preparation rather than producing ambiguous exports.
Empty or wholly excluded documents produce a completed export with no entries.

Migration `2026-10-24-000013_offer_export` adds only the export table. Its unique
key is `(offer_id, source_object_id, schema_version)`. A preparation persists
stable snapshot and entry UUIDs, typed source/stats metadata, and the exact
pending body before upload. `status = 'prepared'` rows retain `prepared_body`
when upload or completion fails. The next command attempt resumes those same
bytes and IDs. On completion, `status = 'completed'` records the storage
reference and completion time and clears `prepared_body`. Completed documents
are immutable artifacts in the `offer_export` bucket, using the snapshot UUID
as their idempotency key. Deploy the object-storage allowlist first.

`OfferExportWasPrepared` references the completed snapshot and its source.
`offer_discovery::prelude::ExportStore::load` reads a completed domain snapshot
and validates its object reference. Repeated events may reference the same
export; downstream delivery must use the stable per-entry UUIDs to deduplicate.
No outbox rows or HTTP requests to offers are produced by this stage. Rollback
removes local export metadata and leaves already uploaded objects.

`make ci` covers fixture conversion and guards. `make test-postgres` covers
source replacement, concurrent preparation, crash recovery, additive migration
behavior and separate HTTP queue batches. `make test-object-storage` also
checks the new bucket against a running object-storage service. Offers' unit
tests deserialize the same checked-in import fixture through its real command.
See [ADR 19](../../docs/adr/0019-immutable-offer-export-preparation.md).
