# Item 20: Offer identifiers through customers

Status: implemented and committed separately; final integration verification pending.

## Goal and confirmed scope

Carry identifiers from each structuralized variation through `OfferImport`,
offers history and projection, customer-facing persistence and APIs, and the
customers app's response model. The user confirmed `sku: Option<Sku>`; the
original TODO's `Option<Gtin>` for SKU was a typo. Customers receives the data
without displaying identifiers on cards.

Preserve source values exactly. This work does not introduce product matching,
cross-retailer deduplication, identifier-based search, new offer identities or
additional extraction heuristics. Missing identifiers are valid.

## Current behavior

Grounded at monorepo `c8ad6e8ff6da4b229c668ea5d89c1e12260b7783`.

- [Shared identifier types](../../lib/retailer_sauce/src/identifier.rs) already
  define `Gtin`, `Sku`, `ShopifyProductGid`, `ShopifyProductVariationGid` and
  `ShopifyHandle`. They serialize as strings. Their constructors reject empty
  strings, but transparent derived deserialization does not call those
  constructors. GTIN checksum/length validation is not currently enforced.
- [Anker structuralization](../../lib/retailers_anker/src/ankerau/offer_details/v1/structuralize.rs)
  emits a nested `identifiers` object per variation, with GTIN, SKU and all
  three Shopify fields. Other storefronts can have optional SKU/GTIN; existing
  export fixtures also contain partial identifier objects.
- [Export conversion](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/conversion.rs)
  ignores that object. [OfferImport](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/entity.rs)
  and the [receiving command](../../services/offers/src/offers/command.rs)
  currently carry title, price, URL and images only.
- Despite its name, [read_offers_identifiers](../../services/offers/src/read_offers_identifiers/entity.rs)
  maps retailer and URL to the internal offer ID. It is separate from the
  product identifiers in this item; its matching behavior remains unchanged.
- The [offers projection](../../services/offers/src/read_offer/entity.rs),
  [customer-facing API](../../services/customer_facing/src/offer_read.rs) and
  [customers client](../../apps/customers/src/offers.rs) do not carry these fields.

## Data contract

From `OfferImport` onward, use these field types:

```rust
gtin: Option<Gtin>,
sku: Option<Sku>,
external_identifiers: ExternalIdentifiers,

pub struct ExternalIdentifiers {
    pub shopify_product_gid: Option<ShopifyProductGid>,
    pub shopify_product_variation_gid: Option<ShopifyProductVariationGid>,
    pub shopify_handle: Option<ShopifyHandle>,
}
```

Define `ExternalIdentifiers` alongside the existing shared identifier types,
with a default containing three `None` values. Reuse the existing wrappers and
their string representation rather than introducing competing definitions.
Keep API DTO mapping separate where OpenAPI/GraphQL require string fields.

The new wire shape is additive at the offer level:

```json
{
  "gtin": "0012345678905",
  "sku": "A1-BLACK",
  "external_identifiers": {
    "shopify_product_gid": "gid://shopify/Product/1",
    "shopify_product_variation_gid": "gid://shopify/ProductVariant/2",
    "shopify_handle": "usb-charger"
  }
}
```

GTIN/SKU and each nested field may be null. The external object is present in
new exports and read responses, even when all members are null. Old payloads
may omit it, which defaults to an empty identifier set; explicit null instead
of an object is invalid. Omitted nested fields default to `None`. Numbers,
arrays and other wrong field types are rejected, not coerced into strings.

Validate present values explicitly at new export/import boundaries. Reuse
constructor semantics and reject embedded NUL before persistence; derived
deserialization alone is insufficient. Preserve case, leading zeros and full
GID strings. Do not trim, convert identifiers to numbers, generate a handle
from the URL or substitute a product GID for a variation GID. Do not silently
add stricter GTIN checks or global GID-format rules in this propagation change.
Malformed present identifiers fail visibly; absence remains valid.

## Proposed update and event behavior

- On creation, store the supplied identifiers in aggregate state and
  `OfferCreated`. Old creation payloads replay with no identifiers.
- On later imports, each supplied non-null identifier replaces that field if
  different. Missing/null identifiers mean no new information and retain
  previously known values. This lets legacy imports and partial identifier
  sets coexist without clearing data. Identifier removal is not included;
  it would require a separate explicit clearing contract.
- Emit one `OfferIdentifiersChanged` when the resulting identifier set changes,
  including an identifier-only import. Its payload contains `id`, `gtin`,
  `sku` and the complete resulting `external_identifiers` object, using the
  same wire types. An identical import emits no identifier event.
- Store the new event as `OfferIdentifiersChanged.v1`. Allocate a new stored
  creation version, retaining all historical readers. Preserve existing
  event ordering and append the identifier event after other updates. Reserve
  sequence capacity for the whole batch before mutating state.
- Both read projections apply the event's complete resulting set. Import
  merge semantics must not be applied again when replaying history or
  projecting an already-decided event.

Identifier changes never change the internal offer ID or the retailer/URL
lookup key. Two different offers may share a GTIN, SKU or external identifier;
do not add uniqueness constraints or merge offers based on these fields.

## Implementation steps

1. **Share types and validation.** Add `ExternalIdentifiers` with serialization,
   defaults and equality support. Introduce boundary guards using the existing
   identifier types. Reuse `retailer_sauce` without depending on a retailer
   implementation crate. Declare dependencies shared by multiple consumers in
   the relevant workspace manifest, keep entries sorted, and inspect dependency
   impact before adding them. No type relocation or new crate is required by
   this plan. Keep existing structuralizer contracts and callers compatible.
2. **Map structuralized identifiers into exports.** Extend the converter's
   variation decoder to accept absent or partial `identifiers` objects. Read
   GTIN/SKU from that variation and group its Shopify fields into
   `external_identifiers`. Preserve variation association and all existing
   price/exclusion rules. An absent legacy object becomes all unknown; wrong
   present types fail conversion. Verify Anker, Ugreen and Minisforum fixtures,
   including non-Shopify/partial cases without fabricated values.
3. **Version immutable exports.** Introduce schema v5 for identifiers, reserving
   v3 for item 17 and v4 for item 19. Recheck shipped contracts before choosing
   the final version; never reuse an existing version with new semantics.
   Support all earlier versions that actually shipped; skipped numbers are
   acceptable. Update command selection, conversion, validators, pending and
   completed record readers, listener serialization and fixtures together.
   Version-aware serialization must omit the new fields entirely for old
   snapshots, including the default external object. Preserve pending bytes,
   snapshot/entry IDs and old outbox payloads on retry. Add a new shared import
   fixture rather than changing the expected v1/v2 payloads.
4. **Extend offers end to end.** Add the fields to `ImportOfferDetails` and the
   temporary REST request/conversion path. Validate before any aggregate write.
   Extend state, create/update behavior, replay, record conversion and the
   active handler. Add nullable text columns for GTIN, SKU and the three Shopify
   values to `read_offers` using an additive migration. Update the repository
   trait and PostgreSQL/in-memory implementations. Register the new event with
   the projector, customer-facing subscriber and HTTP publisher allowlist.
   Keep raw event payloads, message IDs and correlation/causation intact.
5. **Extend customer-facing.** Add equivalent nullable columns to `offers`.
   Handle creation and `OfferIdentifiersChanged`, committing each projection
   change with the existing original-message receipt. Missing-parent updates
   fail and retry without a receipt. Expose nullable string GTIN/SKU and a
   non-null external-identifiers object with nullable string members in REST
   and GraphQL, following each API's naming conventions. Retain pagination,
   ordering, money, images and fields from earlier implemented plans.
6. **Carry identifiers into customers.** Extend the response model with
   compatible defaults for older APIs. Preserve all strings and optional values
   through deserialization. Keep current card HTML, links, price formatting and
   pagination unchanged; no identifier display or filtering is requested.
7. **Document and deploy.** Update the shared library and affected service/app
   READMEs, API/Bruno examples, fixture notes and end-to-end walkthrough. Add an
   indexed ADR for the export/event extension and identifier update semantics,
   superseding only conflicting clauses of previous decisions. Deploy customers
   compatibility, customer-facing consumers, offers consumers/producers, then
   sourcing. Retain legacy readers and queued work; no backfill or queue reset.

## Acceptance and verification

- Conversion fixtures cover all five identifiers, leading-zero GTINs,
  alphanumeric SKUs, full GIDs, handles, partial/absent objects and multiple
  variants sharing a product GID but carrying distinct variant IDs/SKUs.
  Non-Shopify input does not acquire Shopify identifiers.
- Validation tests cover empty and NUL-bearing strings, wrong JSON types,
  external-object null, optional members and constructor-bypass attempts.
  Check exact string round trips without adding unrequested checksum rules.
- Export tests prove old version bodies/IDs remain unchanged, new fields map
  correctly, immutable retries preserve identifiers and unknown versions fail.
  Sourcing and offers consume the same new contract fixture.
- Aggregate/history tests cover creation, all-unknown state, enrichment,
  identifier-only correction, partial updates, legacy imports after enrichment,
  no-op repeats, mixed history and sequence overflow without partial changes.
  Verify identifier changes preserve offer identity and unrelated state.
- Isolated PostgreSQL tests cover migrations over old rows, atomic projection
  and receipt rollback, repeated original-message delivery, missing parents,
  and two offers with identical product identifiers. Verify both projections
  and APIs agree after queues finish.
- API/client tests cover REST and GraphQL names/nullability, full and partial
  identifier sets, old responses and unchanged card HTML. Retain smoke tests.
- Run `make fmt` then root `make ci`. Run `make audit` only if Cargo dependencies
  or lockfiles change and report new advisories. Run affected services/apps with
  `PORT=<port> make serve`, isolated databases and fixture inputs; check health
  endpoints and the customers page. Use explicit queue batches to demonstrate
  creation, an identifier-only update, replay and a legacy import that retains
  known identifiers. Check exact values in export, history, both projections,
  REST/GraphQL and the customers response model.

## Dependencies and completion

Keep this as a separate implementation change from
[scrape time](17-offer-scraped-at.md),
[sequence metadata](18-offers-event-sequence-metadata.md) and
[market/language](19-offer-market-and-language.md). Coordinate export and
creation-event versions and shared aggregate edits. When item 18 is present,
identifier events also carry the per-offer revision in metadata.

The current offers handler's Testimony-save/Mulac-publish failure window remains
a known limitation. Original-message receipts protect customer-facing against
duplicate delivery; distinct updates still apply in processing order. This
plan does not add source-order guarantees or atomic import/publication.

Record actual revisions, migrations, CI/database results and live propagation
evidence before marking TODO and the plan index complete.

## Implementation verification

Implemented alongside items 17–20. See the [verification record](17-20-verification.md)
for completed checks, prerequisite revision, audit comparison, and the remaining
database/live checks. TODO is marked done at the user’s request; the verification
record retains the outstanding checks.
