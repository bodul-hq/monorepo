# ADR 16: Application-owned offers inbox and RabbitMQ outbox delivery

- Status: Accepted
- Date: 2026-10-21

## Context

Mulac 0.3.0 exposes kernel command and event consumers but not inbox or outbox
consumers. Offers already stores customer-facing events in its durable outbox.
The application needs explicit batch delivery without starting background workers.

## Decision

Own the Mulac inbox consumer and optional outbox consumer in `AppState`. Keep
command and event processing on the kernel. Inject `OutboxPublisherPort` into the
outbox consumer; configure the production RabbitMQ implementation with `AMQP_URL`
and optional `OUTBOX_EXCHANGE`. Missing configuration leaves outbox rows untouched
and returns 503. Inbox processing continues to read PostgreSQL, not RabbitMQ.

Use Lapin to publish persistent, mandatory messages with publisher confirms.
Only an ACK without a returned message succeeds. The bundled Mulac AMQP adapter
accepts ACKs containing returned messages, so use a local adapter that rejects
them. Preserve payload and message/event metadata. Reuse a lazy connection,
discard it on failure, and bound connection/publication work to ten seconds.
Provision durable queues and exchange bindings outside the service.

## Consequences

Broker outages do not prevent HTTP startup. Failed delivery follows Mulac retry
and reservation rules. A confirmation followed by a failed database write can
produce duplicates: receivers must deduplicate by message ID. This implements
at-least-once publication, not an atomic database/broker transaction. Unit tests,
isolated PostgreSQL tests and opt-in RabbitMQ tests cover each boundary.
