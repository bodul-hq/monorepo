# Item 18: Offers event sequence metadata

Status: implemented and committed separately; final integration verification pending.

## Goal and confirmed behavior

Store the per-offer revision in metadata for every newly emitted offers domain
event. The user confirmed per-offer numbering, rather than a service-wide
sequence. Creation is revision 1; each subsequent event increments it once,
including multiple events produced by one import. Two offers have independent
sequences.

## Existing behavior and the missing handoff

The [offers aggregate](../../services/offers/src/offers/aggregate.rs) already
assigns sequences to Testimony envelopes. The
[PostgreSQL event store](../../lib/testimony_infra/src/postgres_event_store.rs)
already persists that number in both `event_store.sequence` and
`event_store.metadata.sequence`. Do not implement a second sequence generator
or duplicate this existing storage feature.

The missing part is the offers-to-Mulac handoff: the
[handler](../../services/offers/src/offers/handler.rs) calls `get_just_events()`,
discarding the envelopes before returning events to Mulac. At sibling Mulac
revision `9474319fff2237d81828a23d97df5d2f97741858`, the typed command wrapper
constructs new metadata containing event ID, correlation ID, causation ID and
source only. Its `NewEventMetadata` and `EventMetadata` have no sequence field.
The [outbox subscriber](../../services/offers/src/subscribers/customer_facing.rs)
also constructs a fixed set of metadata fields.

The planned work therefore preserves the existing revision in Mulac
`event_entries.meta.sequence` and offers' `outbox_entries.meta.sequence`, while
retaining Testimony as the source of the revision. It applies to all current
offer event types and any new scrape event from item 17.

## Metadata contract

- Encode `sequence` as a positive JSON integer matching the exact Testimony
  envelope for that event. Do not put it into the domain payload or derive it
  from a queue ID, delivery attempt, timestamp, current row count or batch index.
- Preserve event IDs, correlation, causation, source, event types and raw event
  payloads. A persisted event's retries and outbox replay retain its sequence.
- Legacy queued events may omit sequence and continue processing. Other Mulac
  applications may omit it too; offers must supply it for every new domain
  event. Never represent missing history as sequence 0.
- Treat the existing PostgreSQL `BIGINT` limit as the persistence bound even
  though aggregate sequences are `u64`. Reject an unrepresentable batch before
  partial state mutation or writes.
- Carry sequence through event record, reservation, dispatch and subscriber
  conversions. Preserve it in stored outbox metadata. HTTP inbox forwarding
  and customer-facing ordering enforcement are separate work: the current
  inbox contract has no arbitrary event-metadata field.

## Implementation steps

1. **Add a compatible Mulac metadata path.** Recheck the sibling checkout before
   implementation. Extend the event metadata types and conversions with an
   optional sequence, using serde defaults for old JSON and omitting absent
   values on serialization. Provide a compatible typed-handler mechanism for
   carrying event-specific metadata, such as a default metadata accessor on
   `ApplicationEvent`. Its default must retain existing caller behavior and
   must not let application metadata replace framework-owned event IDs or
   correlation/causation. Update all struct literals, conversions and fixtures
   in Mulac and dependent monorepo crates. Prefer an existing supported hook if
   the sibling dependency has gained one since this plan was written.
2. **Preserve envelopes in offers.** Return a transport adapter containing the
   domain event and its assigned revision from the active command handler.
   Serialize the adapter transparently as the existing domain payload, expose
   the existing routing/event type and pass sequence through the new metadata
   path. Collect the pending envelopes before saving clears them, and return
   them only after successful persistence. Keep direct REST callers compatible
   without creating a second publication path. Do not query the latest offer
   revision after save to label all events in a batch with the same number.
3. **Retain it through delivery inside offers.** Update the customer-facing
   outbox subscriber and typed outbox metadata conversions used by offers so
   the optional field survives storage/read/retry. Do not use `extra_info`,
   which represents queue errors. Preserve raw payloads and idempotent event-ID
   based insertion. Existing events without sequence remain deliverable.
4. **Keep storage additive.** Existing JSONB columns can store the field; a
   relational schema change is not inherently required. Do not rewrite old
   histories, renumber events or infer missing queue revisions from payload
   equality. Repeated values can occur at distinct legitimate revisions.
   Confirm Testimony's existing sequence/metadata agreement with a regression
   test instead of changing its working serialization unnecessarily.
5. **Document the boundary.** Update offers and any changed Mulac library
   READMEs with numbering, optional legacy metadata and the supported range.
   Record an indexed ADR for preserving Testimony revisions through Mulac,
   including ownership of identity versus application metadata. Record the
   actual Mulac revision used; Cargo path dependencies are not revision pins.

## Acceptance and verification

- A creation event has sequence 1 in Testimony's column and metadata, Mulac's
  persisted event metadata, the subscriber envelope and stored outbox metadata.
- An update emitting title, price and two image changes after revision N has
  N+1, N+2, N+3 and N+4 respectively. A no-op import emits no event and consumes
  no revision. Loading history and updating resumes from the stored revision.
- Two offer streams number independently. The optional item 17 scrape-only
  event consumes exactly one revision; test that combination when available.
- All current event types retain their exact raw payload shape, event type and
  stored Testimony tag. No `sequence` key leaks into payloads.
- Database round trips, reservation, subscriber retry and repeated outbox
  delivery retain each event's sequence and identity. Legacy metadata without
  sequence and unrelated Mulac callers still work.
- Unit and isolated PostgreSQL tests cover invalid/zero/out-of-range metadata,
  complete-batch overflow rejection, persistence failure and existing
  concurrent-writer conflict behavior. Do not silently assign a new revision
  to an already persisted event to resolve a conflict.
- Run the sibling repository's required formatting, CI and relevant database
  tests when its metadata APIs change. In the monorepo, run `make fmt` then
  `make ci` from the root to cover path-dependent services and libraries.
  Audit only when Cargo dependencies or lockfiles change.
- Run offers with `PORT=<port> make serve` and an isolated database, check
  `/api/health`, submit creation and multi-change imports and explicitly run
  queue batches. Compare the persisted values at every handoff. Existing REST,
  GraphQL, inbox, queue and outbox smoke tests must remain passing.

## Scope, rollout and completion

Implement independently of [item 17](17-offer-scraped-at.md), preferably first
so its new event can use the same metadata path. Changes in sibling Mulac are a
separate prerequisite change under that repository's instructions; this plan
does not assume the required API already exists or change that checkout.

Deploy metadata-compatible readers before producers that rely on the field.
Keep old metadata optional and avoid historical backfills. Sequence metadata
does not by itself guarantee queue ordering, reject stale customer updates,
deduplicate repeated imports or close the existing Testimony-save/Mulac-publish
failure window. Preserve the current handler's scope and document that limit.

Record actual implementation revisions, CI/database results and live event
samples before marking TODO and the index complete. Keep this change separate
from scrape-time functionality.

## Implementation verification

Implemented alongside items 17–20. See the [verification record](17-20-verification.md)
for completed checks, prerequisite revision, audit comparison, and the remaining
database/live checks. TODO is marked done at the user’s request; the verification
record retains the outstanding checks.
