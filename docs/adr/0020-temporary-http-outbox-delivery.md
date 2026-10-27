# ADR 20: Temporary HTTP outbox delivery to offers

- Status: Accepted
- Date: 2026-10-24
- Supersedes: ADR 16's broker-only delivery direction for the retailer-sourcing to offers hop during this phase. Offers' existing RabbitMQ publisher remains unchanged.

## Context

The TODO explicitly calls for temporary service-to-service HTTP delivery while
the end-to-end offer pipeline is assembled. Retailer sourcing already has durable
outbox tables, and offers already has a durable `/api/inbox`. Recording export
messages is separate from delivering an existing row.

## Decision

Retailer sourcing owns an `OutboxPublisherPort` that posts to a configured
`OFFERS_URL` origin plus `/api/inbox`. Mulac's local outbox service and Axum
adapter own reservation, explicit batch execution, completion and retries. This
does not require migrating sourcing's command/event kernel from its current Git
version. Use the sibling Mulac checkout tested at
`9474319fff2237d81828a23d97df5d2f97741858`.

Each outbox row carries one JSON import payload and a stable `message_id`.
Destination routing key `offers` maps to inbox command key `ImportOfferDetails`.
Preserve payload, message ID, correlation ID and source; default an absent source
to `retailer_sourcing`. Source event ID/type/causation stay in local outbox
metadata because the current HTTP inbox cannot represent them. The export
subscriber must use a distinct stable delivery ID for each importable variation,
without reusing that ID for changed content.

Only `202` with a valid matching message UUID confirms acceptance. The entire
request and acknowledgement have a ten-second deadline; acknowledgement bodies
are limited to 64 KiB. Disable redirects and client retries. Missing destination
or database makes `/api/outbox/run` unavailable without reserving rows. Invalid
destination configuration fails startup; downstream downtime does not.

## Consequences

Lost acknowledgement or a failed local completion write can resend a message.
Offers retains the first inbox row for a repeated message ID; that provides
at-least-once delivery with inbox deduplication, not exactly-once business
processing. The sender never advances the receiving service's queues.

Publisher failures, including rejected requests, follow the existing Mulac
retry schedule and six-attempt limit. A completion failure can leave a stale
reservation requiring explicit operator recovery; no background sweeper is
introduced. Error messages omit credentials and response bodies.

Use exactly one transport for this outbox. A future broker migration must
retain stable message identities and change the configured publisher explicitly;
never publish to two transports or fall back after an ambiguous acknowledgement.
