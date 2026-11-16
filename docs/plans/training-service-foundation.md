# Training service foundation

Status: `[>]` planned; implementation has not started.

## Goal and boundaries

Set up `services/training` as an independently runnable service organized like
`services/offers`, using Poem, PostgreSQL, Mulac and Testimony. Include authenticated
reporting and integration with the development dashboard. Business features,
domain commands/events, inter-service subscriptions, outbox destinations and
deployment infrastructure are deferred. Do not copy offer-specific behavior or
introduce placeholder business entities.

## Implementation

1. Create the binary with `cargo new`, remove generated placeholder code, and
   register it in the services Cargo workspace and Makefile. Follow Offers'
   bootstrap, app/state, infrastructure, orchestration, monitoring and reporting
   boundaries. Future domain slices, projectors and subscribers follow Offers'
   organization when real features are introduced.
2. Reuse workspace dependencies and existing Mulac 0.5 Poem adapters. Declare
   `testimony` and `testimony_infra` under services workspace dependencies and
   migrate Offers to workspace inheritance. Coordinate this shared edit with the
   other foundation plan; do it once. Keep dependencies sorted.
3. Require a dedicated PostgreSQL database through `DATABASE_URL`; create a pool
   and run embedded migrations before serving. Missing, empty, invalid or
   unreachable database configuration fails startup with sanitized diagnostics.
   Provide fresh up/down migrations for the four Mulac queues, their current
   indexes, and Testimony's `event_store`, matching Offers' effective schema.
   Do not copy historical offer tables, migrations or data backfills.
4. Boot a persistent Mulac kernel on that pool with no business handlers or
   subscribers. Use explicit queue runners, with no background workers. Shut down
   the HTTP server and kernel gracefully on SIGINT/SIGTERM. Provide Testimony
   persistence infrastructure and verify it with test-only events rather than
   adding fake production domain types.
5. Build routes through `app()` so tests need no listening port. Support `PORT`
   with default 3000 and reject invalid values. Expose REST health, OpenAPI,
   Swagger UI, GraphQL health and debug-only GraphiQL, as Offers does.
6. Add inbox submission and four queue-run routes using Mulac adapters. Preserve
   Offers' request/response contracts, validation, retries and error IDs. Keep
   outbox delivery unconfigured: valid run requests return 503 without reserving
   rows. Unknown commands follow Mulac failure behavior; no dummy handlers.
7. Add Offers-compatible authenticated database-table and entry-status reporting,
   including current date-window statistics, response headers and error behavior.
   Use `TRAINING_REPORTING_TOKEN`, default `training-development-token`.
8. Extend development-dashboard server-side clients with `TRAINING_URL` (default
   `http://localhost:3007`) and `TRAINING_TOKEN` (default
   `training-development-token`). Add table and queue panels, totals, and a service
   SVG following existing assets. Preserve independent service failures and keep
   tokens out of browser output. Coordinate shared dashboard edits with the other
   plan; each service must remain independently testable.
9. Add README, `.env.example`, Diesel configuration, and Offers-style `serve`,
   migration and isolated PostgreSQL-test Make targets. Preserve shell/command-line
   environment precedence over `.env`. Document local launch on port 3007, a
   separate service database, queue processing and unconfigured outbox behavior.
10. Update root and dashboard documentation, portable/PostgreSQL CI matrices and
    live smoke coverage. Record the shared foundation architecture in a new ADR
    and list it in the ADR index; extend that new decision for the second service
    during the same implementation effort rather than duplicating it.

## Public interfaces

- `GET /api/health`, `/api/openapi.json`, `/api/docs`.
- `/graphql` with health query; `/graphql/ide` only in debug builds.
- `POST /api/inbox` and `/api/{inbox,commands,events,outbox}/run`.
- Authenticated `GET /api/reporting/database-tables` and
  `/api/reporting/entry-statuses`.

No business HTTP APIs or GraphQL mutations are introduced in this foundation.

## Validation and acceptance

- Test REST/GraphQL health, API documentation, release IDE exclusion, port parsing
  and startup configuration failures.
- Test inbox validation and duplicate-message handling, queue limits and empty
  batches, unknown-command failures, and unconfigured outbox behavior.
- Use isolated PostgreSQL schemas to verify migration up/down/reapply, durable
  queue persistence and Testimony append/load, metadata and sequence conflicts.
- Test reporting authentication/contracts, dashboard totals, unavailable-service
  handling, token isolation and icon routes.
- Run `make fmt` before `make ci` in affected workspaces. Run PostgreSQL tests and
  `make audit` because implementation changes dependencies/lockfiles; report new
  advisories. Never claim blocked checks passed.
- Start the service with `PORT=3007 make serve` against its database and check
  health. Start the development app and verify `/dashboard` shows its reports.
  Run both services together for final dashboard smoke coverage.

Move this task independently to `[~]` when implementation begins and `[x]` only
when implementation and validation are complete. These plans authorize no
production database resets or deployments.
