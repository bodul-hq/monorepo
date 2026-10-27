# ADR 17: Local Mulac adapters for retailer sourcing

- Status: Accepted
- Date: 2026-10-24

## Context

Retailer sourcing uses Mulac's Git tag `0.3.0`, while offers already uses the
sibling checkout and its extracted HTTP adapters. Retailer sourcing duplicates
batch validation, blocking-task execution, and result conversion. The next
pipeline phase needs the same local Mulac changes without changing its Axum
framework or feature-slice bus boundary.

## Decision

Use the sibling `mulac` checkout for the services workspace's kernel and share
the commanding/eventing API dependencies through workspace inheritance. This
integration was verified with Mulac revision
`9474319fff2237d81828a23d97df5d2f97741858`; path dependencies intentionally use
the checked-out files during this development phase rather than pinning a
remote release.

Keep retailer sourcing's handler registration, reactions, dispatcher, and
domain-facing bus traits. Wrap the persistent kernel's existing consumers in
`CommandingService`/`EventingService` and Mulac's Axum adapters. HTTP handlers
declare their routes and delegate; Mulac owns request validation, off-runtime
execution, and batch result conversion. Reuse Mulac's Utoipa schema wrappers.

Retain optional-database startup, manual queue progression, and zero inline
drain rounds. Start no worker or stale-reservation sweeper. Keep the kernel
handle until HTTP shutdown has drained active requests, then cancel it.

Adopt the adapter's plain-text `400` validation errors and sanitized `500`
task errors. Typed invalid request bodies that previously returned `422` now
return `400`. Preserve content-type/body-limit rejection, the successful JSON
batch shape, and the existing JSON `503` response for a valid request without
a database. Validate requests before checking that configuration. Consumer
failures remain `200` batch reports with null `processed` and per-item errors.

## Consequences

Build environments need the sibling Mulac checkout. The services graph no
longer resolves duplicate Git/path copies of the Mulac kernel and queue crates.
Clients that interpreted validation errors as JSON or depended on `422` must
use the documented response contract.

Stored command/event names, payloads, metadata, and queue schema remain intact;
no migration or queue reset is introduced. Isolated PostgreSQL tests consume
old-format rows, including null metadata and retry history, and verify separate
manual stages. HTTP tests cover boundary validation, failure behavior, and
shared schemas. Axum, feature-slice architecture, and outbox transport decisions
remain unchanged.
