# Customer facing

Poem service with durable Mulac inbox acceptance and explicit queue batch APIs.
REST and GraphQL health report liveness. Offer projection and read APIs are
separate work: this bootstrap registers no production command handlers or event
subscribers.

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
| `POST /api/inbox` | Store a message and return `202`, `{"message_id":"<uuid>"}` |
| `POST /api/inbox/run` | Convert one batch of due inbox messages into queued commands |
| `POST /api/commands/run` | Execute one batch of due commands |
| `POST /api/events/run` | Dispatch one batch of due events |
| `POST /api/outbox/run` | `503`, `outbox publisher is not configured`; rows remain untouched |
| `GET /api/openapi.json` | OpenAPI description |
| `GET /api/docs/` | Swagger UI |
| `GET /graphql`, `POST /graphql` | GraphQL `{ health }` |
| `GET /graphql/ide` | GraphiQL in debug builds, `404` in release |

Submit the Mulac envelope, with a nonempty payload and nonblank routing key and
source:

```sh
curl http://localhost:3003/api/inbox \
  -H 'content-type: application/json' \
  -d '{"routing_key":"DevelopmentProbe","source":"manual","payload":{"value":"example"}}'
curl http://localhost:3003/api/inbox/run \
  -H 'content-type: application/json' -d '{"limit":1}'
curl http://localhost:3003/api/commands/run \
  -H 'content-type: application/json' -d '{"limit":1}'
```

The example deliberately uses an unregistered development command. Inbox
submission and processing succeed; command execution reports a missing handler
and schedules a retry. It does not create an offer. Incoming offers events will
need a command handler or an explicit event bridge: Mulac inbox processing
queues commands, rather than dispatching directly to event subscribers.

The optional `message_id` and `correlation_id` must be UUID strings. Omitting the
message ID generates a UUIDv7. Repeating an existing message ID succeeds and
preserves the original row, even if the new payload differs. Unknown routing
keys are accepted. An accepted message is durable but has not been processed.

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
inspect and recover stale reservations with Mulac's sweep tooling before they
can run again. Do not use migration resets as recovery. Ctrl+C/SIGTERM waits for
in-flight HTTP requests and then shuts down the kernel.

The [Bruno collection](../../bruno/customer_facing/README.md) provides the same
manual requests.

## Checks

```sh
make fmt
make ci
make test-postgres
make -C .. audit
PORT=3003 make serve
```

`make ci` runs configuration and HTTP tests without PostgreSQL. The opt-in
PostgreSQL suite requires `DATABASE_URL_TEST`; each case creates and drops only
its own generated schema. It verifies migration round trips and failures,
durable acceptance, duplicate IDs, metadata, separate queue stages, batch
limits, unknown-command retries, and unchanged unavailable-outbox rows.
Successful dispatch uses handlers/subscribers defined only in tests.

Run `cargo test --release graphiql_is_served_only_in_debug_builds` to check the
release GraphiQL route. After serving, check health, GraphQL health, OpenAPI and
each runner. Run the audit when dependencies or the lockfile change and report
new advisories. See [ADR 18](../../docs/adr/0018-customer-facing-mulac-queues.md)
for the runtime and startup decision.
