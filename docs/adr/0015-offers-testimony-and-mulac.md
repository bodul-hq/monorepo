# ADR 15: Offers with Testimony and Mulac

- Status: Accepted
- Date: 2026-10-18

## Context

The offers service needs a working page-import flow based on bodulv6's domain,
application, infrastructure, and bootstrap boundaries. Testimony supplies
aggregate history and repository contracts; Mulac 0.3.0 supplies durable command
and event queues. Testimony's normal save order is append, then publish. A
publication failure after persistence must not append duplicate history when
the command retries. A projection cannot safely allocate the aggregate identity
because it may lag behind command processing.

## Decision

Keep `core`, `app`, `infra`, `bootstrap`, and `api` modules inside the existing
Poem offers crate. Domain types contain validated values and no persistence or
HTTP dependencies. Application handlers use repository interfaces. Infrastructure
owns versioned serialized records, PostgreSQL rows, and Mulac adapters.

Use retailer code plus the exact supplied absolute HTTP(S) URL as the unique
identity. Preserve URL query strings and other spelling differences. Bound URLs
to 2048 bytes so the unique index can store the key. Allocate the offer UUID in
an authoritative identity table; lock that row during each import. Imports
replace a complete page snapshot; identical values produce no event.

Add `ConnectionEventStore` to `testimony_infra`. It borrows a caller-owned
Diesel connection while sharing serialization and SQL with the existing
pool-based `PostgresEventStore`. Preserve that pool API. Within one transaction,
create/find and lock the identity, check the command receipt, replay validated
history, apply the import, append events through `EventSourcedRepository` with
a no-op publisher, and save the resulting publishable events in the receipt.

Put the same command UUID in Mulac metadata and the command payload. Return
receipt events only after the transaction commits. A retry of that command
returns its original events, even if later commands have changed the offer.
Reject reuse of a command UUID with different input. Mulac may assign new event
envelope UUIDs on retries; project full snapshots with a conditional upsert that
only accepts a higher aggregate sequence. Event names and JSON variants use
explicit `.v1` names.

Use `DATABASE_URL` for the pool and apply embedded migrations before serving.
Without it, health/docs remain available and persistence endpoints return 503.
Invalid configured settings fail startup. Call synchronous repositories and
Mulac consumers with `spawn_blocking`. Share runtime handles through `Arc`.

Expose REST import, lookup by ID/identity, and manual command/event batch routes.
Keep GraphQL health and standard documentation routes. No background consumers
or sweepers run in this version. Queue failures use Mulac backoff/attempt limits;
interrupted reservations require operator recovery. Offers become visible only
after event processing. Updates follow execution order, with no upstream
observation-time precedence. Start with a fresh database.

## Consequences

- History and command receipts commit atomically; retryable publication is
  separate. Receipt retention must outlive possible command redelivery.
- Read models tolerate duplicate and out-of-order delivery. They are eventually
  consistent and must not be used to decide whether to create an aggregate.
- A failed batch may have committed some items; Mulac cannot report their count
  in its error result. Return a null processed count plus error messages.
- Manual processing is observable through REST, but operators must run both
  queues and recover abandoned reservations/dead items when needed.
- PostgreSQL is the only runtime infrastructure dependency. Catalog imports,
  sourcing integration, external delivery, search, and GraphQL mutations remain
  separate work.
- Isolated-schema integration tests cover rollback, concurrent identity
  allocation, receipt replay, publication failure, projection ordering, and HTTP
  behavior. The library adapter preserves existing pool-based callers.
