# Item 11: Offer images from sourcing to the customers homepage

Status: completed within the user-confirmed scope below (2026-10-25).

## Follow-up contract change

The user subsequently requested initial images inside `OfferCreated`, with
separate `OfferImageAdded` events only for unseen images on updates. This
supersedes the creation-related requirements in the original plan below; see
[ADR 25](../adr/0025-initial-images-in-offer-created.md). The initial verification
record remains a record of the original implementation, not the current
creation event contract.

Follow-up verified on `d9ac2e07f16c919598a8971a09bab27d276acc7f` plus these
working-tree changes: `make fmt` and `make ci` in `services` passed; PostgreSQL
suites passed 20 offers and 19 customer-facing tests, including atomic creation
rollback and retry. A local isolated-schema run using `PORT=... make serve`
checked service health and the customers page, one creation event carrying
`[A,B]`, one update addition C at position 2, and replay retaining `[A,B,C]` in
both projections, REST and GraphQL. Logs: `/tmp/created-images-ci.log`,
`/tmp/images-offers-postgres.log`, `/tmp/images-customer_facing-postgres.log`
and `/tmp/created-images-live.log`. No dependencies or migrations changed;
audit was not required. Operational queues were not changed.

## Goal and agreed scope

Carry an ordered list of image URLs through retailer_sourcing, offers,
customer_facing and apps/customers. Render the first image on each customer
offer card. Creation emits one image-added event per image after OfferCreated;
updates append only images not already present. The user confirmed that invalid
image URLs must be skipped without preventing an otherwise valid offer import.

Originally grounded at monorepo `f5cd7bc`. Implementation was validated on
`84a4ace8adc9659843b593162ce5e44c2dc1349b` plus the working-tree changes.

**Confirmed scope adjustment:** inspection found that the assumed durable offers
handler and import receipt tables are absent from this checkout (including the
original baseline). The user explicitly chose “Keep the current handler;
implement images only.” The implementation therefore extends the retained typed
handler, propagates its errors, and does not restore durable import receipts or
claim atomicity between Testimony history and Mulac publication. Requirements
below concerning that absent handler are superseded by this scope adjustment.
Customer-facing's existing transactional receipts are retained and verified.
Existing offers acquire images on their next image-bearing import. Automatic
backfill, image removal/reordering, image downloads or object-storage uploads,
carousels, JavaScript and automatic queue workers are outside this item.

## Current behavior and constraints

- [Structuralized offers](../../lib/retailer_sauce/src/structuralized.rs) already
  contain images. [Export conversion](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/conversion.rs)
  selects title, price and URL and drops images.
- [ADR 19](../adr/0019-immutable-offer-export-preparation.md) fixes export v1's
  output and requires immutable bodies and stable snapshot/entry identities.
  [Export storage](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/store.rs)
  and its record decoder currently accept one global schema version.
- The [offers aggregate](../../services/offers/src/offers/aggregate.rs) replays
  creation, title and price events. [Stored event records](../../services/offers/src/infra/records.rs)
  have explicit versioned tags; raw delivery payloads are separate contracts.
- [HTTP publication](../../services/offers/src/orchestration/customer_facing_publisher.rs)
  and [customer-facing projection](../../services/customer_facing/src/offer_projection.rs)
  allow three event types. Customer-facing applies each original message once,
  committing its receipt and business changes together.
- The [browse API](../../services/customer_facing/src/offer_read.rs) paginates
  offers, and the [customers client](../../apps/customers/src/offers.rs) renders
  30 cards through server-generated HTML.

## Image and event contracts

Use `images: ["https://shop.example/front.jpg", ...]` in new import payloads and
REST/GraphQL offer results. Preserve source order, keeping only the first
occurrence of each exact URL. Accept absolute HTTP(S) URLs with a host, no
credentials, whitespace padding or control characters, and at most 2048
characters, consistent with the existing offer URL limit. Do not trim,
canonicalize, resolve relative URLs, strip queries or probe remote servers.
Skip invalid URL strings; an empty or entirely invalid list becomes `[]`.
Malformed field types remain contract errors. Record skipped counts without
logging complete URLs. Apply equivalent filtering to direct offers imports.

Missing `images` in legacy imports means an empty incoming list, never a
request to clear images. On create, `[A, A, B]` produces OfferCreated followed
by image additions for A and B. With existing `[A, B]`, importing `[B, C, A, C]`
adds only C, yielding `[A, B, C]`. Reordering, omission and repeated imports
produce no image events. Title and price retain their existing behavior.

Add the exact public event type/routing key `OfferImageAdded`, with raw payload:

```json
{"id":"<offer UUID>","image_url":"https://shop.example/front.jpg","position":0}
```

`position` is a zero-based, nonnegative integer assigned by aggregate append
order, represented as `u32` and stored in a compatible SQL integer type. The
stored Testimony tag is `OfferImageAdded.v1`. Keep all existing event names,
tags and payloads unchanged; OfferCreated does not acquire an image list.
Persist position because independently processed messages can arrive in a
different order. Projection reads sort by position, not arrival time. This
does not introduce revision ordering for title or price events.

## Implementation sequence

### 1. Version sourcing exports without losing retries

Introduce export schema v2 containing filtered variation images. Retain v1
conversion, decoding and serialization with its original payload shape.
Newly generated ProcessOfferForExport commands explicitly select v2; add a
schema-version field whose absence on already queued commands means v1.
Carry that version through preparation, lookup and conversion. The existing
unique key includes schema version, so v1 and v2 may coexist for one source.

Read and validate both supported versions in pending/completed record loading
and the export-message listener. Compare the row, body and event versions to
each other instead of requiring the current version everywhere. Unknown
versions fail visibly. Preserve pinned source references, pending bytes,
snapshot IDs and entry UUIDs on retries; never rewrite a v1 snapshot as v2 or
add an `images` key when reserializing a v1 entry for its existing message ID.
New v2 exports always include the array, including `[]`.

Continue publishing ImportOfferDetails through the existing HTTP inbox route.
Do not change retailer-code serialization, money, source identity, exclusion
rules or the object-storage bucket. Upgrade shared fixtures so v1 and v2
exports can each be deserialized by the offers import contract.

### 2. Extend offers imports, aggregate history and projection

Accept images in ImportOfferDetails and the temporary REST import DTO, defaulting
absent fields to an empty list. Pass images through every active command path,
including durable import handling and the typed handler where retained.

Keep ordered unique images in aggregate state. Emit creation first, then image
events in input order. Updates retain title-then-price event ordering and append
new image events afterwards. Validate the full sequence/position capacity
before any mutation; propagate errors rather than ignoring update failures.
History replay starts old offers with no images, applies additions and rejects
inconsistent positions or duplicate image entries without producing new events.

Extend stored event conversion, public serialization, subscriber registration
and outbox type validation. Preserve stable message IDs and raw payloads through
HTTP publication. Keep import receipt/event persistence atomic under retries.

Add image persistence to the offers read projection through an additive
migration. Store offer ID, image URL and position, with a parent foreign key and
uniqueness for both `(offer_id, position)` and `(offer_id, image_url)`. Return
ordered images wherever that read model is exposed. Apply the same behavior to
the in-memory repository. A repeated identical addition succeeds; conflicting
position/URL data fails. Images received before the parent exists retry normally.
Creation, title and price handling must preserve already projected images.

### 3. Project and expose images in customer-facing

Register OfferImageAdded with the command consumer and validate its complete
payload, including URL and position. Invalid event payloads fail visibly;
filtering belongs at import time, before valid domain events are produced.
Persist images through an additive migration with the same parent, position
and exact-URL constraints as the offers projection. Old offers read as `[]`.

Commit each addition and its original-message receipt in the existing
transaction. A missing parent or conflicting addition rolls back the receipt
and follows normal retries. Duplicate original message IDs remain no-ops after
later updates. Identical additions with different message IDs also succeed
without duplicate images. Out-of-order additions may leave temporary gaps;
later arrivals fill them and ordered reads recover the intended first image.

Extend REST and GraphQL offer results with a non-null ordered string array.
Keep existing fields, money representation, ordering, limit/offset and has_more
semantics. Fetch images for the selected page without multiplying offer rows
or issuing one image query per offer. Retain existing database error handling.

### 4. Render the first image in customers

Extend the server client with an images array defaulting to empty when talking
to an older API. Render only the first image in each card. Validate its URL
before emitting an escaped `src`; use escaped offer-title alt text, lazy loading,
fixed aspect ratio and object-fit containment. Missing or unsafe images use
an unobtrusive empty-image state without preventing the offer from rendering.
The browser loads retailer images directly; the app does not fetch them.
Keep price, title, retailer, purchase link and pagination usable if loading fails.
Check responsive layouts and both themes without adding JavaScript.

### 5. Documentation and rollout

Write a new ADR and index it, superseding only ADR 19's image exclusion and
ADR 21/22's three-event restrictions, while documenting v2 compatibility and
append-only image ordering. Preserve accepted ADR text. Update affected service
and app READMEs, OpenAPI/GraphQL examples, Bruno import/browse examples and
the end-to-end walkthrough.

Deploy consumers before producers: customers with missing-array compatibility,
customer-facing with its migration and new event handler, offers with its
migration/import/event support, then sourcing emitting v2. Do not roll offers
back to a binary unable to read image history after new events are stored.
Do not roll customer-facing back while image messages are pending. Retain
legacy export handling and receipts; no queue reset or historical rewrite.

## Acceptance and verification

- Conversion/contract tests: real retailer image fixtures, missing/empty arrays,
  stable deduplication, invalid URLs, all-invalid lists, wrong JSON types, exact
  query preservation and unchanged offer/exclusion behavior. Old imports remain
  accepted. V1 output remains unchanged; v2 carries images.
- Export PostgreSQL tests: pending v1 retries retain exact bytes/IDs, completed
  v1 messages retain their original payloads, v2 creates distinct exports, both
  versions survive listener replay, and unknown/mismatched versions fail.
- Aggregate tests: one addition per unique image, append-only updates, no-op
  retries, old/mixed history replay, exact event order and serialization,
  overflow without partial mutation, and invalid-history rejection.
- Offers and customer-facing PostgreSQL tests: migration preserves old data,
  original-message replay/concurrent replay, receipt rollback, duplicate and
  conflicting additions, additions before creation and reverse arrival order.
  Verify read models agree after queues finish and title/price updates retain
  images. Check publishers preserve the new event's payload and message ID.
- REST/GraphQL tests: ordered arrays, empty arrays for old rows, unchanged
  pagination with multiple images per offer, and existing error semantics.
  Customers tests cover first-image-only rendering, HTML escaping, unsafe URLs,
  legacy responses, missing images and unchanged pagination/error pages.
- Run `make fmt` then `make ci` in each changed workspace. Run relevant
  `make test-postgres` targets for sourcing, offers and customer-facing using
  isolated schemas. Audit only if Cargo dependencies or lockfiles change.
- Start each changed service/app with `PORT=<port> make serve` and check health
  or its page. In an isolated end-to-end run, process a fixture with `[A, B]`
  through export, sourcing outbox, offers inbox/commands/events/outbox and
  customer-facing inbox/commands. Verify API arrays and image A in the page;
  update with `[B, C]`, replay deliveries, and verify `[A, B, C]` with A still
  shown. Include a controlled out-of-order image delivery. Run queues manually
  and capture desktop/mobile light/dark previews, including image load failure.
- Record actual revisions, commands and results before marking TODO and this
  plan completed. Do not count skipped database or live checks as passing.


## Implementation and verification record — 2026-10-25

Implemented compatible export v2, exact ordered image filtering, append-only
`OfferImageAdded` events/history, additive child-table migrations in both
consumers, ordered REST/GraphQL arrays, and first-image customer cards. The
customers page now includes light/dark palettes. Added
[ADR 24](../adr/0024-append-only-offer-images.md), service/app and shared-library
documentation, shared v1/v2 fixtures, Bruno examples and the
[combined walkthrough](../offer-images-walkthrough.md).

Revisions: monorepo base `84a4ace8adc9659843b593162ce5e44c2dc1349b`, with this
implementation uncommitted; sibling Mulac
`9474319fff2237d81828a23d97df5d2f97741858` unchanged.

Validation performed:

- Root `make fmt`, followed by `make ci` in `lib`, `services`, and `apps`:
  passed. HTTP fixtures require running outside the restricted socket sandbox;
  earlier sandbox permission errors were rerun with local networking enabled.
  Logs: `/tmp/images-lib-final-ci.log`, `/tmp/images-services-final-ci.log`,
  `/tmp/images-apps-final-ci.log`. A final offers-only `make ci` also passed
  after updating its supported-event diagnostic (`/tmp/images-offers-final-ci.log`).
- `python3 /tmp/bodul-images-postgres.py <service>` executes each service's
  `make test-postgres` with its existing test database and per-test schemas.
  Offers: 19 passed. Customer-facing: 18 passed. Retailer sourcing: 60 passed.
  These were actual database runs, not the ignored results in ordinary CI.
  Logs: `/tmp/images-<service>-postgres.log`.
- Compatibility tests preserve v1 pending bytes and completed payload shapes,
  distinguish v2 snapshots/entry IDs, replay both versions through the listener,
  reject unknown/mismatched versions and malformed image arrays, and verify
  old commands default to v1 while new commands explicitly select v2.
- Aggregate tests cover event order, exact serialization and history tags,
  deduplication, append-only updates, legacy/mixed replay, invalid histories,
  u32 position bounds and sequence overflow without partial mutation.
- Database tests cover additive migration preservation, both projection
  constraints, memory/PostgreSQL parity, missing-parent retry, conflicting
  additions, reverse arrival, repeated/concurrent receipt replay, receipt
  rollback, retained images after title/price changes and API arrays.
- `python3 /tmp/bodul-images-live.py` started all three services and customers
  with `PORT=<port> make serve` (3141–3144), checked each health/page endpoint,
  and ran manual queues against isolated test schemas. A local in-memory
  object-storage HTTP fixture supplied structuralized documents and retained
  immutable exports; no operational object-storage objects were created.
  `[A,A,invalid,B]` exported as `[A,B]` through sourcing commands/events/outbox,
  offers inbox/commands/events/outbox and customer-facing inbox/commands.
  Position 1 deliberately arrived before position 0. Importing `[B,C]` then
  yielded `[A,B,C]` in both projections and REST/GraphQL, with A still in the
  HTML. Replayed both outboxes and customer-facing inbox messages without
  changing the images. Log: `/tmp/images-live.log`.
- Firefox WebDriver verified loaded and failed images, light/dark preferences
  and no horizontal overflow at desktop and mobile layout sizes. Reviewed
  `/tmp/images-desktop-light.png`, `/tmp/images-desktop-dark.png`,
  `/tmp/images-mobile-light.png`, `/tmp/images-mobile-dark.png`; the page
  includes loaded, missing and failed-image cards with usable purchase links.
  The test browser forced lazy-image loading before capture; production HTML
  retains `loading="lazy"` and contains no JavaScript. Servers and generated
  schemas were removed after the run.
- `git diff --check`: passed.

Audit is **not passing**. Adding `url` to shared required an audit; it was already
resolved in both lockfiles, and no resolved dependency versions changed. Root
`make audit` stopped in `lib` with existing rustls-webpki 0.103.15
(RUSTSEC-2026-0382) and webpki-root-certs 1.0.9 (RUSTSEC-2026-0383) findings.
A separate `make audit` in services reported eight findings in unchanged
rustls 0.23.45 (RUSTSEC-2026-0346/0347/0348/0349), rustls-webpki 0.103.15
(RUSTSEC-2026-0382), and zip 8.6.0 (RUSTSEC-2026-0367/0368/0369).
Warnings also include fxhash, kuchiki, rand, unsafe-libyaml, tokio
(RUSTSEC-2026-0386), and yanked num-modular. Logs:
`/tmp/images-audit.log` and `/tmp/images-services-audit.log`. These advisories
were discovered during verification but not introduced by this change; no
exceptions were added and dependency upgrades remain separate work.

Operational limits: no backfill, removals/reordering, image downloads, carousel,
JavaScript or background worker. The user-excluded durable offers import
receipt/crash guarantees were not implemented or counted as passing. The live
run uses controlled fixtures, not remote retailer image availability. Roll out
consumers before producers as described above.
