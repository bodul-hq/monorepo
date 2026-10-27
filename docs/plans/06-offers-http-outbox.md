# Publish offers outbox messages to customer-facing over HTTP

Status: Completed. Covers TODO item 6. See the offers README's “HTTP delivery
to customer-facing” and [ADR 21](../adr/0021-offers-http-outbox-to-customer-facing.md).

## Goal and completion boundary

An explicit offers outbox batch delivers existing offer events to
customer-facing's durable HTTP inbox. An outbox row completes only after valid
acknowledgement of that same message identity.

Customer-facing's inbox and queue infrastructure are already implemented by
item 2. Item 6 is independently complete when transport works against that
boundary; item 7's offer persistence is not a prerequisite. HTTP acceptance
does not mean the offer projection has been updated.

HTTP is the active transport for this phase. Missing HTTP configuration leaves
publication unavailable. No selectable RabbitMQ mode or automatic transport
fallback is part of this round. Code organization and implementation mechanisms
are for the implementing tool to determine.

## Inspected baseline and evidence

Inspected monorepo revision: `5315da8f36e50be3d70d0da7f873f34262992a6c`.
Sibling Mulac revision: `9474319fff2237d81828a23d97df5d2f97741858`.
Cargo path dependencies use the actual sibling checkout, not a pinned revision.
Recheck relevant contracts if either checkout changes.

Existing-behavior references:

- [Offers README](../../services/offers/README.md),
  [outbox producer](../../services/offers/src/subscribers/customer_facing.rs),
  and [event types](../../services/offers/src/offers/event.rs).
- [Customer-facing README](../../services/customer_facing/README.md) and
  [queue contract](../../services/customer_facing/src/orchestration.rs).
- [ADR 16](../adr/0016-offers-rabbitmq-outbox.md),
  [ADR 18](../adr/0018-customer-facing-mulac-queues.md), and
  [ADR 20](../adr/0020-temporary-http-outbox-delivery.md).
- Local Mulac `libs/inbox_api/src/contract.rs` and `service.rs`,
  `libs/inbox/src/inbox_consumer.rs`, and `libs/outbox/src/outbox_consumer.rs`.

Offers already records raw offer-event JSON with destination
`customer_facing`, event type in metadata, and equal event/message UUIDs.
These are not Testimony's tagged event-store records. The current publisher
uses RabbitMQ. ADR 20 changes only the sourcing-to-offers hop and does not yet
authorize a changed offers-to-customer-facing implementation.

## Shared message contract: items 6 and 7

This section is identical in both handoffs. Both implementations must satisfy
it without depending on the other's internal design.

Send `POST /api/inbox` with `Content-Type: application/json`.

| HTTP field | Required value |
| --- | --- |
| `payload` | Existing event payload as a JSON object |
| `routing_key` | Exact supported event type below |
| `message_id` | Existing stable outbox message UUID |
| `correlation_id` | Existing optional correlation UUID; null or absent when unset |
| `source` | Existing nonblank source; `offers` when absent |

Only destination `customer_facing` and these event types are eligible:

| Event type, exactly as emitted | Payload fields |
| --- | --- |
| `OfferCreated` | `id`, `retailer_code`, `title`, `price`, `url`, `created_at` |
| `OfferTitleChanged` | `id`, `title` |
| `OfferPriceChangedEvent` | `id`, `price` |

The price event's `Event` suffix is part of the existing contract. Payload
`id` is the original offer UUID serialized as a string. `price` contains
an integer minor-unit `amount` and a `currency` string. Retailer codes use
enum serialization, for example `AnkerAu`. Preserve payload values; no
Testimony wrapper, extra event wrapper, or JSON-encoded payload string is used.

```json
{
  "routing_key": "OfferCreated",
  "message_id": "0191d6c4-8f5e-7a3b-9c1d-2e3f4a5b6c7d",
  "correlation_id": null,
  "source": "offers",
  "payload": {
    "id": "0191d6c4-8f5e-7a3b-9c1d-2e3f4a5b6c7e",
    "retailer_code": "AnkerAu",
    "title": "USB charger",
    "price": {"amount": 1999, "currency": "AUD"},
    "url": "https://shop.example/Charger?variant=123#details",
    "created_at": "2026-10-24T12:00:00Z"
  }
}
```

Title and price updates use this envelope with their own routing key and
payload fields. The message UUID identifies a delivery of an event;
`payload.id` identifies the offer. Never reuse a message UUID for different
content. An explicitly blank source or invalid payload is a failure.

The inbox has no dedicated original event-type or original causation fields.
The HTTP routing key carries the event type. Original causation remains in
sender metadata; adding unsupported top-level fields does not preserve it.

Current Mulac inbox processing converts routing key into command type,
preserves payload and correlation/source, generates a new command UUID, and
sets command causation to the original message UUID. Retrying inbox processing
can therefore create different command UUIDs for the same original message.
That distinction matters for replay safety.

The existing inbox accepts unknown routing keys durably. Its `202` only
acknowledges storage, not semantic validity or successful application. Duplicate
message IDs are accepted while retaining the first stored row, even when a
later payload differs.

## Configuration and transport requirements

The destination setting is `CUSTOMER_FACING_URL`, an HTTP(S) origin; delivery
uses its `/api/inbox` endpoint.

- An unset value disables publication. An explicitly empty, invalid, or
  non-Unicode value fails startup with a sanitized configuration error.
- Origins have a host, no credentials, no non-root path, no query or fragment,
  and no surrounding whitespace or control characters.
- Shell and Make command-line values override `.env`, including explicitly
  empty values.
- Downstream downtime does not prevent offers from starting.
- Without a configured database or HTTP destination, publication is unavailable.
- Legacy `AMQP_URL` and `OUTBOX_EXCHANGE` settings do not activate publication
  or fallback during this phase. Document their changed operational meaning.
- Exactly one transport is active; no dual publication or ambiguous-result
  fallback occurs.

Only `202` with valid JSON containing the same message UUID is acknowledgement.
Require a ten-second deadline covering the request and acknowledgement body,
and a maximum 64 KiB acknowledgement body. Redirects and automatic HTTP-client
retries are disabled. Errors do not disclose credentials or response bodies.

Preserve message identity, payload, and correlation through retries. Sender
completion depends on durable inbox acceptance, never receiver command/event
execution. No downstream runners, background workers, or recurring reservation
sweepers start automatically.

## Batch, recovery, and compatibility requirements

Retain the existing explicit `POST /api/outbox/run` contract and limits of
1 through 1000. Invalid requests fail validation before unconfigured-state
handling. Valid unconfigured requests return `503` without reserving rows.

Full success reports `{"processed":N,"errors":[]}`. Existing Mulac item/storage
failure behavior reports errors with null `processed`; successful items in a
partially failed batch remain committed. Preserve existing task-failure and
content-type behavior.

Publisher failures follow Mulac's existing retry scheduling and six-attempt
default. A lost acknowledgement or failed local completion can cause redelivery.
A failed completion can leave a stale reservation requiring operator recovery.
Do not claim exactly-once processing or immediate retries/dead-lettering.

Existing valid outbox rows remain deliverable without changing message IDs or
business payloads. Preserve queued work, statuses, attempts, and reservations.
Invalid legacy/manual rows fail visibly; they must not be silently discarded
or assigned different identities on retry.

## Acceptance scenarios

| Scenario | Required evidence |
| --- | --- |
| Each supported event | Exact routing key, raw payload object, stable message UUID |
| Metadata and source | Correlation preserved; absent source defaults; blank source fails |
| Existing queued rows | Compatibility without changed business content or identity |
| Unknown destination/type or malformed payload | Visible failure without false completion |
| Matching and duplicate `202` | Sender completion only for the expected UUID |
| Wrong/missing UUID, invalid/oversized JSON | Failed delivery |
| Other success status, redirect, `400`, `429`, `500` | Failed delivery under normal retry policy |
| Timeout or disconnect after acceptance | Retry retains identity and content |
| Acceptance followed by completion failure | No claim of exactly-once delivery; recovery demonstrated |
| Partial batch | Successful rows remain completed; failures remain recoverable |
| Missing configuration | `503` and untouched queue |
| Destination offline during startup | Service still starts |
| Legacy AMQP settings present | No broker publication or fallback |
| Error output | No credentials or response bodies exposed |

Use deterministic HTTP receiver tests and isolated PostgreSQL tests for queue
effects, including separate receiver storage. Verify a real existing
customer-facing inbox accepts the messages and retains one row on duplicate
submission. Item 7's projection is not required for this acceptance test.
Once both items exist, a combined projection walkthrough is additional evidence.

## Verification and operational handoff

Follow repository and offers-specific `AGENTS.md` instructions. Run
`make -C services fmt` before `make -C services ci`, and relevant
`make -C services/offers test-postgres` checks with a separate
`DATABASE_URL_TEST` and isolated schemas. Run relevant receiver integration
checks; audit with `make -C services audit` only if Cargo dependencies or
a lockfile change.

Start offers from its service directory with `PORT=3101 make serve` and
check `/api/health`. For a live acceptance walkthrough, run customer-facing
on another port with its own database. Preserve smoke tests and cover any
new routes. Report unavailable integration prerequisites accurately.

Document configuration, acknowledgements, queue-stage boundaries, retries,
retry exhaustion, and stale-reservation recovery without operational database
resets. A new ADR must supersede ADR 16's applicable offers-delivery decision;
accepted ADRs remain unchanged. Public configuration and transport behavior
must be reflected in service documentation and operator examples.
