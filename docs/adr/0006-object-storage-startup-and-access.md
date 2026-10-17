# ADR 6: Object storage startup and access

- Status: Accepted
- Date: 2026-10-15
- Supersedes: the GET `/graphql` requirement in [ADR 3](0003-graphql-over-get-and-debug-only-graphiql.md), for `object_storage` only

## Context

The object storage specification defines creation mutations and a POST-only
GraphQL transport. Both APIs depend on a separate PostgreSQL database and must
share authentication and immutable storage semantics. The existing local
infrastructure calls that database `object_storing`, while the service crate is
named `object_storage`.

## Decision

- Implement `services/object_storage` using axum, as accepted in ADR 5, with
  async-graphql and Diesel's pooled PostgreSQL backend.
- Serve GraphQL only through POST `/graphql` with an `application/json` body.
  Validate the document and all variables before any resolver executes, and
  distinguish request failures from execution failures in the HTTP adapter.
- Require `DATABASE_URL`, successful database connectivity, UTF8 encoding, and
  successful embedded migrations before binding the HTTP listener. This service
  has no database-free runtime mode.
- Require one service-specific hard-coded development bearer token on every
  route, including health, documentation and its assets, GraphiQL, and fallbacks.
  Check credentials before parsing bodies or processing GraphQL operations.
- Keep GraphiQL at `/graphql/ide` in debug builds only, as required by ADR 3.
- Keep the existing `object_storing` local database infrastructure and parent
  Compose files unchanged. The service uses a separate database from sourcing.

## Consequences

- GET `/graphql` returns 405 after authentication. Mutations cannot run via GET.
- Startup fails rather than exposing a service that cannot store artifacts.
  `/api/health` is authenticated liveness; storage failures use API errors.
- Browser documentation requires an Authorization header on navigation and
  asset requests, not just on API calls from the page.
- Repository readers know the development token. Production authentication
  will need a separate decision.
- Other services retain ADR 3's GET and POST GraphQL behavior. Retention,
  deduplication, and retailer sourcing integration remain separate work.
