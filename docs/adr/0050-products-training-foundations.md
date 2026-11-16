# 50. Products and Training service foundations

Status: Accepted

## Context

Products and Training need independently runnable foundations matching Offers,
before their domain behavior is defined.

## Decision

Use separate Poem services with poem-openapi and async-graphql, each owning its
PostgreSQL database. Follow Offers' bootstrap, route composition, infrastructure,
orchestration, monitoring and reporting boundaries. Reuse published Mulac 0.5
adapters and local Testimony libraries through workspace dependencies.

Create fresh queue and event-store schemas. Boot a persistent Mulac kernel without
business handlers/subscribers or automatic workers. Keep outbox delivery
unconfigured until a destination is specified. Introduce no placeholder domain
entities or cross-service message contracts. Test-only events verify Testimony.

Expose Offers-compatible reporting with service-specific bearer tokens, and extend
the development dashboard using server-side authenticated clients. Keep the
existing explicit unauthenticated operational queue API convention.

## Consequences

Each service can be started, tested and evolved independently. Business slices,
versioned event records, projections and delivery configuration arrive with future
features. Reusing the established adapters keeps validation and queue behavior
consistent without creating a new shared service framework. Service-owned reporting
and bootstrap code follow Offers and require coordinated maintenance when their
contracts change. Existing service databases are not migrated or reset.
