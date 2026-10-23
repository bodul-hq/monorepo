# Build the offers application around Testimony and Mulac

## Summary

Implement a working page-import flow with the same separation of
responsibilities as `bodulv6`: domain logic, application use cases,
infrastructure adapters, and bootstrap wiring.

Keep these as modules within the existing offers crate. Testimony manages
aggregate history; Mulac queues commands and events. REST provides import and
lookup operations. Commands and events run explicitly, and offers become
readable after event processing.

## Application structure and domain

- Organize the service into `core`, `app`, `infra`, `bootstrap`, and `api`
  modules, with a thin `main`. Move the existing offer skeleton into these
  boundaries and remove obsolete catalog placeholders and demonstration code.
- Keep domain types independent of Diesel, HTTP, and Mulac. Application
  handlers depend on repository interfaces; infrastructure owns database
  records and serialization.
- Identify an offer by **retailer code plus the exact supplied URL**. Validate
  absolute HTTP(S) URLs without stripping query parameters or otherwise
  merging different URLs. Assign a UUID on first import.
- Implement `ImportOfferFromPage`, which creates an aggregate or updates an
  existing one. Repeated imports with identical values produce no new domain
  event.
- Store title, URL, retailer code, current price, optional old price, and
  optional discount percentage. Use the existing `money` crate for `i64`
  minor-unit amounts and supported currencies; both prices use the supplied
  currency.
- Complete creation, update, sequence tracking, and replay. Each persisted
  event contains enough data to reconstruct the resulting offer state. Use
  explicit, versioned serialized records with lossless conversions.

## Persistence and processing

The flow is:

**REST import → Mulac command → Testimony history → Mulac event → offer
projection → REST lookup**

- Add migrations for Mulac infrastructure, Testimony's event table, an offer
  identity table, command receipts, and an offer read model. Enforce unique
  retailer/URL identities and aggregate/sequence pairs.
- Add a connection-bound `EventStore` adapter to `testimony_infra`, sharing SQL
  and conversions with its existing PostgreSQL adapter. Preserve the current
  pool-based API.
- Execute each import inside one Diesel transaction: create/find and lock its
  identity, check its command receipt, load and mutate the aggregate, append
  history, and store the resulting publishable events in the receipt.
- Use `EventSourcedRepository` with its no-op publisher inside that
  transaction. After commit, return the saved events to Mulac for publication.
- Include the generated command ID in the command payload and metadata. On
  redelivery, return the receipt's original events without appending history
  again. This makes publication retryable when history has already committed.
- Project events using full offer snapshots and aggregate sequence numbers.
  Upsert only newer sequences, making duplicate and out-of-order delivery
  harmless.
- Build a service-local Mulac adapter following `retailer_sourcing`. Register
  the import handler and projection subscriber; expose explicit batch
  processing and keep background workers disabled.
- Run synchronous database and consumer operations through `spawn_blocking`.
  Share pools and runtime handles through `Arc`; avoid the reference
  container's shared `Rc<RefCell<_>>` design.

## REST and startup

| Endpoint | Behavior |
| --- | --- |
| `POST /api/offers/import` | Validate page data, enqueue an import, return `202` with its command ID. |
| `GET /api/offers/{id}` | Return the projected offer, including its ID and sequence, or `404`. |
| `GET /api/offers?retailer_code=…&url=…` | Look up a projected offer by its identity; both parameters are required. |
| `POST /api/commands/run` | Process eligible commands using the existing sourcing-style batch request/report. |
| `POST /api/events/run` | Process eligible events and update projections using the same batch interface. |

- Reject empty titles, invalid retailer codes or URLs, unsupported currencies,
  negative prices, and discount percentages outside `0–100`. Imports replace
  the complete set of mutable fields; omitted optional prices/discounts clear
  previous values.
- Document all REST operations in OpenAPI. GraphQL retains its existing health
  operation.
- Use `DATABASE_URL`; connect and apply embedded migrations on startup when
  configured. Without it, serve health and documentation while offer and
  processing endpoints return `503`. Invalid configured database settings fail
  startup.
- Preserve `PORT`, `app()` construction, and graceful shutdown. Add environment
  examples, migration targets, and a documented import → run commands → run
  events → lookup walkthrough.
- Record the architecture and retry strategy in a new ADR and update the
  offers documentation.

## Validation and defaults

- Unit tests: creation, changes, unchanged imports, monetary validation, event
  conversion, and replay equivalence.
- PostgreSQL tests in isolated schemas: migration setup, transaction rollback,
  concurrent first imports sharing one identity, command redelivery without
  duplicate history, publication retry, and duplicate/out-of-order projection
  events.
- HTTP tests: import validation and acceptance, database-disabled responses,
  lookup behavior before and after projection, batch limits, and OpenAPI
  coverage.
- Run formatting and CI for affected crates, dependency audits, PostgreSQL
  integration checks, and a live health check.

Defaults: start with a fresh offers database; do not migrate bodulv6 data.
Exclude sourcing-service integration, catalog imports, external message
delivery, listing/search, and GraphQL offer operations from this first
implementation. Updates follow command execution order; upstream
observation-time precedence is deferred.
