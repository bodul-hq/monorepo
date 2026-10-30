# TODO implementation plans

Plan numbers follow the original [TODO.md](../../TODO.md) sequence; the EU locale
clarification extends item 22. Status markers follow the
[TODO workflow](../../AGENTS.md#todo-workflow): `[ ]` pending, `[.]` planning,
`[>]` planned, `[~]` implementing, and `[x]` implemented.

## Current round and dependencies

[Plan 34: Inbox and outbox statistics](34-inbox-outbox-statistics.md) is implemented
for all three dashboard services, extending the existing UTC-day reports and
colored boxes to all four queues. PostgreSQL and live checks remain blocked
by environment configuration and socket permissions; see its verification record.

[Plan 33: Today's command and event status counts](33-today-entry-status-counts.md)
is planned for all three dashboard services. It supersedes plan 32's preservation
of all-time command/event counts; implementation has not started.

[Items 17–20 implementation and verification](17-20-verification.md) records the
separate implementation commits and remaining integration checks.

| Item | Work | Status and prerequisites |
| --- | --- | --- |
| 1 | Upgrade sourcing to local Mulac | Completed. [Original plan](01-retailer-sourcing-local-mulac.md). |
| 2 | Customer-facing queues and health | Completed. [Original plan](02-customer-facing-queue-runners.md). |
| 3 | Processing for export | Completed. [Original plan](03-processing-for-export.md). |
| 4 | Record export messages in sourcing outbox | Completed. [Handoff](04-offer-export-outbox-listener.md). |
| 5 | Publish sourcing outbox to offers | Completed. [Original plan](05-retailer-sourcing-http-outbox.md). |
| 6 | Publish offers outbox to customer-facing | Completed. [Handoff](06-offers-http-outbox.md). |
| 7 | Persist customer-facing offers | Completed. Transactional projection and original-message replay protection verified with direct inbox submissions. [Handoff](07-customer-facing-offer-projection.md). |
| 8 | Paginated REST and GraphQL offers API | Completed. [API contract and evidence](08-customer-facing-offers-api.md); isolated PostgreSQL and live HTTP checks passed. |
| 9 | Customers homepage with 30 offers | Completed. [Homepage plan and evidence](09-customers-offers-homepage.md); 30/1 pagination, projected updates and desktop/mobile previews verified. |
| 10 | Customer-facing dashboard reports | Completed. [Plan and evidence](10-customer-facing-dashboard.md); authenticated reports, isolated PostgreSQL checks, and four-service dashboard previews verified. |
| 11 | Offer images through sourcing, offers, customer-facing and customers | Completed within confirmed scope (current offers handler retained). [Image support plan](11-offer-images.md); append-only events, compatible export versioning and first-image rendering. Builds on items 3–9. |
| 12 | Customers offer links open a new tab | Completed. [Plan and verification](12-customers-offers-new-tab.md); 15 customers tests and responsive browser checks passed. |
| 13 | Retailer code over the offer image | Completed. [Plan and verification](13-customers-retailer-image-overlay.md); 15 customers tests and responsive browser checks passed. |
| 14 | Price and offer action share a row | Completed. [Plan and verification](14-customers-price-offer-link-alignment.md); 15 customers tests and responsive browser checks passed. |
| 15 | Currency symbols and fixed per-currency price formatting | Completed. [Display contract and verification](15-customers-currency-formatting.md); all 13 currencies, exact integer limits, accessible descriptions and responsive browser checks. |
| 16 | Plain-text offer titles | Completed. [Text contract and verification](16-customers-plain-text-titles.md); HTML fragment parsing, shared display titles, escaping and responsive browser checks. |
| 17 | Scrape time through customer-facing | Implemented and verified. [Timestamp propagation plan](17-offer-scraped-at.md); successful download time, legacy unknown values, immutable exports and scrape-only updates. Independent of item 18. |
| 18 | Per-offer event revision in metadata | Implemented and committed separately; final integration verification pending. [Sequence metadata plan](18-offers-event-sequence-metadata.md); preserve existing Testimony revisions through Mulac and offers outbox storage. Requires a compatible Mulac metadata path. |
| 19 | Source-backed target market and language through customers | Implemented and committed separately; final integration verification pending. [Market and language plan](19-offer-market-and-language.md); explicit source evidence, unknown/conflict handling and data-only customers support. Coordinate export/event versions with item 17. |
| 20 | GTIN, SKU and external identifiers through customers | Implemented and committed separately; final integration verification pending. [Identifier propagation plan](20-offer-identifiers.md); optional typed identifiers, immutable export compatibility and data-only customers support. Coordinate versions with items 17 and 19. |
| 21 | Fuzzy offer title search and customers search page | Implemented; restricted-environment acceptance checks pending. [Search plan](21-offer-title-search.md); typo-tolerant relevance ranking, limit/offset pagination and a centered HTML header form. Independent of items 17–20. |
| 22 | Source-locale market fallback and primary language | Implemented; database/live verification pending. [Locale normalization plan](22-locale-market-fallback.md); derive missing markets from supported country regions and normalize every new observation’s language. Changes item 19’s selection policy; retain immutable history. |
| 23 | Customer-facing Shopify identifiers in JSON | Implemented; PostgreSQL/live verification pending. [Storage migration plan](23-customer-facing-shopify-identifiers-json.md); preserve all existing values in a typed JSON object before dropping three columns. Extends item 20; keep public contracts compatible. |
| 24 | [Comparison pricing](24-offer-comparison-pricing.md) | Implementation and acceptance tests present; database/storage/live execution pending. Fields introduced in v6; source writer v8, creation writer v9. |
| 25 | [Availability and condition](25-offer-availability-condition.md) | Implementation and acceptance tests present; database/storage/live execution pending. Fields introduced in v7; source writer v8, creation writer v9. |
| 26 | [Description content](26-offer-content.md) | Implementation and acceptance tests present; database/storage/live execution pending. Source export v8; creation now v9 under item 28. |
| 27 | [Identity product identifiers](27-identity-product-identifiers.md) | Implementation and acceptance tests present; database/live execution pending. No wire version change. |
| 28 | [Localized offer content](28-localized-offer-content.md) | Implemented. Merge language entries in offers and downstream; retain source v8 inputs and flat display/search fields. Database/live checks pending. |
| 29 | [Sourcing command workers](29-sourcing-command-workers.md) | Implemented. COMMAND_WORKERS defaults to four; events/outbox remain manual. Database/live checks pending. |
| 30 | [Offer download delay](30-offer-download-delay.md) | Implemented. Six seconds before each handler fetch, including retries; portable ordering tests pass. |
| 31 | [Sitemap fetch delay](31-sitemap-fetch-delay.md) | Implemented. Random inclusive 1–5 seconds between actual fetch attempts; portable traversal/pacing tests pass. |
| 32 | [Daily sourcing queue statistics and dashboard](32-daily-queue-statistics-dashboard.md) | Planned. Entries scheduled for today in UTC, grouped by current status across all four sourcing queues; separate development dashboard. |

Item 32 was planned with a dedicated sub-agent against monorepo `f0c4e50` and
Mulac `fd29f72`. The user confirmed `scheduled_at` and UTC day boundaries.
The plan covers the authenticated development endpoint, a separate dashboard,
consistent query snapshots, compatibility and acceptance tests. Implementation
has not started.

Items 28–31 were planned separately with sub-agents against monorepo `9fb43bc`
and Mulac `fd29f72`. The user confirmed scope, language merging, compatibility
fields, four configurable command workers, per-handler download pacing and
replacement of sitemap batch pauses. Items 30/31 precede enabling item 29;
item 28 proceeded independently. All four implementations are complete; see
their [verification record](28-31-verification.md) for checks and runtime limits.

[Items 24–27 verification](24-27-offer-facts-verification.md) records the current
checks and environmental limitations. Their completion plans were reviewed
separately by one sub-agent per item against `d256fbc` on 2026-10-26. Each plan
identifies the existing implementation, remaining work, exact compatibility
requirements, validation commands, and evidence needed to mark that item done.
The 2026-10-27 follow-up from `dfe0de1` added the missing shared export recovery,
independent fact transitions, projection/receipt/migration and queued identifier
tests. No production defect was found in executed checks. See the verification
record for current commands/results; ignored database cases remain unexecuted.

Item 27 can be completed independently. Items 24–26 share the pipeline and
additive migrations: v6 introduced comparison pricing, v7 statuses, and v8
content; source exports remain v8 and item 28 advances offer creation to v9.
Run the common environment setup and full CI
once, then assess each item's acceptance separately. Coordinate the persisted
export/retry fixtures in items 24–26: a shared current-v8 case can cover all
three facts while retaining legacy and introductory-version cases. Deploy downstream readers
and migrations before the sourcing writer; stop event runners for the identity
backfill. Existing code and migrations should be verified and repaired where
necessary rather than recreated. Update TODO markers using the workflow above
and record verification results and limitations in the linked reports.

Items 21–23 have separate implementation plans, reviewed by one sub-agent per
item against monorepo `ed55cb1`. Item 21 retains the confirmed typo-tolerant,
ranked search scope. Item 22 follows the updated requirement to normalize language
for every new observation and derive a country market only when missing; this
requires a new ADR superseding the affected localization policy. Item 23 preserves
existing identifier values during migration, as confirmed by the user.

These items can be implemented independently. Coordinate items 21 and 23 where
search reuses customer-facing’s offer record/DTO mapping, and allocate distinct
migration and ADR numbers at implementation time. Existing immutable exports and
history remain readable; no queue reset or unsolicited historical rewrite is
part of these plans.

Item 20 is grounded at monorepo `c8ad6e8ff6da4b229c668ea5d89c1e12260b7783`.
The user confirmed `Option<Sku>` for SKU and customers data support without
card display changes. It preserves retailer/URL offer identity and carries
product identifiers as additional data. Existing plans remain separate.

Item 19 adds a separate handoff grounded at monorepo
`d6686d7290f7ca8050564476a3242007d3bdfefc`. The user confirmed target sales
market rather than shipping destinations, and customers data support without
card display changes. Extraction must preserve evidence and leave unsupported
or conflicting values unknown. It is independently implementable; coordinate
shared contracts and schema versions with items 17 and 18.

Items 17 and 18 are separate implementation handoffs, grounded at monorepo
`fbe3f7d2eb660c5860728dada40b8d9a32490e6f` and sibling Mulac
`9474319fff2237d81828a23d97df5d2f97741858`. The user confirmed per-offer event
revisions and successful download timestamps that also advance on otherwise
unchanged offers. Implementing item 18 first is convenient, but neither item
depends on the other. Both preserve the current offers handler's known
Testimony-to-Mulac publication limitation and require their own verification.

Items 15 and 16 completed the previous TODO round on 2026-10-26. Their combined
card layout was verified at desktop, tablet and mobile widths in both themes.
The user confirmed fixed formatting per currency for item 15. The customers CI
passes with 21 tests. Dependency audit still reports existing advisories; see
item 16's completion evidence.

Items 4, 6, and 7 can be implemented and verified independently. Item 4 needs
no customer-facing work. Item 6 completes at durable inbox acceptance and does
not require item 7. Item 7 can consume directly submitted contract fixtures
without item 6. A combined walkthrough follows once the pieces are available.

The new handoffs specify requirements, current contracts, scope boundaries,
failure expectations, and acceptance evidence. Implementation design and code
organization are left to the implementing tool. Source links provide evidence
of existing behavior, not prescribed edit locations.

Plans 1, 2, 3, and 5 are historical planning documents; some wording predates
their implementation. For implemented behavior, consult current code, service
READMEs, and accepted ADRs rather than treating old provisional statements as
outstanding decisions.

## Agreed cross-service boundaries

Items 6 and 7 contain the same shared message contract. HTTP inbox routing keys
are the exact event types `OfferCreated`, `OfferTitleChanged`, and
`OfferPriceChangedEvent`. Raw event payloads and stable message UUIDs are
preserved. Mulac converts inbox messages to commands; a newly generated command
UUID is distinct from the original message identity.

Item 11 plans to extend this contract with `OfferImageAdded` and an image position for
stable ordering. It preserves existing event payloads and adds export schema
v2 while retaining v1 retries. Its invalid-image policy was confirmed by the
user: skip invalid URLs and import the otherwise valid offer.

HTTP replaces offers' active RabbitMQ delivery during this phase. Missing HTTP
configuration leaves publication unavailable; no fallback or dual publication
is requested. [ADR 20](../adr/0020-temporary-http-outbox-delivery.md) covers only
sourcing-to-offers. Item 6 requires a new ADR for the offers-to-customer-facing
change to [ADR 16](../adr/0016-offers-rabbitmq-outbox.md).

Customer-facing must tolerate duplicate replay without regressing already
applied state. Distinct updates apply in processing order. Offers now retains
revisions in its own queue metadata; HTTP forwarding does not carry that metadata.
Source-order guarantees and stale-update protection are outside this round. Updates arriving before creation fail visibly and
follow normal retries rather than creating partial offers.

## Environment and verification

The new plans were grounded at monorepo revision
`5315da8f36e50be3d70d0da7f873f34262992a6c` and sibling Mulac revision
`9474319fff2237d81828a23d97df5d2f97741858`. Local Cargo path dependencies
use the checked-out files, so implementers must record the actual revision
they validate and recheck contracts when either checkout changes.

The services share a Cargo workspace. Coordinate shared dependency and lockfile
changes while keeping each item's acceptance independently verifiable.
Follow applicable `AGENTS.md` instructions, formatting before CI, relevant
isolated PostgreSQL tests, and live service health checks. Audit only when Cargo
dependencies or lockfiles change. Never claim a skipped integration check passed.

Each handoff describes its own completion evidence and operational boundaries.
Items 28–31 add command workers and projection-content backfills to the earlier
scope. Events and outbox delivery remain manual. Operational database resets
are outside the implementation scope.
