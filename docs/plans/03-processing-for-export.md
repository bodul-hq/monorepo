# Processing structuralized offers for export

Status: Draft. Export interpretations and legacy-event handling below are pending.

TODO item: “Add flow for converting data from `structurize` to `offer_export`:
name it `processing for export`.”

Add a `processing_for_export` feature slice in `retailer_sourcing`. It consumes
the existing structuralization result, prepares the data needed by offers, and
emits the export event after durable storage succeeds. It can be implemented
using the existing `Bus`, `Handle`, and `Event` interfaces; the local Mulac
upgrade is coordination work, not a prerequisite. Registering an outbox
listener and publishing HTTP requests belong to later TODO items.

## Decisions awaiting interpretation

The implementation steps describe the recommended choices below. These choices
are proposals, not accepted requirements; revise the dependent steps after the
user answers.

| Question | Recommended interpretation | Other interpretations offered |
| --- | --- | --- |
| Which structuralized variations become exports? | Export each priced variation; record a reason for every variation skipped because its price is missing. | Include missing-price variations and extend the offers contract; select a representative variation; forward the whole structuralized document. |
| What is `offer_export`? | A persisted immutable export snapshot, with an event referencing it. | An event carrying the complete export payload; a DTO persisted only in existing event/outbox rows. |
| How should existing events without a pinned source be handled? | Require fresh structuralization so each export has an unambiguous source. | Explicitly catch up from the current snapshot; support both as named operator actions. |

Selecting missing-price exports or whole-document forwarding changes the
existing offers import contract and the dependency assessment. Do not silently
introduce either into the first implementation.

## Current behavior and contracts

- [`structuralize_offer::Handler`](../../services/retailer_sourcing/src/offer_discovery/structuralize_offer/handler.rs)
  uploads a JSON document, saves its reference, and returns
  [`OfferWasStructuralized`](../../services/retailer_sourcing/src/offer_discovery/structuralize_offer/event.rs).
  That event currently contains the sourcing offer ID, consolidated sitemap ID,
  retailer code and page URL. It carries neither the document nor its immutable
  storage reference.
- [`offer_details_structuralized`](../../services/retailer_sourcing/src/offer_discovery/structuralize_offer/store.rs)
  has one current row per sourcing offer. Each replacement uploads an immutable
  object and replaces the row's reference. Reading by `offer_id` returns the
  last committed snapshot, which may differ from the document that originally
  caused a queued event.
- Live documents have a `variations` array. For example,
  [Anker AU](../../lib/retailers_anker/src/ankerau/offer_details/v1/structuralize.rs)
  provides title, price and a variant-specific URL;
  [Anker US](../../lib/retailers_anker/src/ankercom/offer_details/v1/structuralize.rs),
  [Ugreen CA](../../lib/retailers_ugreen/src/ugreenca/offer_details/v1/structuralize.rs),
  and [Minisforum US](../../lib/retailers_minisforum/src/minisforumus/offer_details/v1/structuralize.rs)
  have optional price fields. The shared `retailer_sauce::structuralized::Offer`
  type does not describe these actual producer documents.
- Offers accepts the `ImportOfferDetails` command with `retailer_code`, `title`,
  `price: { amount, currency }`, and `url`. Its
  [command](../../services/offers/src/offers/command.rs) and
  [HTTP conversion test](../../services/offers/src/offers/http/rest.rs) are the
  contract references. Retailer enum serialization produces values such as
  `AnkerAu`; a lowercase retailer slug is not the same wire representation.

## Implementation steps

### 1. Define and verify the conversion boundary

Add a typed input projection for the fields needed from `variations`, and an
export DTO matching the existing import command. Keep both in the new slice;
do not make sourcing depend on the offers binary crate. Use existing shared
`RetailerCode` and `Money` types. Sourcing does not currently depend directly on
`money`: declare it once in the services workspace as
`money = { path = "../lib/money" }` and inherit it in sourcing and offers.
Regenerate the services lockfile and audit the changed dependency graph.
Ignore unrelated retailer fields while rejecting malformed required export
fields. Add explicit guards for nonblank titles, nonnegative price amounts, and
absolute HTTP(S) URLs, following the constraints in offers' HTTP conversion;
typed deserialization alone does not enforce these business rules. Preserve
valid values rather than trimming or rewriting them.

Under the recommended interpretation, prepare one export entry for each priced
variation. Preserve its title, exact URL, integer minor-unit amount and currency.
Take retailer identity from the structuralization event. Do not substitute the
parent page URL, normalize query strings, convert prices through floating point,
or invent zero for an absent price. Availability is not an export filter in this
proposal. Keep images and identifiers out of the import payload because offers
does not accept them today; the source snapshot remains available for later work.

Record missing-price exclusions with the source variation index, available
identity/URL and a reason. Distinguish a legitimate empty variations array from
a malformed document. Invalid typed data fails preparation; missing prices
follow the user-selected policy. Detect repeated exact retailer/URL identities
within one document and report them instead of relying on eventual queue order
to choose a winner.

### 2. Pin the source and allocate stable export identities

An export must retain the immutable source reference it actually used. Carry
the complete typed structuralized storage reference into newly emitted
`OfferWasStructuralized` events and the new `ProcessOfferForExport` command.
Use a compatibility-aware optional field when decoding existing queued events;
newly produced events must always supply it. Extend the structuralization store
result narrowly to expose the reference already created during its write.

Before implementation, explicitly settle handling of legacy queued events that
lack that reference. The historical document cannot be recovered reliably from
the current metadata row. Either resolve and persist the current reference as
an explicitly documented legacy catch-up operation, or require a fresh
structuralization run. Do not claim that a latest-row lookup reproduces the
historical event's input. Keep reactions pure: any current-reference lookup and
pinning belongs in the handler/store, not the event-to-command conversion.

Key export preparation by sourcing offer ID, pinned source object ID and export
schema version. Persist a UUIDv7 export snapshot ID and per-entry IDs once, then
reuse them on retries and concurrent delivery of that same source snapshot.
A later source object or an explicit schema-version change creates a new export.
An entry ID is a delivery identity; it must not replace the offer identity that
the receiving service owns.

Persist preparation state before uploading the export object, without holding
a database connection across HTTP. A retry reads the same source reference,
IDs and prepared representation. Use the persisted export ID as the object
storage idempotency key, rather than calling `create_offer`, which generates a
new external ID on every call. Define a recoverable preparation/completion
transition so an upload success followed by a database failure can retry with
identical bytes and the same key.

### 3. Add export persistence

Follow [ADR 7](../adr/0007-domain-entities-and-postgres-records.md),
[ADR 10](../adr/0010-typed-record-metadata.md), and
[ADR 12](../adr/0012-pipeline-object-storage.md): domain types in `entity.rs`,
private `*PgRecord` types in `store/record.rs`, and typed storage/reference
metadata using `Json<T>`.

For the proposed persisted-snapshot interpretation:

- Add an additive migration for `offer_export` metadata and the durable
  preparation state. Store export ID, source offer/sitemap/retailer references,
  pinned source object reference, schema version, status, timestamps, and typed
  result counts. Enforce uniqueness on the source-snapshot/version key.
- Keep completed export bodies immutable in an `offer_export` object-storage
  bucket. The body contains ordered entries with stable entry IDs and import
  payloads, plus the recorded exclusions. Preparation state must retain enough
  information to reproduce the exact upload after a crash.
- Add that bucket to the object storage
  [allowlist](../../services/object_storage/src/model.rs), its tests and docs,
  and sourcing's [object storage adapter](../../services/retailer_sourcing/src/platform/object_storage.rs).
  Deploy the allowlist before enabling export production.
- Validate retrieved object ID, bucket, external ID and byte length before JSON
  decoding. Release database connections before object-storage calls.
- Preserve existing pipeline and queue data. Rollback removes only the new
  feature's schema; document that already uploaded immutable artifacts remain.

Record the immutable export identity, durable upload preparation, and
source-reference extension in a new ADR and link it in the ADR index. Durable
preparation is an export-specific change to ADR 12's upload-before-local-write
and no-durable-replay policy. Identify the superseded portions, including event
compatibility, rather than editing that accepted ADR.

### 4. Register the processing slice and publish its result

Create `offer_discovery/processing_for_export/` with `command.rs`, `event.rs`,
`entity.rs`, `handler.rs`, `reacts_to.rs`, `store.rs`, `store/record.rs`,
`prelude.rs`, and focused tests, following the neighboring feature slices.

Register its reaction to `OfferWasStructuralized` and its handler through
[`offer_discovery::prelude::register`](../../services/retailer_sourcing/src/offer_discovery/prelude.rs).
The existing bus continues to enqueue and execute commands only through the
manual queue endpoints. A structuralizer returning no document still produces
no downstream work.

The handler resolves the pinned source, prepares/reuses the snapshot, completes
its durable storage, then returns the export event through `Handle`. Document
its stored event name and schema before release. Its payload references the
stable export ID, source identity and schema version; later subscribers load
the completed snapshot through a public domain-facing read operation.

Return the same snapshot reference on retry. Mulac may persist duplicate event
envelopes after a failure boundary, so downstream deduplication must use stable
export entry IDs rather than newly generated envelope IDs. An empty completed
snapshot is a valid processed result with zero publishable entries. Storage or
conversion failures emit no success event and remain visible to the queue's
existing retry/error handling.

Do not add the outbox subscriber, outbox publisher, offers HTTP calls, or new
HTTP routes in this change. Exposing the completed snapshot and event is the
handoff to those TODO items.

## Verification and acceptance

Use checked-in representative structuralized fixtures from Anker, Ugreen and
Minisforum. Cover multiple variants, exact variant URLs, missing prices,
empty variations, duplicate identities, malformed money, supported zero-decimal
currencies, zero amounts, and preservation of existing integer `Money` values.
Assert the serialized payload matches the existing offers command shape;
verify a fixture through that actual command deserializer in offers' tests.

Unit tests cover reaction routing, pure conversion, recorded exclusions, and
no success event on preparation or storage failure. Isolated PostgreSQL tests
cover additive migration/down/up behavior, concurrent preparation, stable IDs,
source replacement after pinning, and successful upload followed by failed
completion/retry. The object-storage test double must verify reuse of the same
external ID and bytes. Extend the existing opt-in live object-storage checks
for the new bucket. Exercise commands/events in separate batches and assert
that a completed export is readable without either downstream service running.

After implementation, run `make fmt` and `make ci` for affected folders, plus
`make test-postgres` for sourcing and the relevant object-storage checks. Run
`make audit` only if dependencies or a lockfile change. Start each changed
service with a distinct `PORT=<port> make serve` and check `/api/health`;
sourcing needs its documented database and object-storage configuration for
the pipeline walkthrough. Update service READMEs and root pipeline documentation
to describe the new stage, legacy-event handling and manual batch sequence.

Completion means a structuralized document produces one durable export snapshot
with stable entry identities and an export event, according to the confirmed
variation policy. Reprocessing a pinned snapshot reuses its preparation and
completed result; it does not require the local Mulac upgrade, an outbox
listener, or a running offers/customer-facing service.
