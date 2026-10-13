# ADR 3: GraphQL over GET and debug-only GraphiQL

- Status: Accepted
- Date: 2026-10-12
- Supersedes: the `/graphql` route in [ADR 1](0001-web-frameworks-for-apps-and-services.md)

## Context

ADR 1 served GraphiQL on `GET /graphql` and GraphQL queries only on `POST
/graphql`. GraphQL clients and caches that send queries with `GET` received the
GraphiQL page instead of a result, and every build, including production, served
an interactive query editor to anyone who could reach the service.

## Decision

- `/graphql` executes GraphQL requests sent with `GET` (query string) or `POST` (body).
- GraphiQL is served at `/graphql/ide`, and only in debug builds (`cfg!(debug_assertions)`). Release builds don't register the route, so it returns 404.

## Consequences

- `GET` queries work with standard GraphQL clients and HTTP caches.
- Release builds expose no GraphiQL page. Introspection is still enabled, so tools can read the schema from a release build.
- async-graphql doesn't stop a `GET` request from running a mutation. No service has mutations yet. The first one must come with a check that rejects mutations sent with `GET`, as the GraphQL-over-HTTP specification requires, so cross-site links can't trigger them.
- To use GraphiQL against a service, run a debug build (`make serve`).
