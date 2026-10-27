# ADR 18: Customer facing persistent Mulac queues

- Status: Accepted
- Date: 2026-10-24

## Context

Customer facing has health and documentation routes but no durable acceptance
boundary for messages from offers. The local Mulac checkout provides shared
Poem contracts and explicit inbox, command, event, and outbox batch adapters.

## Decision

Require the service's own PostgreSQL database and apply embedded queue
migrations before serving HTTP. Keep REST and GraphQL health as liveness
responses. Assemble a persistent Mulac kernel with zero automatic drain rounds
and expose all four manual batch runners plus durable inbox submission.
Retain the kernel until graceful HTTP shutdown completes.

Use the sibling local Mulac checkout, shared with offers through workspace
dependencies. Retailer sourcing keeps its exact existing Git dependency until
its independent migration is implemented.

Do not start background workers or configure an outbound publisher. The outbox
runner returns 503 without reserving rows. No production handlers or subscribers
are registered in this bootstrap: unhandled commands fail visibly and follow
Mulac's retry rules. Future offer projection must account for the inbox
consumer's command-queue boundary.

## Consequences

Startup now fails when PostgreSQL is missing or unusable, while health after
startup remains independent of database availability. Queue contracts,
validation, blocking work, and retries stay in Mulac. There is no RabbitMQ
requirement and this decision does not change offers' delivery transport.
Developers need the sibling checkout; tests record its integration revision.
Tests use isolated PostgreSQL schemas and test-only dispatch handlers.
