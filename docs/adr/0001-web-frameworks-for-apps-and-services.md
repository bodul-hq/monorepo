# ADR 1: Web frameworks for apps and services

- Status: Accepted; the `/graphql` route is superseded by [ADR 3](0003-graphql-over-get-and-debug-only-graphiql.md)
- Date: 2026-10-12

## Context

The monorepo has two kinds of HTTP crates:

- **Services** (`customer_facing`, `offers`, `retailer_sourcing`) need to expose a REST API described by OpenAPI and a GraphQL API.
- **Apps** (`customers`, `development`) need to serve web pages to users.

## Decision

### Services

| Service             | HTTP framework | OpenAPI                                     | GraphQL                           |
| ------------------- | -------------- | ------------------------------------------- | --------------------------------- |
| `customer_facing`   | poem           | poem-openapi, with Swagger UI               | async-graphql, async-graphql-poem |
| `offers`            | poem           | poem-openapi, with Swagger UI               | async-graphql, async-graphql-poem |
| `retailer_sourcing` | axum           | utoipa, utoipa-axum, with utoipa-swagger-ui | async-graphql, async-graphql-axum |

Every service uses the same routes:

| Path                | Purpose                        |
| ------------------- | ------------------------------ |
| `/api/...`          | REST endpoints                 |
| `/api/openapi.json` | OpenAPI specification          |
| `/api/docs`         | Swagger UI                     |
| `/graphql`          | GraphQL (POST), GraphiQL (GET) |

### Apps

`customers` and `development` are poem servers that render HTML on the server.
No Rust is compiled to WebAssembly or runs in the browser.

### Common

- Every app and service runs on the [tokio](https://crates.io/crates/tokio) runtime.
- Every app and service reads its listen port from the `PORT` environment variable and defaults to 3000. Port 80 needs privileges on systems that restrict low ports, so a default of 80 would stop ordinary users from starting the servers.
- Every app and service binds to `0.0.0.0` so containers can expose it.

## Consequences

- Poem is used everywhere except `retailer_sourcing`, so code and patterns carry over between most crates. Developers who work on `retailer_sourcing` also need to know axum.
- All services use async-graphql, so GraphQL code looks the same in each and could be shared through `lib/shared`.
- poem-openapi generates the spec from the handlers themselves. utoipa needs a separate annotation on each handler (`#[utoipa::path]`), and that annotation can drift from the handler.
- Apps and services build with plain cargo. No wasm target or frontend build tool is needed.
- Interactive UI in the apps needs server round-trips or JavaScript. Choosing an HTML template engine is left for a later decision.
- Servers start locally without privileges. To run several at once, give each a distinct `PORT`. Deployments that need port 80 must set it explicitly or map an external port to 3000.
