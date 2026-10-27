# Item 17: Offer scrape time through customer-facing

Status: implemented and verified on 2026-10-26.

## Goal and confirmed behavior

Add `scraped_at: Option<DateTime<Utc>>` to `OfferPageWasDownloaded` and carry
the originating download time through extraction, normalization,
structuralization, export, offers and customer-facing. The user confirmed that
the time means a successful page download, older events retain `None`, and a
successful new scrape must reach customer-facing even when title, price and
images do not change.

Customer-facing stores and exposes the latest known successful scrape time.
Creation time remains a separate value. This item does not add a timestamp to
customers' cards or introduce background queue runners.

## Starting behavior

- The [download handler](../../services/retailer_sourcing/src/offer_discovery/download_offer_page/handler.rs)
  fetches and stores a page before emitting an event without its download time.
- Intermediate handlers load the current stored artifact by offer ID. For
  example, the [extraction store](../../services/retailer_sourcing/src/offer_discovery/extract_offer_page/store.rs)
  can load a replacement downloaded after the command was queued. Simply
  forwarding a timestamp through commands could label that replacement with
  an earlier scrape time.
- [Export sources and payloads](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/entity.rs)
  support immutable schema v1/v2 snapshots. Export preparation already pins
  the structuralized object reference and preserves prepared bytes on retries.
- The [offers handler](../../services/offers/src/offers/handler.rs) emits events
  only for creation or changed title, price and images. It saves Testimony
  history before returning events to Mulac; publication is not atomic with
  that save and the current handler has no durable import receipt.
- The [customer-facing projection](../../services/customer_facing/src/offer_projection.rs)
  applies events and original-message receipts in one transaction. Its
  [read API](../../services/customer_facing/src/offer_read.rs) exposes creation
  time, but has no scrape-time field.

## Timestamp and event contract

Use a UTC instant captured once immediately after a successful HTTP fetch.
Emit the download event only after storage succeeds. Failed fetches or failed
storage do not publish a successful scrape. A genuine re-download captures a
new time; retries of later processing stages retain their source time.

Missing or explicit null timestamps decode as `None`; malformed strings fail
validation. Never manufacture a historical timestamp from queue processing,
export preparation, offer creation, UUIDs or migration time. Encode known times
as RFC3339 UTC and define microsecond precision at capture so PostgreSQL
round trips do not introduce apparent new scrapes through precision loss.

Proposed domain contract:

- Add optional `scraped_at` to imports and `OfferCreated`. Old creation records
  replay with `None`. Introduce a new stored creation version while retaining
  both existing `OfferCreated.v1` and `.v2` readers.
- Add `OfferScrapedAtUpdated` with raw payload
  `{"id":"<offer UUID>","scraped_at":"<UTC RFC3339 instant>"}` and stored
  tag `OfferScrapedAtUpdated.v1`. This event always has a known timestamp.
- On creation, include the timestamp in the creation event. On subsequent
  imports, emit the new event when the incoming known timestamp is newer than
  the aggregate's value, including imports with no other changes. Keep the
  existing title, price and image event order and append this event afterward.
- An absent, equal or older timestamp does not erase or regress a known value.
  Apply the same monotonic rule in both read projections. This rule concerns
  scrape time only; it does not establish stale-update protection for prices
  or titles. Duplicate delivery remains governed by existing receipts.

## Implementation steps

1. **Preserve download provenance.** Add the optional field to the download
   event and every downstream command/event mapping. Persist the time alongside
   the artifact it describes where intermediate rows are replaced. For new
   pipeline messages, carry immutable source references through extraction,
   normalization and structuralization, following the existing export-source
   pattern. Read the referenced object rather than a newer current row.
   Store and forward the same time with each derived artifact. Legacy messages
   without provenance remain readable with unknown time; do not combine their
   timestamp with an unverified current artifact. Cover source replacement
   between enqueue and execution explicitly.
2. **Version immutable exports.** Introduce schema v3 with optional scrape time
   on its source and each import payload, preserving image behavior. Newly
   generated export commands select v3; queued commands retain their explicit
   v1/v2 or legacy default v1. Update conversion, source validation, pending and
   completed record decoding, export-event handling and shared fixtures.
   Version-aware serialization must preserve v1/v2 payload shapes and source
   representations: never insert a new null key into an old message. Keep
   snapshot/entry identities and prepared bytes on retry. Reject conflicting
   provenance for the same immutable source/version instead of rewriting it.
3. **Extend offers.** Accept the optional field in `ImportOfferDetails` and its
   temporary REST DTO. Thread it through the active typed handler and aggregate
   state, creation, update, history replay and stored record conversion. Reserve
   capacity for the full event batch before mutation, including a scrape-only
   event. Add a nullable `TIMESTAMPTZ` column to `read_offers`, update PostgreSQL
   and in-memory repositories and register the new event with the projector,
   customer-facing subscriber and HTTP publisher allowlist. Retain raw payloads,
   stable delivery IDs and existing correlation/causation behavior.
4. **Extend customer-facing.** Add a nullable `TIMESTAMPTZ` column to `offers`
   through an additive migration, leaving old rows null. Decode the creation
   field and register `OfferScrapedAtUpdated`. Apply the monotonic timestamp
   update and original-message receipt in one transaction. An older/equal
   update for an existing offer succeeds as a no-op; an absent parent fails
   and retries without recording a receipt. Expose nullable `scraped_at` in
   REST and `scrapedAt` in GraphQL as RFC3339 UTC, preserving pagination,
   creation ordering, money and images.
5. **Document and deploy.** Update affected service READMEs, contract fixtures,
   API/Bruno examples and the pipeline walkthrough. Add an indexed ADR for v3,
   download provenance and the additional event, superseding only conflicting
   clauses of prior export/event decisions. Deploy customer-facing consumers,
   offers consumers/producers, then sourcing. Upgrade downstream pipeline
   readers before starting new timestamp-bearing downloads. Keep old export
   readers and queued work; no historical backfill or queue reset is included.

## Acceptance and verification

- Download tests prove capture at successful fetch, stable propagation and no
  event on fetch/storage failure. Each stage tests missing/null/known times,
  invalid input, restart/retry and source replacement while work is queued.
- Export tests prove v1/v2 retries retain exact payloads and IDs, v3 delivers
  the same timestamp on every exported variation, pending preparation survives
  restart, and source conflicts or unknown versions fail visibly.
- Aggregate tests cover creation with/without time; scrape-only updates;
  simultaneous field/image/time changes; equal, older and absent timestamps;
  legacy/mixed replay; and capacity failure without partial mutation.
- Isolated PostgreSQL tests cover additive migrations, both projections,
  receipt rollback, repeated delivery, reverse timestamp arrival, missing
  parents and preservation of timestamps during unrelated changes. Test UTC
  normalization and fractional-second round trips.
- REST/GraphQL tests check known and null timestamps, unchanged pagination and
  API schemas. Keep all existing smoke tests passing.
- Run `make fmt` then `make ci` from the root. Run `make audit` only if Cargo
  dependencies or lockfiles change. Run the affected services using
  `PORT=<port> make serve` with isolated databases and check `/api/health`.
  With fixture downloads and explicit queue batches, show time T1 arriving in
  both projections and APIs, then a newer identical-content scrape T2 advancing
  time without title/price/image events. Replay T1 and verify time stays T2.

## Dependencies and completion

This item can ship independently of [item 18](18-offers-event-sequence-metadata.md).
If item 18 is present, the new scrape event also carries its per-offer revision.
Coordinate the aggregate/handler edits and keep each item in a separate commit.

The current Testimony-to-Mulac publication failure window remains a known
limitation: this plan does not claim exactly-once import or atomic publication.
Verify and record failures at that boundary; repairing it requires a separately
agreed persistence change. The acceptance flow assumes successful existing
queue persistence and delivery.

Record implementation revisions, actual checks, migration and live-flow evidence
before marking this item complete. These are implementation acceptance criteria,
not evidence that the feature already exists.

## Implementation evidence

Captured microsecond UTC after fetch; immutable storage metadata pins all three
intermediate readers and persists scrape provenance. New exports use v3; v1/v2
serialization remains unchanged. Both projections and the API now carry time.

`make fmt` and root `make ci` passed. PostgreSQL suites passed with four test
threads: customer-facing 20, offers 21, sourcing 60 tests. Source replacement,
monotonic replay, receipt rollback and timestamp-only imports are covered.
Live `PORT=... make serve` checks passed for all affected services and customers.
Explicit queue batches carried T1 creation and an identical-content T2 update
through v3 export, Testimony, outbox, both projections and REST/GraphQL. Replaying
T1 emitted no new event and retained T2. Generated test schemas were removed.
No dependencies changed, so no audit was needed for this item.
