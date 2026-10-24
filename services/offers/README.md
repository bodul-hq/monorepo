# Offers service

## Current server

The read-only reporting module exposes authenticated `GET /api/reporting/database-tables`
and `GET /api/reporting/entry-statuses`, matching retailer sourcing's JSON contracts.
Use `Authorization: Bearer <token>`; `OFFERS_REPORTING_TOKEN` defaults to
`offers-development-token`. The development dashboard uses `OFFERS_URL` and
`OFFERS_TOKEN` to fetch both reports server-side. Reports return `Cache-Control:
no-store`; missing or incorrect credentials return `401`, an unavailable database
returns `503`, and query failures return `500` without database diagnostics.
Table row counts are estimates from PostgreSQL statistics for the active schema;
command/event status counts are exact. Reporting does not run `ANALYZE` or mutate data.

`PORT=3001 make serve` starts Poem (default port: 3000). Set `DATABASE_URL` to
connect to PostgreSQL and apply the embedded migrations at startup. An invalid,
empty, missing, or unreachable database configuration fails startup.

When a database is configured, `main` also starts the Mulac kernel with PostgreSQL
persistence using the same pool and shuts it down when the HTTP server exits.
`start_mulac` registers the offer creation handler and builds a direct command
gateway for the API. Both read projectors subscribe to `OfferCreated`; the offer
projector also subscribes to `OfferTitleChanged` and `OfferPriceChangedEvent`.
No background queue workers are started. The queue APIs process one batch from
one queue at a time:

| POST endpoint | Effect |
| --- | --- |
| `/api/inbox/run` | Convert stored inbox messages to queued commands. |
| `/api/commands/run` | Execute queued commands and enqueue their events. |
| `/api/events/run` | Dispatch queued events to the projectors. |
| `/api/outbox/run` | Publish a batch to RabbitMQ when `AMQP_URL` is configured; otherwise return `503` without reserving rows. |

All four require JSON `{"limit":100}` with an integer from 1 through 1000.
Missing or invalid limits and malformed JSON return `400`. Configured consumers
return `200` with `{"processed":N,"errors":[]}` on success (including `N=0`
when no work is eligible), or `{"processed":null,"errors":["…"]}` for consumer
errors, including storage failures. A failed blocking task returns `500`.
Successful items in a partially failed batch remain committed. Mulac controls
reservations, scheduling, retry backoff, and the default six-attempt limit;
these routes do not reset failed or reserved rows. These operational routes
have no authentication, matching the direct import route.

To process stored inbound work, call inbox, then commands, then events.
Inbox metadata's `routing_key` names the command type, and its payload contains
the serialized command. The inbox route processes database rows; it does not
fetch messages from a broker. The `subscribers::customer_facing` subscriber records `OfferCreated`,
`OfferTitleChanged`, and `OfferPriceChangedEvent` in `outbox_entries` when events
are dispatched. Each row preserves the payload, event type, event ID, source,
and correlation/causation IDs, with routing key `customer_facing` and content
type `application/json`. Re-delivery of the same event ID does not insert a
second row. Missing event metadata or an outbox write failure fails the
subscriber so normal event retry handling applies. Projection and outbox writes
are not one transaction. `AppState` owns the inbox, commanding, eventing and
outbox adapters.

Mulac owns `/api/commands/run` behavior. `src/orchestration/commanding_api.rs`
only declares the route and delegates to `mulac_poem::io::CommandingAdapter`
(feature `commanding`). The shared `commanding_api` crate validates the limit
and runs one batch through the kernel's command consumer, so commands executed
over HTTP still record their events without dispatching them. A failed command
is reported as its `CommandError` text, for example
`no handler registered for command type '…'`, and is retried with backoff
until it becomes dead after six attempts. See Mulac's ADR-010.

Mulac also owns `/api/events/run` behavior. `src/orchestration/eventing_api.rs`
only declares the route and delegates to `mulac_poem::io::EventingAdapter`
(feature `eventing`). The shared `eventing_api` crate validates the limit and
dispatches one batch through the kernel's event consumer. The projectors and
the `customer_facing` subscriber stay registered with the kernel in
`start_mulac`, so the route dispatches to exactly the subscribers registered
there. A subscriber failure is reported as its `EventError` text, for example
`subscriber execution error: event metadata is required for customer-facing outbox delivery`,
and the event is retried with backoff until it becomes dead after six
attempts. Events of a type without subscribers complete and are counted.

Mulac also owns `/api/outbox/run` behavior. `src/orchestration/outbox_api.rs`
only declares the route and delegates to `mulac_poem::io::OutboxAdapter`
(feature `outbox`). The shared `outbox_api` crate validates the limit and
publishes one batch through an outbox consumer that
`OutboxService::from_postgres` builds from the pool and Mulac's
`ConfirmedAmqpPublisher` (`mulac-outbox` feature `amqp`). Without `AMQP_URL`,
`start_mulac` leaves the adapter
`unconfigured()`, so valid requests return `503`
`outbox publisher is not configured` without touching storage. Publication
failures are reported with their `OutboxError` text, for example
`transport error: RabbitMQ publication timed out`, and use Mulac's retry
schedule. `src/orchestration/rabbitmq.rs` reads and validates the environment
variables and builds the publisher; broker topology also stays in offers.

### Inbox submission

`POST /api/inbox` stores one generic message and returns `202 Accepted` with
`{"message_id":"<uuid>"}` once PostgreSQL accepts it. It requires a `payload`
JSON value and nonblank `routing_key` and `source` strings. Missing or null
payloads, blank strings, empty objects and empty arrays are rejected. Numbers
(including zero), booleans and nonempty containers are accepted; nested values
are not checked for emptiness. Optional `message_id` and `correlation_id` are
UUID strings. Omit `message_id` to generate a new UUID. Supplied metadata and
routing keys are preserved, including surrounding whitespace in nonblank strings.

Reusing a message ID returns `202` and preserves every column of the original
row, even when the submitted payload or metadata differs. Malformed JSON,
missing required fields, empty payloads, blank routing keys or sources, and
invalid UUIDs return `400`.
Storage and blocking-task failures return `500` without database diagnostics.
Unknown routing keys and arbitrary JSON payloads are accepted; command-specific
validation happens during processing. This route has no authentication, matching
the queue run routes, and does not start processing or read from a broker.

Mulac owns both inbox routes' behavior. `src/orchestration/inbox_api.rs` only
declares `POST /inbox` and `POST /inbox/run` and delegates to
`mulac_poem::io::InboxAdapter`. The shared `inbox_api` crate validates
requests and runs storage and processing; `InboxService::from_postgres` builds
the recorder and a consumer that only queues commands. Every queue route,
including `/commands/run` and `/events/run` through `CommandingAdapter` and
`EventingAdapter`, uses the adapter's
`RunRequest` and `RunResponse` types, so the OpenAPI schema names are
registered once. See Mulac's ADR-009 for the shared contract.

For example, submit the serialized `ImportOfferDetails` command (its payload
uses the retailer enum name `AnkerAu` and `price.amount`; the direct import
request uses the slug `ankerau` and `price_amount`):

```sh
curl -i http://localhost:3001/api/inbox \
  -H 'content-type: application/json' \
  -d '{"routing_key":"ImportOfferDetails","source":"manual-import","payload":{"retailer_code":"AnkerAu","title":"USB charger","price":{"amount":1999,"currency":"AUD"},"url":"https://example.com/charger"}}'

curl http://localhost:3001/api/inbox/run \
  -H 'content-type: application/json' -d '{"limit":100}'
curl http://localhost:3001/api/commands/run \
  -H 'content-type: application/json' -d '{"limit":100}'
curl http://localhost:3001/api/events/run \
  -H 'content-type: application/json' -d '{"limit":100}'
```

Each run advances only its own queue. The last step populates `read_offers` and
`read_offer_identities`; read HTTP routes are not active yet. In Bruno, send
**Submit inbox message**, **Run inbox**, **Run commands**, then **Run events**.

### RabbitMQ delivery

Set `AMQP_URL` to an `amqp://` or `amqps://` URL to enable outbox delivery.
`OUTBOX_EXCHANGE` defaults to the empty RabbitMQ exchange. Provision a durable
`customer_facing` queue in that virtual host first, or configure an existing
exchange with a durable queue bound using routing key `customer_facing`.
The service does not declare broker topology or read inbound RabbitMQ messages.
Inbox processing still reads the database inbox.

`POST /api/outbox/run` publishes only its requested batch. Messages are persistent,
mandatory, and use publisher confirms: only an ACK without a returned message
completes the row. Unroutable messages, NACKs, connection errors and ten-second
connection/publication timeouts fail the item and use Mulac's retry schedule.
Connections are opened lazily, reused, and discarded on publication failure.
A broker outage therefore does not prevent HTTP startup. Invalid URL configuration
does fail startup. Keep credentials out of logs and use TLS for remote brokers.

Payload, message/event IDs, event type, correlation/causation IDs, source, content
type and routing key are preserved. Delivery is at least once: if a confirmation
or the database completion write is lost, a retry may publish again. Receivers
must deduplicate by message ID. Startup migrations backfill missing/null message
IDs from existing event IDs. No background worker is started.

`AMQP_URL_TEST=... make test-rabbitmq` is a smoke test of offers' publisher:
confirmed persistent delivery, unroutable-message failure and recovery, using a
temporary exclusive queue. Mulac's `ConfirmedAmqpPublisher` tests cover the
publication semantics in detail.
`make test-postgres` also tests outbox completion and retry state with an injected
publisher, without requiring RabbitMQ.

After a successful `/api/tmp-import`, run the inbox, commands and events steps
to populate `read_offers` and `read_offer_identities`:

```sh
for queue in inbox commands events; do
  curl http://localhost:3001/api/$queue/run \
    -H 'content-type: application/json' -d '{"limit":100}'
done
```

`POST /api/tmp-import` accepts:

```json
{
  "retailer_code": "ankerau",
  "title": "USB charger",
  "price_amount": 1999,
  "currency": "AUD",
  "url": "https://example.com/charger"
}
```

The request is validated and converted directly into an inbox message whose
payload matches `ImportOfferDetailsCommand`, with routing key `ImportOfferDetails`
and source `offers-api`, the same as `POST /api/inbox` would store it. Success returns
`202` with `{"message_id":"<uuid>"}`; nothing is executed yet. `/api/inbox/run`
queues the command and `/api/commands/run` executes it through
`ProcessOfferHandler`, with the inbox's and command queue's retries. Invalid
input returns `400`; inbox storage failures return `500` with `inbox message
could not be stored`. The generated offer UUID is new for every import:
identity deduplication, update commands, and receipts are not wired yet.
Stored records use the version tags `OfferCreated.v1`, `OfferTitleChanged.v1`,
and `OfferPriceChanged.v1`. Existing records in earlier formats are not migrated.

Other active routes are `/api/health`, `/api/openapi.json`, `/api/docs/`, and
GraphQL health at `/graphql`, with `/graphql/ide` in debug builds. `make ci`
runs conversion, validation, and HTTP smoke tests. `DATABASE_URL_TEST=... make
test-postgres` verifies HTTP import persistence and storage-error handling in an
isolated schema, which it removes afterward. It also verifies the staged
inbox submission/command/event HTTP flow, payload and metadata preservation,
generated IDs, duplicate-ID behavior, batch limits, retry scheduling, projection
creation and updates, and that the unavailable outbox route leaves rows unchanged.

The sections below describe the earlier full-service plan; the queued import
submission and read lookup routes are not active in the current server. The
manual queue run routes described above are active.

Imports retailer page snapshots into a Testimony aggregate, publishes changes
through Mulac, and serves a PostgreSQL projection. The module boundaries follow
`/home/nulllabs/work/labs/bodulv6`:

| Module | Responsibility |
| --- | --- |
| `src/core.rs` | `RetailerOffer`, events, sequence tracking, replay. |
| `src/offers/offer_identity.rs` | Validated retailer and URL identity. |
| `src/offers/offer_page.rs` | Validated offer page snapshot. |
| `src/app.rs` | Import/project use cases and repository interfaces. |
| `src/infra/` | Versioned records, transactional PostgreSQL repositories, Mulac adapters. |
| `src/bootstrap.rs` | Database startup, migrations, shared runtime construction. |
| `src/api.rs` | REST DTOs, OpenAPI, GraphQL health, `app()` routes. |
| `src/main.rs` | Port configuration, server, graceful shutdown. |

See [the implementation plan](offers-plan.md), [bodulv6 research](testimony.md),
and [ADR 15](../../docs/adr/0015-offers-testimony-and-mulac.md).

## Read model repositories

`read_offer` and `read_offers_identifiers` define repository traits,
with `InMemory` and Diesel `Postgres` implementations exported from each
module's `repository`. Inserts reject duplicate UUIDs and exact retailer/URL
pairs; they never overwrite existing records. The offer repository stores all
entity fields in `read_offers`, independently of identifier insertion. The
identifiers repository uses `read_offer_identities`. The offer repository also updates
title and price independently, failing when the offer is missing. Its projector
preserves identity and creation time on updates; price changes replace both
amount and currency. Invalid event payloads and repository failures propagate
to Mulac. No read HTTP routes are exposed yet.

Creation inserts reject duplicates, including repeated delivery of the same
creation event. The two projector writes are not atomic: a partial failure can
leave one projection inserted, and a retry can then fail on that duplicate.
Update events carry no sequence, so updates apply in consumer delivery order.
The PostgreSQL contract tests run with
`make test-postgres`; in-memory contracts run with `make ci`.

## Requirements and startup

Use the repository's pinned Rust toolchain, GNU Make, libpq, and PostgreSQL 16
or later. Cargo also needs the configured `null-labs-hq` registry and the pinned
Mulac Git dependency (`0.3.0`). No separate Testimony process, object storage, or retailer sourcing process is
required. RabbitMQ is required only when outbox delivery is enabled.

```sh
make .env                 # optional; edit the commented examples
PORT=3001 make serve
```

`PORT` defaults to `3000`. `DATABASE_URL` is required. Missing, empty,
unreachable or invalid database configuration causes startup to fail. `make` loads `.env`
and preserves environment/command-line overrides; direct `cargo run` requires
exported environment variables.

For persistence, create **separate UTF8 development and test databases**, then
set their URLs. The database user needs DDL permissions for startup migrations;
the test user additionally needs `CREATE SCHEMA`. This repository's sibling
`compose-infra.yaml` already defines `offers_dev` and `offers_test` in
`offers-db`; use its configured application credentials and reachable address.
Alternatively use your own local PostgreSQL:

```sh
export DATABASE_URL='postgres://offers:replace_me@127.0.0.1:5432/offers_dev'
export DATABASE_URL_TEST='postgres://offers:replace_me@127.0.0.1:5432/offers_test'
PORT=3001 make serve
```

Startup applies embedded migrations before serving. Optional explicit
migration commands require Diesel CLI with PostgreSQL support:

```sh
make migrate
# Development reset only: deletes all offers and queue data!
make migrate-redo
```

Start with a fresh offers database. The migrations were rebuilt on 2026-10-19
and are a new baseline, not an upgrade path for databases with the previous
migration history. They create Mulac's queue tables, `event_store`,
`read_offer_identities`, and `read_offers`; the unused `offer_views` and
`offer_command_receipts` tables are omitted. There is no bodulv6 data migration.

## Flow and consistency

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant Queue as Mulac queues
    participant Handler as Import handler
    participant DB as PostgreSQL
    Client->>API: POST /api/offers/import
    API->>Queue: Persist command with command UUID
    API-->>Client: 202 command_id
    Client->>API: POST /api/commands/run
    Queue->>Handler: ImportOfferFromPage.v1
    Handler->>DB: Transaction: identity lock, history, command receipt
    Handler-->>Queue: Receipt's versioned events, after commit
    Client->>API: POST /api/events/run
    Queue->>DB: Upsert projection only for newer sequence
    Client->>API: GET /api/offers?retailer_code=...&url=...
    API->>DB: Read projection
    API-->>Client: Offer snapshot + ID + sequence
```

Nothing processes queues in the background. Queued imports become readable
after **both** processing steps. Queued commands execute in consumer execution order;
upstream observation timestamps do not establish precedence.

An offer's identity is the retailer code plus the **exact supplied URL**. Query
strings, fragments, case, explicit ports and trailing slashes are preserved;
different strings identify different offers. URLs must be absolute HTTP(S),
unpadded, free of control characters, and at most 2048 bytes. The limit keeps
the unique PostgreSQL identity index within its key-size bound.

Imports replace the complete page snapshot. Import requests contain only retailer
code, URL, title, price amount and currency; old price and discount are set to
null. Identical snapshots create no history event. Prices are nonnegative
`i64` minor units in the supplied currency. Supported currencies are defined by
`money::Currency`: USD, EUR, CAD, AUD, GBP, JPY, KRW, HKD, PLN, SEK, VND, NZD, INR.
Retailer codes use `bodul_shared::retailer::RetailerCode` slugs, such as
`ankerau`; unsupported codes are rejected. Titles must contain non-whitespace
text.

### Transactions and retries

`read_offer_identities` assigns the stable UUID and serializes imports for each
retailer/URL. A scoped `testimony_infra::ConnectionEventStore` participates in
the same Diesel transaction as `offer_command_receipts`. The handler replays
validated history, applies the import, and saves via `EventSourcedRepository`
with its no-op publisher. History and the receipt either both commit or both
roll back. No event publication occurs inside that transaction.

After commit, Mulac persists the receipt's events in `event_entries`. If
publication fails, retrying the **same command ID** returns the original
receipt's events without appending history again. Reusing that ID for different
input fails. A new REST request receives a new command ID; the API does not
provide a client-supplied idempotency key.

Mulac may publish duplicate envelopes with different envelope IDs. Events carry
full snapshots and offer sequence numbers; `offer_views` accepts only newer
sequences. Duplicate and out-of-order events cannot overwrite newer data.

`commands/run` and `events/run` consume only eligible work. Mulac uses backoff
and an attempt limit (six by default), so a retry need not be due immediately.
The report is `{ "processed": N, "errors": [] }` on full success; if any item
fails, `processed` is null and `errors` contains failure messages. Already
successful items in such a batch remain committed. Inspect the corresponding
queue's `status`, `attempts`, `scheduled_at` and `extra_info` for failures.
This service does not run Mulac's stale-reservation sweeper: a process killed
while holding reservations requires operator recovery before those rows can
run again. Graceful shutdown waits for active HTTP requests.

Stored event variants are explicitly named `RetailerOfferCreatedFromPage.v1`
and `RetailerOfferUpdatedFromPage.v1`; the command type is
`ImportOfferFromPage.v1`. Preserve these names and record compatibility when
changing the application.

## Routes

| Method/path | Response |
| --- | --- |
| `GET /api/health` | `200`, text `ok` (liveness). |
| `POST /api/tmp-import` | Validate an import and store it in the inbox; `202`, `{ "message_id": "<uuid>" }`. |
| `POST /api/offers/import` | `202`, `{ "command_id": "<uuid>" }`. |
| `POST /api/inbox` | Store one message; `202`, `{ "message_id": "<uuid>" }`. |
| `GET /api/offers/{id}` | Projected offer, or `404`. |
| `GET /api/offers?retailer_code=…&url=…` | Exact identity lookup; both parameters required. |
| `POST /api/commands/run` | Consume commands with `{ "limit": 1..1000 }`. |
| `POST /api/events/run` | Consume events with the same batch request. |
| `GET /api/openapi.json` | OpenAPI operations and schemas. |
| `/api/docs/` | Swagger UI. |
| `GET /graphql`, `POST /graphql` | GraphQL `{ health }`. |
| `GET /graphql/ide` | GraphiQL in debug builds only. |

Malformed/invalid input returns `400`; storage failures return `500`. Batch item failures are reported in the
`200` batch response. These development endpoints have no authentication;
control access when exposing the service beyond local development.

## Import walkthrough

With the database configured and the server running on port 3001:

```sh
curl -i http://localhost:3001/api/offers/import \
  -H 'content-type: application/json' \
  -d '{"retailer_code":"ankerau","url":"https://shop.example/charger?color=red","title":"USB charger","price_amount":999,"currency":"AUD"}'

curl http://localhost:3001/api/commands/run \
  -H 'content-type: application/json' -d '{"limit":100}'

curl http://localhost:3001/api/events/run \
  -H 'content-type: application/json' -d '{"limit":100}'

curl --get http://localhost:3001/api/offers \
  --data-urlencode 'retailer_code=ankerau' \
  --data-urlencode 'url=https://shop.example/charger?color=red'
```

The last response includes `id`, `sequence: 1`, and the imported fields. Use
that ID with `GET /api/offers/{id}`. Repeating the import with different fields
and running both queues updates the same offer and increments its sequence.
Repeating identical fields keeps its sequence unchanged.

## Direct testing import

`POST /api/tmp-import` validates the import JSON shown above and converts it
directly into an inbox message with the `ImportOfferDetails` payload and routing
key. It returns `202` with the inbox `message_id`; run the inbox, commands and
events queues before reading projections.

```sh
curl http://localhost:3001/api/tmp-import \
  -H 'content-type: application/json' \
  -d '{"retailer_code":"ankerau","url":"https://shop.example/charger","title":"USB charger","price_amount":999,"currency":"AUD"}'
```

Invalid input returns `400`, and inbox storage failures return `500`.

Open the [offers Bruno collection](../../bruno/offers/README.md), select its
`local` environment, and send **Import directly (testing)** followed by
**Run inbox**, **Run commands**, **Run events** and **Look up offer**. Set `offersBaseUrl` to your server address.

## Checks

```sh
make fmt
make ci
DATABASE_URL_TEST='postgres://offers:replace_me@127.0.0.1:5432/offers_test' make test-postgres
make -C .. audit           # when dependencies/lockfiles change
PORT=3001 make serve       # then curl http://localhost:3001/api/health
```

CI runs domain, record, configuration and HTTP smoke tests without PostgreSQL.
HTTP tests use concrete lazy pools; storage-failure tests point at an unavailable
socket, while persistence tests use the configured test database.
The opt-in PostgreSQL suite uses a fresh schema per case, then drops only that
schema. It covers migrations down/up, transaction rollback, shared-connection
Testimony behavior, concurrent imports, receipts, publication failure/retry,
projection ordering, and the complete HTTP flow. It never resets the selected
database's existing tables.

Catalog imports, sourcing integration, external event delivery, list/search,
and GraphQL offer mutations are outside this first implementation.
