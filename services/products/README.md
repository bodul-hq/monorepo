# Products service

Offers-style service foundation using Poem, poem-openapi, async-graphql,
Mulac 0.5 and Testimony. Business commands, events and projections are not yet
registered. Each service owns a separate PostgreSQL database.

## Running

Provide PostgreSQL 16 or later and a database dedicated to this service:

```sh
make .env
# Edit .env and set DATABASE_URL to your database URL.
PORT=3006 make serve
curl http://localhost:3006/api/health
```

`DATABASE_URL` is required. Missing, empty, invalid or unreachable values fail
startup; embedded migrations run before serving. `PORT` defaults to 3000;
invalid values fail startup. Shell and Make command-line values override `.env`,
including explicitly empty values. `RUST_LOG` configures logging (default warn).
SIGINT/SIGTERM drain HTTP requests and shut down the Mulac kernel.

## Routes and processing

| Route | Behavior |
| --- | --- |
| `GET /api/health` | Liveness, returns `ok` |
| `/api/openapi.json`, `/api/docs` | OpenAPI and Swagger UI |
| `/graphql` | GET/POST GraphQL health query |
| `/graphql/ide` | GraphiQL in debug builds only; release returns 404 |
| `POST /api/inbox` | Durable message acceptance, returns 202 with message UUID |
| `POST /api/inbox/run` | Queue commands from inbox messages |
| `POST /api/commands/run` | Execute a command batch |
| `POST /api/events/run` | Dispatch an event batch |
| `POST /api/outbox/run` | Returns 503: no publisher is configured |
| `GET /api/reporting/database-tables` | Estimated row counts and storage |
| `GET /api/reporting/entry-statuses` | Exact queue counts scheduled today in UTC |

Queue APIs use Mulac's Offers-compatible contracts. Batch bodies are
`{"limit":100}`, with integer limits 1–1000. Invalid requests return 400;
consumer errors use Mulac's error IDs and retry behavior. Inbox requires payload,
routing_key and source, with optional message_id and correlation_id. Duplicate
message IDs preserve the first stored envelope. Queue APIs have no authentication,
as in Offers. No automatic queue workers run. Unknown commands fail during
command processing; accepting them into the inbox does not imply a handler exists.
Outbox runs never reserve rows while delivery is unconfigured.

Reporting requires `Authorization: Bearer <token>` and sends `Cache-Control:
no-store`. Set `PRODUCTS_REPORTING_TOKEN` (development default:
`products-development-token`). The dashboard uses `PRODUCTS_URL` (default
`http://localhost:3006`) and `PRODUCTS_TOKEN` with the same token.
Database and upstream diagnostics are not exposed in report errors.

## Structure and persistence

Bootstrap owns the pool and migrations; `app()` composes routes and adapters;
`orchestration` owns queue API declarations, `monitoring` health, and `reporting`
read-only statistics. `infra` exposes the PostgreSQL Testimony event store for
future domain slices. The initial migration contains only Mulac queues and their
indexes plus `event_store`. No offer-specific schemas or fake domain types exist.

## Validation

```sh
make fmt
make ci
DATABASE_URL_TEST=postgres://user:password@localhost/products_test make test-postgres
make audit # when Cargo dependencies or lockfiles change
```

PostgreSQL tests use unique disposable schemas or connection-local temporary
tables and require CREATE/DROP SCHEMA privileges. They cover migrations,
Testimony persistence and conflicts, inbox deduplication, kernel restart,
unknown commands, reporting windows and disabled delivery. Portable route tests
cover health, documentation, GraphQL, validation and error sanitization.
`make migrate` applies pending migrations; `make migrate-redo` destroys and
recreates service tables and is intended only for disposable development data.
