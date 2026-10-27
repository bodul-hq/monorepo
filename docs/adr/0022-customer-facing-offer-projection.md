# ADR 22: Customer-facing offer projection with original-message receipts

- Status: Accepted; supersedes ADR 18's bootstrap-only handler policy
- Date: 2026-10-24

## Context

Customer-facing already durably accepts inbox messages and manually processes
queues. Incoming offers events must maintain its own offer model. Mulac turns
each inbox routing key into a command type, preserving the original message ID
as command causation. Retrying inbox processing can generate another command
UUID for the same event. The current kernel's typed handler wrapper passes only
the payload to handlers, omitting that original identity.

## Decision

Add service-owned `offers` and `applied_offer_messages` tables through an
additive migration. Preserve offers' business properties and identity constraints,
using shared `RetailerCode` and `Money` types for wire decoding. No other
service's database is consulted.

Assemble a Mulac command consumer with an `ErasedCommandHandler` that receives
the whole command envelope. Bind this consumer to the existing command HTTP
adapter; retain the persistent kernel for the other existing queue wiring and
shutdown lifecycle. Keep Mulac's reservation, validation, blocking execution,
completion, and retry behavior. This avoids changing the sibling Mulac checkout
or looking up metadata by payload or transient command identity.

Handle exactly `OfferCreated`, `OfferTitleChanged`, and
`OfferPriceChangedEvent`. Apply changes directly at the command stage without
emitting synthetic events. Require original message causation metadata. Insert
a receipt keyed by that UUID and apply the business write in one PostgreSQL
transaction. `INSERT ... ON CONFLICT DO NOTHING` serializes concurrent claims;
an existing committed receipt means successful replay without another write.
Any application failure rolls back the receipt. Queue completion is separate,
so a crash between commit and completion is safe to retry.

## Consequences

Creation conflicts and updates before creation fail visibly and follow existing
retry limits. A valid duplicate succeeds even after later updates or restart.
Receipts must be retained for the replay lifetime. All supported messages
remain manually driven; the event runner has no projection work and the outbox
publisher remains unconfigured. Health and startup contracts stay unchanged.

Distinct updates follow processing order, without source revisions or stale
update rejection. No API for browsing offers, backfill, or automatic queue
recovery is introduced. PostgreSQL tests use direct inbox submissions and
isolated schemas to verify receipt, application, replay, rollback, concurrency,
and migration preservation independently of an upstream publisher.
