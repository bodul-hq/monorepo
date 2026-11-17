# 79. Initial-fill startup and HTTP boundaries

Status: Accepted

## Context

Initial-fill preparation and command dispatch lived in `http.rs`, while shared
request and response bodies lived in `http/axum.rs`. A future Poem adapter needs
the same HTTP contract, and other callers need startup without an HTTP dependency.

## Decision

- `initial_offer_fill/start.rs` owns preparation, validation, run identities,
  starting commands, blocking queue execution, and application failures. Empty
  retailer lists are rejected during preparation for every caller.
- `initial_offer_fill/http.rs` owns serialized request/response bodies, OpenAPI
  schemas, and the mapping from application results to HTTP outcomes. It has no
  dependency on Axum types.
- `initial_offer_fill/http/axum.rs` owns extraction, route registration, endpoint
  documentation, and rendering those outcomes as Axum responses. The feature's
  prelude re-exports its routes.

The whole batch is prepared before dispatch. Dispatch remains sequential and
stops on failure, preserving confirmed prior runs and existing error bodies.

## Consequences

Preparation and queueing tests live with startup; serialization tests live with
the shared HTTP contract. A future HTTP framework adapter can reuse that
contract and mapping. Startup stays in one module until its responsibilities
justify further splitting. Existing feature-slice boundaries from ADR 4 remain
in effect.
