# Publish retailer sourcing outbox messages to offers

Status: Planned provisionally. Covers TODO item 5. Its selection as independent
is pending the user's definition of independence. The publisher can be built
and verified now using seeded outbox rows and the existing offers inbox; item 4
supplies real messages later and is not a code prerequisite.

Publish one requested batch from retailer sourcing's durable outbox to
`{offers}/api/inbox`. A row completes only after offers confirms durable
acceptance of the same message ID. Export conversion and the subscriber that
records export messages remain separate work.

## Existing boundaries

- [The infrastructure migration](../../services/retailer_sourcing/migrations/2026-10-13-000001_mulac_infrastructure/up.sql)
  already creates `outbox_entries`, including retry/reservation fields and
  `extra_info`. Retailer sourcing has no outbox HTTP runner or publisher yet.
- [The current state and bus](../../services/retailer_sourcing/src/platform/state.rs)
  support optional-database startup and manual command/event processing. Keep
  their existing behavior, Axum framework, and authentication boundaries.
- [Offers' inbox](../../services/offers/src/orchestration/inbox_api.rs) already
  accepts `POST /api/inbox`; it returns `202` after durable recording. Running
  its inbox, commands, and events remains a separate set of explicit actions.
- Local [OutboxService](../../../mulac/libs/outbox_api/src/service.rs) accepts
  a pool and an application-owned `OutboxPublisherPort` through `from_postgres`.
  [The Axum adapter](../../../mulac/libs/mulac_axum/src/outbox.rs) supplies
  validation, blocking execution, and an unconfigured `503` response.
- [The inbox contract](../../../mulac/libs/inbox_api/src/contract.rs) accepts
  payload, routing key, message ID, correlation ID, and source. It has no
  dedicated source event ID, event type, or causation ID fields.

## Delivery contract

Proposed handoff for the later export subscriber: one outbox row per
importable variation, with payload matching
[`ImportOfferDetailsCommand`](../../services/offers/src/offers/command.rs).
The publisher does not load export snapshots, split variation arrays, convert
prices, or interpret retailer documents.

| Proposed input from the outbox | HTTP inbox field |
| --- | --- |
| JSON import payload | `payload`, as a JSON object rather than a JSON-encoded string |
| `metadata.message_id` | `message_id`, unchanged across retries |
| Destination `routing_key = "offers"` | Command `routing_key = "ImportOfferDetails"` |
| `metadata.correlation_id` | `correlation_id`, unchanged |
| Nonblank metadata source | `source`, unchanged; use documented `retailer_sourcing` fallback when absent |

Confirm this handoff when item 4 is planned. The delivery `message_id` identifies
one importable entry; it is distinct from the source `event_id`, which may
describe an export containing several entries. The receiving inbox converts
its routing key into a command type. Sending `offers` as that HTTP routing key
would queue an unknown command. Reject other destinations before HTTP.

For example, the import payload is:

```json
{
  "retailer_code": "AnkerAu",
  "title": "USB charger",
  "price": { "amount": 1999, "currency": "AUD" },
  "url": "https://shop.example/products/charger?variant=123"
}
```

Preserve retailer enum serialization, exact URLs, and integer money values.
Validate JSON and the inbox envelope locally; prove the import payload shape
against offers' command deserializer in a contract test. The later subscriber
owns preparation of valid import fields. Retain source event ID/type/causation
in the outbox for diagnostics; do not claim the existing HTTP inbox preserves
them. Do not add ignored fields or wrap the command payload to carry them.

Require exactly `202`, a bounded valid JSON response, and a `message_id` that
parses to the UUID sent. A generic `2xx`, missing ID, mismatched ID, or invalid
response is not acknowledgement. Offers treats repeated IDs as accepted and
retains the original row, even if a subsequent payload differs. Therefore an
outbox message ID must never be reused for different content. This provides
at-least-once transport with inbox deduplication, not exactly-once business
processing or a guarantee that the offer projection is already updated.

## Implementation steps

1. **Add local outbox dependencies independently.** Declare shared `outbox`
   (`mulac-outbox`), `outbox_api`, and `inbox_api` paths once in
   `services/Cargo.toml` and inherit them in current consumers. Use local
   `mulac_axum` with `outbox` and `utoipa` features. Reuse retailer sourcing's
   existing reqwest client dependency. Keep AMQP features at the offers crate
   that currently needs them, rather than enabling them for this publisher.
   Regenerate the services lockfile and check the dependency graph.

   If [item 1](01-retailer-sourcing-local-mulac.md) has not landed, leave the
   existing Git kernel and command/event bus unchanged. This outbox uses local
   outbox types and the shared Diesel pool, not the Git kernel's outbox types.
   Coordinate the common manifest declarations when item 1 lands; do not make
   its command/event adapter migration part of this task.

2. **Configure one fixed destination.** Add an `OFFERS_URL` origin setting and
   construct its `/api/inbox` URL centrally. Accept HTTP(S) with a host; reject
   malformed/non-Unicode values, embedded credentials, query/fragment, and
   unexpected path prefixes. An absent value leaves delivery unconfigured;
   an explicitly invalid value fails startup with a sanitized error. Preserve
   `.env` overrides. Missing database configuration also leaves the outbox
   adapter unconfigured. Do not probe offers during startup.

3. **Implement the application publisher.** Add a focused platform module
   implementing the local `OutboxPublisherPort`. Use a reusable asynchronous
   reqwest client and the application runtime handle, following
   [the object-storage bridge](../../services/retailer_sourcing/src/platform/object_storage.rs).
   Its synchronous `publish` runs only on Mulac's blocking worker; it must not
   block a Tokio runtime task or create a second runtime. Disable redirects
   and automatic client retries, set a ten-second total request/response
   timeout, and bound the acknowledgement body to 64 KiB. Serialize the
   existing inbox request type and send `Content-Type: application/json`.

4. **Leave completion and retries with Mulac.** Return success only after the
   acknowledgement checks above. Map connection errors, timeouts, non-202
   responses, and invalid acknowledgements to sanitized publisher failures.
   Never delete or mark a row complete in the HTTP client. The current
   [consumer](../../../mulac/libs/outbox/src/outbox_consumer.rs) retries all
   publisher errors through its ordinary schedule and default six-attempt
   limit; do not promise immediate dead-lettering for HTTP `400` or conversion
   errors returned by this publisher. Preserve partial-batch successes. A lost
   acknowledgement or failed completion write can resend the same ID.

5. **Wire a manual runner.** Build
   `OutboxAdapter::new(OutboxService::from_postgres(pool, publisher))` when both
   persistence and the destination are configured; otherwise use
   `OutboxAdapter::unconfigured()`. Store it in `AppState`, expose
   `POST /api/outbox/run` through `app()`, and document it with Utoipa. Reuse
   `RunOutbox` and `OutboxRunResponse`, including `limit` from 1 through 1000.
   A valid unconfigured request returns `503` without reserving rows; invalid
   requests still return `400` first. Consumer/storage/publication failures
   return `200` with null `processed` and errors; task failure returns `500`.
   No worker, inbox runner, or downstream command runner starts automatically.

6. **Check existing row compatibility.** Verify queue columns and metadata
   against the local storage implementation. Each eligible row needs a
   concrete stable `message_id`; new subscriber rows will use their stable
   export-entry identity. If existing rows lack IDs, add a forward migration
   with a deterministic per-row mapping after inspecting their meaning; do
   not blindly reuse one source event ID across multiple variation rows.
   Preserve payloads, statuses, retries, and reservations. Leave old migration
   files unchanged. Document operator recovery for stale reservations without
   adding a recurring sweeper here.

7. **Record the temporary transport decision.** The TODO explicitly authorizes
   HTTP during this phase. Add a new ADR superseding the relevant RabbitMQ
   transport requirement in [ADR 16](../adr/0016-offers-rabbitmq-outbox.md),
   describe the source-to-offers hop and its acknowledgement/deduplication
   limits, and list it in the ADR index. Coordinate with the offers-to-customer
   publisher plan if it writes the same decision first. Keep the accepted ADR
   intact. Configure exactly one publisher for this outbox; never fan out to
   HTTP and AMQP or fall back to another transport after an ambiguous result.
   Update sourcing's README, `.env.example`, and Bruno outbox request.

## Verification and completion

Use fake HTTP endpoints to cover correct request conversion, matching `202`,
duplicate acknowledgement, malformed/oversized responses, wrong UUID, other
success statuses, redirects, `400`, `429`, `500`, connection failure, timeout,
and a connection dropped after recording. Check that retries keep the same
message ID and command content, and errors do not expose response bodies or
connection credentials. Extend route tests for validation, OpenAPI, unconfigured
state, bounded batches, and task failure.

Seed representative rows in the existing isolated-schema PostgreSQL harness;
no export listener is needed. Verify completion, retry counters/schedules,
partial batches, unavailable delivery leaving rows untouched, and a failed
completion write after successful acceptance. Test against a real existing
offers inbox backed by a separate test schema: resend the same message, observe
one unchanged inbox row, then run offers' inbox/commands/events separately to
prove the fixture imports. Never invoke downstream queues from the publisher.

During implementation, run `make -C services fmt`, `make -C services ci`,
`make -C services audit`, and both services' relevant `make test-postgres`
checks. Start sourcing with `PORT=3102 make serve` from its folder and the
documented database/object-storage configuration; check `/api/health` and the
new outbox route. Run offers on another port for the delivery walkthrough.

Completion means seeded import messages reach the existing offers inbox with
stable identities, only confirmed acceptance completes sourcing rows, and
failures follow Mulac's retry behavior. Real export traffic becomes available
when the separate export subscriber is implemented.
