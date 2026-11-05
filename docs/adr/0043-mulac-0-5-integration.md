# ADR 43: Mulac 0.5 integration

Status: Accepted; supersedes ADR 42's API-package aliases and feature selection.

Date: 2026-11-05

## Context

Mulac 0.5.0 folds the separate API and framework adapter packages into its
commanding, eventing, inbox and outbox crates. It also changes public imports,
repository construction, HTTP error reporting and exhausted-entry status.

## Decision

Resolve all Mulac packages at 0.5.0 through the existing null-labs-hq registry.
Use each queue crate's `prelude`, `poem` or `axum-openapi` feature, and
`api-testing` fixtures. Use `Async*Service` for HTTP execution and concrete
Diesel repositories for customer-facing's custom projection consumer.

Keep the existing service frameworks, routes and queue runners. Adopt Mulac's
UUID error IDs in batch responses and internal HTTP errors; log the underlying
diagnostics through tracing. Offers and customer-facing install a stderr
subscriber with a default `warn` filter, configurable through `RUST_LOG`.
Retailer sourcing retains its existing logging setup.

## Consequences

Clients must treat `errors` entries as diagnostic IDs and look up `error_id`
in service logs. Batch failures still return HTTP 200 with null `processed`;
successful items in a partial batch remain committed. Internal HTTP failures
return an error ID rather than a fixed message. Semantic validation messages
remain readable; malformed bodies receive IDs.

Exhausted entries remain failed (status 4) after six attempts and are ineligible
for automatic retry. Status 7 remains a legacy/manual dead state. Recovery is
still operator-owned. Existing queue tables and migrations are retained.

The services lockfile contains seven Mulac packages, all at 0.5.0, including
the new shared `mulac_http` package. No local path dependencies are introduced.
