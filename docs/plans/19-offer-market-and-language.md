# Item 19: Source-backed offer market and language

Status: implemented and committed separately; final integration verification pending.

## Goal and confirmed scope

Extract the offer's target sales market and the language of its content from
the downloaded source during retailer_sourcing extraction. Preserve those
facts through normalization, structuralization, export, offers,
customer-facing and the customers app's data model.

The user confirmed that region means the target sales market, not shipping
destinations, and that customers should receive the data without displaying
it on cards yet. Unknown values remain unknown. Do not infer them from the
retailer code, hostname, URL path, currency, product title or a language model.
Translation, market filtering, shipping eligibility and UI localization are
outside this item.

## Current behavior and source evidence

Grounded at monorepo `d6686d7290f7ca8050564476a3242007d3bdfefc`.

- The shared [extraction facade](../../lib/retailer_data_extractor/src/lib.rs)
  runs storefront-specific architectures. The service and offline tools both
  use this facade, so new extraction must work in both paths.
- [Anker AU's raw document](../../lib/retailers_anker/src/ankerau/offer_details/v1/raw/nextdata.rs)
  types Next.js locale fields and alternate-language links. Those fields have
  different scopes and do not establish one universal locale mapping.
- [Ugreen CA's shop schema](../../lib/retailers_ugreen/src/ugreenca/offer_details/v1/raw/shop.rs)
  extracts Apple Pay configuration with `countryCode: "US"` and currency CAD.
  Payment configuration must not become the offer's target market.
- Saved page `../data/ugreenca/offer_details/sources/fr-products-15054a.html`
  contains `<html lang="fr">`, `Shopify.locale = "fr"` and
  `Shopify.country = "CA"`, alongside the US payment country and a PayPal
  `fr_FR` locale. Preserve this distinction in a checked-in regression fixture.
- Saved page `../data/ankerau/offer_details/sources/au-products-a110b-anker-prime-20k-220w-power-bank.html`
  declares `<html lang="en-AU">`, Next.js locale `au`, default locale `us`,
  and a Klarna widget locale `en-US`. Its document language is directly stated;
  neither the widget nor a guessed mapping from `au` proves its language.
- [Export conversion](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/conversion.rs)
  currently copies title, price, URL and images from variations. The
  [customers client](../../apps/customers/src/offers.rs) has no market or
  language fields.

The saved files are local survey evidence, not portable test dependencies.
Implementation must add minimal representative fixtures and survey all
implemented storefronts; these two pages do not establish rules for every shop.

## Evidence rules

1. Read declarations tied to the current downloaded offer/page. For language,
   prefer an explicit declaration for the offer content; otherwise use the
   current document's `html[lang]`. A validated storefront's active language
   setting can be a fallback when its meaning is demonstrated by fixtures.
   Preserve a complete language tag such as `en-AU`; do not reduce it to `en`
   or derive a target market from its region subtag.
2. For market, accept an explicit active storefront market or an offer-level
   target-market declaration. Parse supported Shopify country declarations
   from the saved script as data, without executing JavaScript. Additional
   retailer-specific paths need evidence that they describe the current
   market. A selector listing every possible market does not identify the
   selected one.
3. Exclude payment-provider country, merchant address, shipping destinations,
   browser preferences, unrelated widgets, default locale and alternate-page
   language lists. A field merely named `country` or `locale` is insufficient.
   Do not fetch another page to fill a missing value for the stored page.
4. Preserve raw candidates, selectors/JSON paths and selection/rejection
   reasons in extraction artifacts. Define precedence by scope: offer-content
   declarations can be more specific than page declarations; page declarations
   outrank unrelated defaults. Contradictory candidates of equal applicable
   scope produce unknown with conflict evidence, rather than an arbitrary winner.
5. Missing, unsupported or malformed optional declarations produce unknown
   fields without rejecting an otherwise valid offer. Preserve diagnostics for
   corpus review. This tolerance applies to extraction; malformed typed
   import/event payloads remain validation errors.

## Proposed data and update contract

Use a typed `localization` object throughout the downstream contract:

```json
{
  "localization": {
    "target_market": {"kind": "country", "code": "CA"},
    "language": "fr"
  }
}
```

Both fields are independently nullable. Represent a multi-country market as
`{"kind":"region","label":"Europe"}` only when the source explicitly names
that market. Preserve a bounded, validated source label; do not expand it into
an invented country list or turn unknown into “Worldwide”. Reuse the existing
[country type](../../lib/everyday/src/country_code.rs) where appropriate; it
deliberately excludes regions such as EU. Validate language tags with a suitable
existing or narrowly scoped parser after checking its supported format.
Normalization may adjust spelling/case but must not add missing information.

Keep evidence with source artifacts and immutable export lineage. Public data
contains selected facts, not raw scripts or extraction diagnostics. Associate
facts and evidence with the same document; never decorate an older export
using a newer extraction.

- Old imports without `localization` mean no observation. On creation they
  produce unknown values; on update they preserve existing values.
- New extraction/export versions include the object. Null fields mean the
  source could not establish that fact, including conflicts. They replace the
  previous value so it is not presented as established by the new source.
  A null object is invalid; null fields inside the object are valid.
- New `OfferCreated` payloads carry the object, with compatibility for older
  creation payloads. Emit `OfferLocalizationChanged` with
  `{"id":"<offer UUID>","localization":{...}}` when selected values change,
  including known-to-unknown transitions and localization-only changes.
  Evidence-only changes and identical observations produce no domain event.
- Use a new stored creation version and `OfferLocalizationChanged.v1` while
  retaining historical readers. The new event replaces both fields as one
  observation. Preserve unrelated title, price, image and scrape-time behavior.

## Implementation steps

1. **Survey and capture evidence.** Follow the
   [retailer integration playbook](../retailer-integration.md) for affected
   storefronts. Build a coverage table of explicit sources, selectors/paths,
   conflicts and unknown cases. Survey each implemented storefront with saved
   pages; record missing corpora and unsupported shapes without guessed
   defaults. Add portable fixtures for supported source families and the
   conflicting payment/widget examples.
2. **Extend extraction and intermediate contracts.** Capture declarations
   through the shared facade and storefront architectures; avoid claiming an
   entire HTML element before nested islands have been extracted. Raw schemas
   use `deny_unknown_fields`, so update them deliberately with optional fields
   accepting old extractions. Normalize selected facts and evidence into shared
   typed values and carry them through every implemented structuralizer into
   relevant variations. Inherit page-level facts only for variations represented
   by that page. Keep service/offline behavior identical. Old artifacts remain
   readable without consulting unrelated current HTML.
3. **Version exports.** Add the object to a new immutable export schema and
   import payload, updating conversion, validation, decoders, listeners and
   fixtures. Preserve v1/v2 bytes, payload shapes and message IDs on retry.
   Item 17 proposes v3: reserve this format as v4 and support whichever earlier
   versions actually shipped. A skipped number is acceptable; never reuse v3
   with another meaning. New commands select the new version; queued commands
   retain their original version/default. Old artifacts without evidence yield
   unknowns. Preserve evidence in snapshot lineage without leaking it into the
   public payload or changing the payload of an existing message ID.
4. **Extend offers.** Update the import command and temporary REST DTO,
   validation, aggregate state, replay, stored record conversion and active
   handler. Reserve capacity for the complete event batch before mutation.
   Add nullable target-market JSONB and language text columns to `read_offers`,
   with typed validation and consistent in-memory behavior. Register the new
   event with the projector, customer-facing subscriber and publisher.
   Preserve raw payloads and delivery identities.
5. **Extend customer-facing.** Add equivalent nullable columns to `offers` and
   register the event. Apply the observation and original-message receipt in
   one transaction. Duplicate delivery is a no-op; updates before creation
   retry without recording a receipt. Return localization in REST and GraphQL,
   including null fields for old rows. Define an explicit country/region
   discriminator in both schemas. Preserve pagination, ordering and other data.
6. **Carry data into customers.** Extend its server-side response model with
   the typed object and an unknown default when an older API omits it. Test
   known, partially known and unknown values. Keep card output, page language,
   price formatting and pagination unchanged; do not add badges or filters.
7. **Document and deploy.** Add an indexed ADR for evidence selection, unknown
   semantics, export/event versions and the additional event. Preserve accepted
   ADRs, superseding only conflicting clauses. Update affected READMEs,
   API/Bruno examples and the walkthrough. Deploy customers compatibility,
   customer-facing readers, offers readers/producers, then sourcing. Retain
   legacy readers and queued payloads; no automatic backfill or queue reset.

## Acceptance and verification

- Portable fixtures prove that the Ugreen example yields CA/fr, independent
  of US payment metadata and the PayPal locale. Anker preserves `en-AU`; its
  market remains unknown unless separate valid active-market evidence is
  demonstrated. Removing accepted declarations yields unknown even when the
  retailer code, URL and currency suggest an answer.
- Cover country and regional markets, full language tags, missing/blank/invalid
  declarations, conflicting applicable declarations, defaults, alternate links,
  multilingual pages and malicious strings. Never execute source scripts.
  Distinguish current-page language from other languages a site offers.
- Run extraction, normalization and structuralization fixtures for every
  supported source family, plus saved-corpus checks for affected storefronts.
  Verify facade/offline parity and report skipped/missing coverage accurately.
- Export tests cover new observations, old artifacts, v1/v2 and any shipped v3
  retries, immutable source association, version mismatches and stable IDs.
  Add a shared payload fixture consumed by offers.
- Aggregate/projection tests cover omission, initial values, either-field
  changes, explicit unknowns, localization-only changes, identical imports,
  mixed history and capacity failure. Isolated PostgreSQL tests cover additive
  migrations, old rows, rollback, original-message replay and missing parents.
- API/client tests verify identical selected facts in REST, GraphQL and
  customers, old-response compatibility, malformed-payload rejection and
  unchanged card HTML. Keep all existing smoke tests passing.
- Run `make fmt` then root `make ci`; audit only when Cargo dependencies or
  lockfiles change. Run affected services/apps using `PORT=<port> make serve`
  with isolated databases and fixtures; check health endpoints and the customers
  page. Explicitly run queue batches and verify a known observation, a
  localization-only change, an unknown observation and a legacy replay all
  reach the expected state without changing card presentation.

## Dependencies and completion

Implement separately from [scrape time](17-offer-scraped-at.md) and
[event sequence metadata](18-offers-event-sequence-metadata.md). Coordinate
export and creation-event version allocation before implementation; if either
has shipped, extend its final contract instead of replacing it. When item 18
is present, localization events carry the same per-offer revision.

The current offers handler's Testimony-to-Mulac publication failure window and
processing-order updates remain existing limits. Original-message receipts
prevent repeat delivery from applying twice, but distinct older observations
can arrive later. Record this behavior rather than claiming sequence metadata
alone provides source-order protection.

Mark complete only after recording implemented source coverage, unknown/conflict
examples, actual CI/database results and end-to-end evidence.

## Implementation verification

Implemented alongside items 17–20. See the [verification record](17-20-verification.md)
for completed checks, prerequisite revision, audit comparison, and the remaining
database/live checks. TODO is marked done at the user’s request; the verification
record retains the outstanding checks.
