# object_storage

Immutable UTF-8 artifacts in a separate PostgreSQL database, with authenticated
REST and POST-only GraphQL APIs. Implements [the storage specification](../../docs/object-storage.md).
Each record owns a fresh data row at creation; deduplication, retention, and
retailer sourcing integration are deferred.

## Run locally

Requires PostgreSQL 16 or later for native JSONB input validation. Use the
existing `object_storing-db` infrastructure in the parent workspace's
`compose-infra.yaml`. This service is named `object_storage`; the database and
credentials retain the existing `object_storing` names. No Compose changes are
needed. The container has no published port, so the example requires access to
its fixed address on the `bodul` network. Override the URLs for another host.

```sh
make .env
PORT=3003 make serve
```

The Makefile reads `.env`; shell values take precedence. `DATABASE_URL` is
mandatory and is also used by the Diesel migration targets. Startup checks
connectivity, PostgreSQL version, and UTF8 database encoding, then applies
embedded Diesel migrations before opening the HTTP listener. The database must
be dedicated to this service. `PORT` defaults to 3000 and must be a valid `u16`.
Ctrl+C and SIGTERM drain in-flight HTTP requests. Logs go to stderr and daily
`object_storage.log.YYYY-MM-DD` files under `LOG_DIR` (default `logs`), with
`RUST_LOG=info` by default. Request bodies and credentials are not logged. Client-facing errors omit
database diagnostics and artifact contents.

## Migrations

The reversible Diesel migration in `migrations/00000000000000_create_records`
creates `data` and `records`, including the `external_id` column, its per-bucket
uniqueness constraint, and the data foreign key. Its down migration drops both
tables.

Install Diesel CLI with PostgreSQL support if needed:

```sh
cargo install diesel_cli --no-default-features --features postgres --locked
```

From this service directory, configure `DATABASE_URL` in `.env` or your shell,
then run:

```sh
make migration       # apply pending migrations (alias: make migrate)
make migration-redo  # revert and reapply all migrations (alias: make migrate-redo)
```

Redo **deletes all stored records and data**. Both commands require
`DATABASE_URL`; service startup also applies pending migrations automatically.

## Authentication and routes

Every route, including health, OpenAPI, Swagger assets, GraphiQL, and unknown
paths, requires the same service-specific development bearer token:

```sh
export OBJECT_STORAGE_TOKEN=6fe0382d64bba74e205af86e397d902c0f5e6f3d8e15b1a4f87c6a309d752eb1
export OBJECT_STORAGE_URL=http://localhost:3003
curl -H "Authorization: Bearer $OBJECT_STORAGE_TOKEN" "$OBJECT_STORAGE_URL/api/health"
```

This credential is hard-coded in `src/lib.rs` and public to repository readers;
it is a development access check, not a production secret. Authentication runs
before request-body parsing. Failures return `401`, `WWW-Authenticate: Bearer`,
and RFC 9457 Problem Details.

| Route | Purpose |
| --- | --- |
| `GET /api/reporting/database-tables` | Active schema table sizes and estimated row counts |
| `GET /api/health` | Liveness (`ok`); does not query storage |
| `POST /api/buckets/{bucket}/records` | Create or retry |
| `GET /api/buckets/{bucket}/records/{id}` | Retrieve within a bucket |
| `POST /graphql` | GraphQL query or mutation; other methods return 405 |
| `GET /api/openapi.json` | OpenAPI 3.1 with response and bearer documentation |
| `GET /api/docs/` | Vendored Swagger UI (`/api/docs` redirects here) |
| `GET /graphql/ide` | GraphiQL in debug builds; authenticated 404 in release |

For browser documentation, use a browser header-injection tool or a local proxy
that supplies Authorization on the document and every asset request. Swagger's
Authorize button alone cannot authenticate the initial page load. Set the same
Authorization header in GraphiQL's request headers panel for its API requests.

## REST example

Generate a UUIDv7 idempotency key on the client and keep it for retries. For this
example, reuse the fixed key below only for this artifact:

```sh
curl -i "$OBJECT_STORAGE_URL/api/buckets/sitemap_raw/records" \
  -H "Authorization: Bearer $OBJECT_STORAGE_TOKEN" \
  -H 'Content-Type: application/json' \
  --data '{"idempotency_key":"019a1234-5678-7000-8000-000000000001","payload":"<urlset/>","metadata":{"source":"example"}}'
```

Creation returns `201`, `{"id":"<record UUIDv7>"}`, and a relative `Location`.
An identical retry returns `200`, the same ID and Location. Different payload or
metadata for the same bucket/key returns `409`. Unexpected creation fields,
missing fields, null metadata, unknown buckets, and non-v7 UUIDs return `400`.
Use `Location` for retrieval:

```sh
curl -H "Authorization: Bearer $OBJECT_STORAGE_TOKEN" \
  "$OBJECT_STORAGE_URL/api/buckets/sitemap_raw/records/<record UUIDv7>"
```

Retrieval returns `id`, `bucket`, `idempotency_key`, `payload`, `metadata`, and
`created_at` in UTC with a `Z` suffix. The internal data ID is never exposed.
All REST failures use `application/problem+json` with `type`, `title`, `status`,
and `detail`. Storage outages return `503`; unexpected failures return `500`.

## GraphQL example

```sh
curl "$OBJECT_STORAGE_URL/graphql" \
  -H "Authorization: Bearer $OBJECT_STORAGE_TOKEN" \
  -H 'Content-Type: application/json' \
  --data '{"query":"mutation($input:CreateRecordInput!){createRecord(input:$input){id}}","variables":{"input":{"bucket":"sitemap_raw","idempotencyKey":"019a1234-5678-7000-8000-000000000001","payload":"<urlset/>","metadata":{"source":"example"}}}}'
```

Retrieve with `query($id:ID!){record(bucket:"sitemap_raw",id:$id){id payload
metadata createdAt}}` and an `id` variable. The schema uses `JSON` and `DateTime`
scalars and has no GraphQL health field.

GraphQL responses use `application/graphql-response+json`. Request errors return
400, an `errors` array with codes, and no `data`: `GRAPHQL_PARSE_FAILED`,
`GRAPHQL_VALIDATION_FAILED`, `OPERATION_RESOLUTION_FAILURE`, or `BAD_USER_INPUT`.
The adapter validates the document and all selected-operation variables in a
resolver-free preflight pass. A second pass executes only valid requests, so an
invalid variable on a later mutation cannot leave earlier mutations committed.
Nullable retrieval failures return `record: null` plus an error with its path
and location; other selected fields remain available. Execution errors use HTTP
200 with `BAD_USER_INPUT`, `NOT_FOUND`, `CONFLICT`, `SERVICE_UNAVAILABLE`, or
`INTERNAL_SERVER_ERROR`. Malformed JSON and unsupported media types are rejected
before GraphQL processing using Problem Details (400 and 415 respectively).

## Storage behavior

Both APIs call the same validators and store. Database work runs on Tokio's
blocking pool, using Diesel's PostgreSQL connection pool. New records and data
are inserted in a single transaction. A uniqueness race rolls back the losing
data insert before comparing against the committed winner. The database enforces
`UNIQUE (bucket, external_id)` and the data foreign key. The database's
`external_id` column is exposed as `idempotency_key` in REST and `idempotencyKey`
in GraphQL. `data_id` is
intentionally non-unique for future deduplication.

Payload comparison is exact after decoding JSON strings. Metadata object key
order is ignored and array order is preserved. PostgreSQL JSONB equality makes
`1`, `1.0`, and `1e0` equivalent. Numbers retain decimal precision through REST,
GraphQL literals/variables, storage, and responses. There are no application
payload, metadata, or HTTP body-size caps; axum's default body limit is disabled.
PostgreSQL's inherent restrictions still apply: NUL characters and numbers
outside its numeric range produce sanitized input errors. Encode binary payloads
as text (for example Base64) before sending them.

## Tests

```sh
make fmt
make ci                   # unit/router tests; no database needed
make test-postgres        # requires DATABASE_URL_TEST; fails if unavailable
cargo test --release --test http  # checks release GraphiQL availability too
```

`make .env` configures `DATABASE_URL_TEST` for the existing `object_storing_test`
database. Use a PostgreSQL URL for a dedicated test database, whose role can
create/drop schemas. Each integration test migrates a random schema and removes
it afterward; existing tables and records are untouched. Ordinary `make test`
marks database tests ignored instead of silently claiming to run them.

The database suite covers both APIs, all buckets, concurrent matching and
conflicting REST/GraphQL requests, independent payload rows, rollback, exact
metadata numbers, Unicode, unsupported values, request preflight, and bodies
larger than framework defaults. Router tests cover authentication before parsing,
media types, error codes/envelopes, documentation, health, unavailable storage,
POST-only GraphQL, and debug/release GraphiQL. From `services`, run `make ci` to
check every service and `make audit` after changing Cargo dependencies.

## Clients

[`object_storage_client`](../../lib/object_storage_client/README.md) provides an
async Rust API for creation, retrieval, and health, using either REST or GraphQL.
The [Bruno collection](../../bruno/object_storage/README.md) includes authenticated
requests for both APIs, retries, health, OpenAPI, Swagger UI, and GraphiQL.

Retailer sourcing writes all pipeline payloads to buckets matching its table
names: `sitemaps_raw`, `sitemaps_processed`, `sitemaps_consolidated`, and the four
`offer_details_*` buckets. Sitemaps use their local row IDs as idempotency keys;
offer replacements use fresh external IDs while retaining stable local row IDs.
The legacy singular sitemap buckets remain available.

## Database reporting

`GET /api/reporting/database-tables` requires the existing service bearer token.
It reports ordinary tables in the connection's active schema (`current_schema()`),
including migration and bus tables where present; other schemas and views are
excluded. The typed response has this shape:

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
rows and can lag recent writes; reporting performs no row scans or ANALYZE.
Table bytes include TOAST and auxiliary storage; index bytes cover attached
indexes. Total bytes equal table plus index bytes. Successful responses use
`Cache-Control: no-store`. Unavailable databases return 503; unexpected failures
return 500, without database diagnostics.

The [development dashboard](../../apps/development/README.md) consumes this API
through private server-side clients. No reporting client is exported by a shared
library. The route is covered by normal HTTP tests and `make test-postgres`.
