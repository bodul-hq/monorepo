# ADR 21: Temporary HTTP outbox delivery from offers to customer-facing

- Status: Accepted
- Date: 2026-10-24
- Supersedes: ADR 16's RabbitMQ delivery decision for the offers to
  customer-facing hop during this phase. ADR 16's application-owned inbox and
  explicit batch execution remain in force.

## Context

The TODO calls for temporary service-to-service HTTP delivery while the
end-to-end offer pipeline is assembled. ADR 20 made that change for the
retailer-sourcing to offers hop only. Offers already records each
`OfferCreated`, `OfferTitleChanged` and `OfferPriceChangedEvent` event in its
durable outbox with destination `customer_facing`. Customer-facing already
accepts messages durably at `/api/inbox` (ADR 18).

## Decision

Offers owns an `OutboxPublisherPort` that posts to a configured
`CUSTOMER_FACING_URL` origin plus `/api/inbox`. It is the only active
transport. `AMQP_URL` and `OUTBOX_EXCHANGE` are no longer read and never act
as a fallback. Mulac's outbox service and Poem adapter keep owning
reservation, explicit `POST /api/outbox/run` batches, completion and retries.

Only rows with destination `customer_facing` and one of the three event types
are sent. The HTTP routing key is the event type exactly as emitted, including
the `Event` suffix of `OfferPriceChangedEvent`. The payload is the recorded
event JSON object, unchanged and unwrapped. The message ID and correlation ID
are preserved. The source is preserved and defaults to `offers` when absent; a
blank source fails. The original event ID, type and causation stay in offers'
outbox metadata, because the inbox contract has no fields for them.

Acceptance and transport rules match ADR 20. Only `202` with a valid matching
message UUID confirms acceptance. One ten-second deadline covers the request
and acknowledgement. Acknowledgements are limited to 64 KiB. Redirects and
client retries are disabled, and errors are sanitized. Both services use the
shared `lib/inbox_client` crate for this transport. Each service keeps its own
routing policy and error messages.

## Consequences

Delivery is at least once. A lost acknowledgement or a failed completion write
can resend a message. Customer-facing keeps the first inbox row for a repeated
message ID, so this is not exactly-once processing. A `202` means durable
storage only, not that customer-facing processed or projected the offer.

Publisher failures follow Mulac's retry schedule and six-attempt limit. This
includes rejected requests and invalid or unsupported rows. A failed
completion can leave a stale reservation that needs explicit operator
recovery; no sweeper or worker is started. Invalid configuration fails
startup, but an unreachable customer-facing service does not.

Offers no longer contains RabbitMQ publisher code or the `test-rabbitmq`
target. Returning to a broker requires a new decision. That decision must
keep stable message IDs and switch transports explicitly, never publishing to
both transports.
