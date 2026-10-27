# Upgrade retailer sourcing to local Mulac

Status: Planned. Covers TODO item 1. No other unchecked TODO item is a
prerequisite.

Use the sibling Mulac checkout for `retailer_sourcing` and delegate its command
and event HTTP runners to Mulac's Axum adapters, following the assembly pattern
already used by `offers`. The existing sitemap and offer pipeline must continue
processing persisted work through explicit queue runs.

## Current implementation

- [The services workspace](../../services/Cargo.toml) declares `kernel` from
  Mulac's Git tag `0.3.0`; retailer sourcing inherits it. [Offers](../../services/offers/Cargo.toml)
  instead selects the sibling checkout through local paths.
- [The bus](../../services/retailer_sourcing/src/platform/bus.rs) owns handler
  registration, event-to-command reactions, command dispatch, consumer calls,
  and batch result conversion. `start_persistent(pool, 0)` leaves processing
  manual. Feature slices depend on the bus's traits rather than Mulac types.
- [The queue routes](../../services/retailer_sourcing/src/platform/bus_api.rs)
  define their own request/response types, validate the batch limit, and run
  blocking consumers through `spawn_blocking`.
- [Startup](../../services/retailer_sourcing/src/main.rs) permits an absent or
  empty `DATABASE_URL`. In that mode health and documentation work; queue runs
  return `503`. Configured pipeline startup also needs object storage.
- [The original queue migration](../../services/retailer_sourcing/migrations/2026-10-13-000001_mulac_infrastructure/up.sql)
  already creates inbox, command, event, and outbox tables with `extra_info`.
  Their presence does not mean inbox or outbox HTTP processing is wired.
- Local [mulac_axum](../../../mulac/libs/mulac_axum/Cargo.toml) already provides
  `commanding`, `eventing`, and `utoipa` features. Its adapter services accept
  the kernel's existing consumers; no new Mulac feature is needed for this item.

## Scope and compatibility

Keep Axum, feature-slice registration, stored command/event names, payload
formats, correlation/causation propagation, and manual queue progression.
Preserve all existing routes, authentication boundaries, and optional-database
startup. Retain `drain_rounds = 0`; do not copy offers' value of `1` into this
service or start queue workers.

Adopt Mulac's shared validation and error responses for the two queue routes.
This deliberately changes some error contracts:

| Case | Current retailer sourcing | Planned behavior |
| --- | --- | --- |
| Successful run | `200`, `{ "processed": N, "errors": [] }` | Same |
| One or more consumer failures | `200`, null `processed`, error messages | Same; successful items remain completed |
| Limit `0` or above `1000` | `400`, JSON `{ "error": ... }` | `400`, Mulac plain-text validation error |
| Missing, negative, fractional, null, or wrongly typed limit | Axum typed-JSON rejection, normally `422` | `400`, Mulac validation error |
| Malformed JSON | `400` | `400` |
| Missing or non-JSON content type | `415` | `415` |
| Oversized request | Axum body-limit rejection | Preserve body-limit rejection, `413` |
| Blocking task failure | `500`, JSON containing join error | `500`, sanitized Mulac text `queue processing task failed` |
| Valid request without a database | `503`, existing JSON error | Same application-owned `503` response |

Validate requests before checking whether the bus is configured. Unknown fields
remain accepted. Document these changes in the service README and ensure Bruno
requests do not rely on the old validation-error body shape.

Export processing, outbox recording/publication, new inbox routes, and customer
projections belong to separate TODO items. Adding the local dependency does not
enable those features automatically.

## Implementation steps

1. **Unify dependency sources.** Change the services workspace's `kernel` to
   `{ package = "mulac-kernel", path = "../../mulac/kernel" }`. Move offers'
   duplicate local declaration to workspace inheritance. Declare
   `commanding_api` and `eventing_api` once under workspace dependencies with
   their `../../mulac/libs/...` paths, and inherit them in both services.
   Add `mulac_axum` to retailer sourcing with local path
   `../../../mulac/libs/mulac_axum` and features `commanding`, `eventing`,
   `utoipa`. Keep dependency tables alphabetical and enable test-only features
   through dev-dependencies. Inspect the Cargo graph for duplicate Git/path
   Mulac packages and update `services/Cargo.lock` through Cargo.

   Coordinate the shared manifest edits with the customer-facing setup plan.
   Either item can be implemented first. If customer-facing setup has already
   moved the workspace declaration to local Mulac while preserving sourcing's
   old Git dependency in its crate manifest, switch that last declaration back
   to workspace inheritance here. Do not make the customer-facing
   implementation a prerequisite.

2. **Keep application bus behavior and reuse the assembled consumers.** Build
   `CommandingService::new(handle.command_consumer())` and
   `EventingService::new(handle.event_consumer())`, then wrap them in
   `CommandingAdapter` and `EventingAdapter`. Follow
   [offers' startup](../../services/offers/src/main.rs), using the Axum adapters.
   Keep `Bus`, `Command`, `Event`, `Handle`, `Published`, `Reaction`, and
   `Dispatcher` as the feature boundary. Preserve timing logs and the existing
   mapping of handler failures into retryable Mulac errors.

3. **Represent the optional runtime in state.** Extend
   [AppState](../../services/retailer_sourcing/src/platform/state.rs) or its bus
   handle with the adapters, assembled only when the database is configured.
   Extract optional state without rejecting before request validation. Retain
   the persistent kernel handle for explicit shutdown after HTTP requests
   finish, following offers' lifecycle. No background poller or reservation
   sweep is introduced by this change.

4. **Reduce queue handlers to validation extraction and delegation.** Use
   `RunCommands`/`CommandsRunResponse` and `RunEvents`/`EventsRunResponse` from
   `mulac_axum::io`; keep route declarations in `platform/bus_api.rs`. Retain
   only the application-owned unavailable-bus check. Remove duplicate batch
   conversion and blocking-task code when no callers need it. If internal
   callers still need synchronous runs, delegate them to the shared services
   instead of retaining a second validation/result implementation.

5. **Publish the actual API contract.** Use the adapters' Utoipa support, as in
   [Mulac's command example](../../../mulac/libs/mulac_axum/examples/axum_commanding_openapi.rs).
   Both endpoints should reference the same `RunRequest` and `RunResponse`
   components, with limits `1..=1000`. Document `400`, `413`, `415`, `500`, and
   the application-specific `503`, including the distinction between plain
   text adapter errors and the existing JSON unavailable-bus error.

6. **Verify persisted queue compatibility before adding migrations.** Compare
   local Mulac's Diesel queue schemas and metadata deserializers with the
   existing infrastructure migration. Seed old-format command/event rows,
   including null metadata and populated `extra_info`, then consume them using
   the new runtime. Preserve row IDs, command/event names, statuses, attempt
   counts, timestamps, and metadata. Add a forward migration only for a
   demonstrated incompatibility; never recreate queue tables or rewrite the
   accepted migration baseline to make an upgrade pass.

7. **Document the development requirement.** Update root and service README
   setup instructions to require the sibling `mulac` checkout at the same
   directory level as `monorepo`; record the tested Mulac revision for
   reproducibility. Explain the queue error-contract changes and manual runs.
   Record the shared-adapter ownership and temporary local-source arrangement
   in a new ADR and list it in the ADR index. Leave the accepted feature-slice
   and RabbitMQ ADRs intact; this item changes no delivery transport.

## Verification and acceptance

Extend HTTP tests through `app()` for both queue routes, covering bounds `1`
and `1000`, all invalid-limit shapes, malformed JSON, content type, body limit,
unknown fields, unavailable bus, empty queue, partial failure, and sanitized
task failure. Use Mulac's `testing` fixtures where they exercise these boundary
behaviors. Assert OpenAPI schemas and the service-specific `503` contract.

Use [the existing isolated-schema harness](../../services/retailer_sourcing/src/platform/test_db.rs)
to exercise a deterministic command → event → next command sequence through
the actual HTTP endpoints. Assert each call advances only its requested queue,
batch limits are respected, reactions preserve metadata, and failed rows are
unreserved and scheduled for retry. Include a pre-upgrade queue-row fixture so
a successful fresh installation cannot hide a compatibility regression. Keep
the existing feature-slice PostgreSQL tests passing.

Run during implementation:

```sh
make -C services fmt
make -C services ci
make -C services audit
make -C services/retailer_sourcing test-postgres
DATABASE_URL= PORT=3102 make -C services/retailer_sourcing serve
```

The audit is required because this implementation changes Cargo dependencies
and the services lockfile. Supply a separate `DATABASE_URL_TEST` for PostgreSQL
tests. With the last command running, check `/api/health`, `/api/openapi.json`,
and a valid queue request returning the documented `503`. Then run with the
configured development database and object storage and verify an empty or
controlled command/event batch plus health. Do not use destructive database
reset targets for verification.

The item is complete when both services resolve the intended local Mulac source,
retailer sourcing delegates queue execution to its Axum adapters, stored work
survives the upgrade, and the pipeline still advances only on explicit calls.
