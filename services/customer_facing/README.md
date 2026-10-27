# Customer facing

Poem service with durable Mulac inbox acceptance and explicit queue batch APIs.
REST and GraphQL health report liveness. Incoming offers events maintain the
service-owned `offers` table during command processing. Offer read APIs are
separate work; no projection events or outgoing messages are generated.

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

Submit the agreed Mulac envelope directly; no upstream service is required:

```sh
curl http://localhost:3003/api/inbox \
  -H 'content-type: application/json' \
  -d '{"routing_key":"OfferCreated","message_id":"0191d6c4-8f5e-7a3b-9c1d-2e3f4a5b6c7d","correlation_id":null,"source":"offers","payload":{"id":"0191d6c4-8f5e-7a3b-9c1d-2e3f4a5b6c7e","retailer_code":"AnkerAu","title":"USB charger","price":{"amount":1999,"currency":"AUD"},"url":"https://shop.example/Charger?variant=123#details","created_at":"2026-10-24T12:00:00Z"}}'
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
| `OfferCreated` | `id`, `retailer_code`, `title`, `price`, `url`, `created_at` as above |
| `OfferTitleChanged` | `{"id":"<offer UUID>","title":"New title"}` |
| `OfferPriceChangedEvent` | `{"id":"<offer UUID>","price":{"amount":0,"currency":"EUR"}}` |

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

Run `cargo test --release graphiql_is_served_only_in_debug_builds` to check the
release GraphiQL route. After serving, check health, GraphQL health, OpenAPI and
each runner. Run the audit when dependencies or the lockfile change and report
new advisories. See [ADR 18](../../docs/adr/0018-customer-facing-mulac-queues.md)
for the runtime and startup decision.

See [ADR 22](../../docs/adr/0022-customer-facing-offer-projection.md) for the
projection transaction and metadata-preserving consumer decision.
