# Persist customer-facing offers from incoming events

Status: Implemented and verified on 2026-10-24. Covers TODO item 7.

Implementation: [projection](../../services/customer_facing/src/offer_projection.rs),
[PostgreSQL acceptance tests](../../services/customer_facing/src/postgres_tests/projection.rs),
[manual operation and recovery](../../services/customer_facing/README.md), and
[ADR 22](../adr/0022-customer-facing-offer-projection.md).

Verification: services formatting and CI passed; all 12 customer-facing
PostgreSQL tests passed using isolated schemas in `customer_facing_test`.
Startup with `PORT=3103 make serve` against `customer_facing_dev` and live REST
and GraphQL health, OpenAPI, Swagger UI, queue validation, and unconfigured
outbox checks passed. Mulac remains at the inspected revision below.

Dependency audit ran and reported eight existing vulnerabilities: rustls
0.23.45 (RUSTSEC-2026-0346, -0347, -0348, -0349), rustls-webpki 0.103.15
(RUSTSEC-2026-0382), and zip 8.6.0 (RUSTSEC-2026-0367, -0368, -0369).
These versions already occur in repository HEAD; this implementation introduces
no new advisory. Existing warnings also cover fxhash, kuchiki, unsafe-libyaml,
rand, tokio, and yanked num-modular. Dependency remediation is separate work.

## Goal and completion boundary

Customer-facing durably maintains its own `offers` table from incoming offer
events, with the same business properties as offers' `read_offers` model.

Item 2 already provides the service's database and queue infrastructure.
Submit the agreed envelopes directly to verify this task independently of item
6's publisher. Successful completion includes persistence and safe replay,
not merely accepting messages or defining an entity.

Item 8 will add paginated REST and GraphQL retrieval in customer-facing.
Item 9 will consume that API from the customers homepage. Neither is included
here. Historical backfill and cross-service database reads are also excluded.
Implementation design and code organization belong to the implementing tool.

## Inspected baseline and evidence

Inspected monorepo revision: `5315da8f36e50be3d70d0da7f873f34262992a6c`.
Sibling Mulac revision: `9474319fff2237d81828a23d97df5d2f97741858`.
Cargo path dependencies use the actual sibling checkout, not a pinned revision.
Recheck relevant contracts if either checkout changes.

Existing-behavior references:

- [Customer-facing README](../../services/customer_facing/README.md),
  [queue contract](../../services/customer_facing/src/orchestration.rs), and
  [ADR 18](../adr/0018-customer-facing-mulac-queues.md).
- [Offers events](../../services/offers/src/offers/event.rs),
  [read entity](../../services/offers/src/read_offer/entity.rs),
  [read repository contract](../../services/offers/src/read_offer/repository.rs),
  and [existing read-model schema](../../services/offers/migrations/2026-10-19-000002_offers/up.sql).
- [Offers outbox producer](../../services/offers/src/subscribers/customer_facing.rs)
  and [item 6 handoff](06-offers-http-outbox.md).
- Local Mulac `libs/inbox_api/src/contract.rs` and `service.rs`,
  and `libs/inbox/src/inbox_consumer.rs`.

Customer-facing requires its own PostgreSQL database and applies migrations
before serving. It already has durable inbox submission and manual inbox,
command, event, and outbox runners. No production offer handlers or subscribers
are registered at this baseline. The outgoing publisher is unconfigured.

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

## Business and persisted data contract

| Business property | Required representation and meaning |
| --- | --- |
| `id` | Original offer UUID; do not allocate a new customer-facing identity |
| `retailer_code` | Original retailer identity |
| `title` | Original text, including Unicode and meaningful whitespace |
| `price` | Exact signed integer minor-unit amount and currency |
| `url` | Original URL without normalization |
| `created_at` | Original creation instant, not receipt or processing time |

The existing `read_offers` record stores `id` as UUID, `retailer_code`,
`url`, and `title` as text, `price_amount` as BIGINT, `currency` as text,
and `created_at` as TIMESTAMPTZ. All are required. It has a primary key on
offer ID and a unique retailer/exact-URL pair. URL comparison is case-sensitive
(`C` collation), and URL length is limited to 2048 bytes. Existing repository
validation also rejects NUL in title and URL.

Wire retailer values use enum serialization, such as `AnkerAu`; persistence
uses the corresponding retailer code, such as `ankerau`. They express the
same identity. `Money` stores a signed `i64` amount and a supported currency.
Preserve exact minor units, including zero, and currency changes; do not
introduce floating-point conversion or independently redefine money validity.
Preserve creation instants within PostgreSQL's timestamp precision.

Customer-facing's table is named `offers` and has these business properties
and identity constraints. Internal replay bookkeeping is an implementation
choice; no new business properties, synthesized timestamps, URL normalization,
or alternate business IDs are requested.

## Required application behavior

- `OfferCreated` establishes the complete offer.
- `OfferTitleChanged` changes only the title.
- `OfferPriceChangedEvent` changes price amount and currency together.
- Changes preserve unrelated properties and original creation time.
- HTTP acceptance is durable receipt only. Explicit queue processing is
  required; running the documented inbox/command/event sequence must persist
  supported messages. No automatic draining is introduced.
- Applying the same original message again succeeds without changing already
  applied business state, including after newer updates. This applies when
  inbox replay generates different command UUIDs.
- Concurrent processing of one original message has one logical effect.
  Persistence and replay safety survive service restart.
- Failed persistence is not successful application. Retry must remain possible;
  replay tracking must never suppress an event whose business change failed.
- An update received before creation fails visibly and follows existing
  retries. It does not create an incomplete offer. It can succeed on a later
  eligible retry after creation; retry exhaustion is still possible.
- A creation under a different message identity conflicting with an established
  offer or retailer/URL identity fails visibly and does not overwrite it.
- Malformed payloads, invalid identities, and unsupported types do not silently
  succeed or partially modify an offer. The generic inbox may already have
  acknowledged storage; application errors are observable during processing.
- Distinct updates apply in processing order. The contract carries no offer
  revision, so this phase does not guarantee source ordering or that a delayed
  distinct update cannot replace a newer source value. A source-order or
  stale-update protection protocol is outside scope.
- Recognizing an already applied message must not reset the offer to its
  creation values or reapply an earlier title/price.

Keep existing queue data through schema changes. Database startup requirements,
liveness endpoints, queue validation and retry behavior, and the unconfigured
outgoing publisher remain part of the service contract. There is no dependency
on reading another service's database or executing another service's queues.

## Acceptance scenarios

| Scenario | Required evidence |
| --- | --- |
| Creation | All business properties durably preserved |
| Title update | Only title changes |
| Price/currency update | Both change together; unrelated fields preserved |
| Value fidelity | Unicode, whitespace, exact URL, retailer identity, UUID, original instant, and integer precision |
| HTTP submission only | Durable receipt without claiming applied persistence |
| Inbox runner only | Commands queued; no claim that application has completed |
| Full documented queue sequence | Expected final `offers` record |
| Duplicate HTTP message | One inbox identity and one logical business effect |
| Different command UUIDs, same original message | Replay does not duplicate or reset business state |
| Replay after later updates | Older already-applied create/title/price does not regress state |
| Concurrent duplicate | One logical application |
| Persistence failure and retry | No false success or permanently suppressed update |
| Restart | Offer state and replay protection survive |
| Update before create | Visible failure; eligible retry succeeds after creation |
| Distinct updates in reverse source order | Processing-order behavior, with limitation documented |
| Conflicting creation/retailer-URL identity | Existing record preserved; visible error |
| Malformed or unsupported message | No partial change or silent success |
| Existing database with queue data | Additive migration preserves prior queued work |

Use direct submissions to the real customer-facing inbox with isolated
PostgreSQL schemas; no item 6 implementation is necessary. Cover receipt,
queue progression, application, and replay as distinct observable boundaries.
Once item 6 exists, its delivery can be included in a combined walkthrough,
but it does not replace independent acceptance evidence.

## Verification and operational handoff

Follow repository `AGENTS.md` requirements. Run `make -C services fmt`
before `make -C services ci` and
`make -C services/customer_facing test-postgres` with a separate
`DATABASE_URL_TEST` and isolated schemas. Run `make -C services audit` only
if Cargo dependencies or a lockfile change.

Start customer-facing from its service directory with `PORT=3103 make serve`
and its own configured database; check `/api/health` and existing health,
documentation, and queue smoke tests. Extend smoke coverage for any new routes.
Report unavailable prerequisites rather than claiming integration checks passed.

Document the exact manual runner sequence that makes offers visible in the
database, how processing failures are observed, replay guarantees, and the
processing-order limitation. Include retry exhaustion and stale-reservation
recovery without operational database resets. Keep the service's startup and
configuration guidance accurate. Record architecture decisions in new ADRs
when required; do not rewrite accepted ADRs.
