# retailers_anker

Integration for the Anker retailer.

The former Nordic integration now has five active market codes: `ankerse`
(Sweden), `ankerno` (Norway), `ankerfi` (Finland), `ankeris` (Iceland), and
`ankerdk` (Denmark). They share the Nordic sitemap and product URLs, but fetch
country-specific Storefront API prices in SEK, NOK, EUR, ISK and DKK.
The Swedish market here is the existing Nordic store, not `anker.com/se`.
`ankernordicscom` remains available for historical records and queued work.

For these five codes, `retailer_data_extractor::extract_for` reads the downloaded
API JSON directly. Sourcing uses `offer_request_for` to build the country query;
configure `ANKER_NORDICS_STOREFRONT_TOKEN` with the site's public browser
Storefront token. Raw offline exports can retain `.html` filenames but contain
JSON. HTML alone is rejected because it carries Swedish-default prices.
Market, currency, handle, GraphQL errors and incomplete responses are checked
before storage. Products needing more than 250 variants/images fail explicitly.
See [ADR 47](../../docs/adr/0047-nordic-retailer-markets.md) for compatibility
and rollout. Each Nordic retailer owns its legacy query, raw types, parser and captured
response in `offer_details/v1/graphql`. `nordic` only routes compatibility calls
and retains the existing normalized document format.

Each storefront has an offer details pipeline in `offer_details/v1`:

- `architecture()` extracts the page's data islands: the `ld+json` scripts
  plus, depending on the storefront, the Next.js `__NEXT_DATA__` script, the
  App Router flight payload, or the Liquid `ProductJson` record.
- `normalize()` reads them into a strict raw schema, which rejects unknown
  fields, and then into a normalized document.
- `structuralize()` turns the normalized document into the structuralized one.

`ankerkr` runs on Cafe24, has no data island to read, and produces no document
yet.

The AU and US normalizers skip product pages whose URLs identify another known
Anker storefront; older crawls placed regional pages in these folders. Missing
payloads on native pages still fail. German products can omit their featured
image, and EU/Polish specification rows can have text with no separate label
(`name: null`). Japanese PowerBag records without Product JSON-LD retain their
Liquid variants in JPY and use their storefront handle for offer URLs. Italian
JSON-LD offer prices retain their JSON number representation so decimal prices
also deserialize when object-storage clients enable `arbitrary_precision`.
Canadian and German structuralized variations retain Shopify identifiers when
their merchant SKU is blank, with `sku: null` instead of rejecting the page.

The `ankerau` tests run the pipeline over saved product pages in
`../data/ankerau`, next to the monorepo checkout, and skip when that folder is
missing. They select `au-products-*.html`, matching the URL-path filenames
written by the export tool, because older crawls also saved other regions in
that folder. The other storefronts are covered by unit tests. The facade's
`normalization_report` integration test also checks saved extracted pages from
the storefronts with previously reported normalization failures.

`ankerau` is the reference implementation in the
[retailer integration playbook](../../docs/retailer-integration.md), which
describes how to add or finish a storefront.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

To exercise the Italian regression under the development app's JSON features:

```sh
cargo test -p retailers_anker ankeritalycom --features serde_json/arbitrary_precision
```

The Italian corpus test reads `../data/ankeritalycom/offer_details/extracted`,
checks normalization and structuralization without writing files, and skips when
that data checkout is absent.

Canonical comparison prices, availability, item condition, and descriptions are
documented in [offer facts source coverage](../../docs/offer-facts-source-coverage.md).

## Anker initial fill

The versioned AU/CA integrations and shared `shopify::initial_fill::markets`
configuration support all 17 active Anker stores except Korea. Shared typed
queries take country/language variables for every operation and continuation.
Canada remains English-only. Germany, France, Poland, Italy, Japan and Vietnam
use DE, FR, PL, IT, JA and VI respectively; the other storefronts use EN.
Nordic markets keep separate prices and discover their browser token automatically
for initial fill; `ANKER_NORDICS_STOREFRONT_TOKEN` remains an optional override
(and is still required by the separate sitemap offer-download flow). See
[ADR 54](../../docs/adr/0054-anker-browser-token-discovery.md).
Italy/Japan attempt optional public token acquisition and retain token-free access
when no token is found; other stores
acquire the selected browser token from their configured source. The full field
selection and exhaustive pagination are preserved for both access modes.
All retailers use initial pages of 5 for every selected connection. Continuations
fetch 25 collections/images/media, 10 variants, 20 selling-plan groups, and 5
items per nested variant connection or selling-plan list. Authentication does
not change these sizes; see
[ADR 56](../../docs/adr/0056-anker-operation-specific-page-sizes.md).
See the [market table](../../services/retailer_sourcing/README.md#anker-initial-offer-fill),
[ADR 53](../../docs/adr/0053-anker-storefront-initial-fill-markets.md), and
[captured evidence](src/shopify/initial_fill/fixtures/README.md).

The checked-in schema and full details selection come from
`wip/switching-shopify-retailers-to-api` (`c3f5872`). Catalog requests select only
product GIDs and cursors. Separate typed continuation operations cover product,
variant, and selling-plan connections. The saved WIP response in the fixtures
has incomplete collections and is deliberately retained as a pagination regression.
Presentation selections additionally request `id` to avoid the generated client's
empty enum for objects selecting only `__typename`.
`make ci` compiles the query documents and tests response validation; no live
Shopify request is made by the library tests.

## Offer details v2 raw representations

AU and CA expose `AnkerAuRawProduct` / `AnkerCaRawProduct`,
`AnkerAuRawResponse` / `AnkerCaRawResponse`, and
`AnkerAuRawOfferDetails` / `AnkerCaRawOfferDetails` from `prelude`.
These deserialize a merged product, a GraphQL response (single product or the
older full-products connection), and a stored initial-fill artifact respectively:

```rust
use retailers_anker::prelude::AnkerAuRawOfferDetails;
let raw: AnkerAuRawOfferDetails = serde_json::from_str(&saved_json)?;
let title = &raw.product.title;
let variants = &raw.product.variants.edges;
```

All 17 configured Anker retailers own concrete types in their own
`offer_details/v2/raw.rs`, exposed as `<retailer>_raw_v2` in the prelude. Every product
field is typed, including connections, images/media, variants, prices, quantity
rules, selling plans and localization. Unknown fields fail rather than being
silently dropped. Decimal amounts remain strings; other numeric scalars retain
JSON numbers. Diagnostic metadata and recorded continuation response pages stay
JSON evidence. Artifacts use schema 1 without context and schema 2 with context; v2 names the pipeline.
Extracted representations and existing extraction behavior are unchanged.

Each retailer's fixtures preserve its own captured product data. Run
`cargo test -p retailers_anker offer_details::v2` in `lib` for fixtures, or
`cargo test -p retailers_anker saved_graphql_corpus -- --ignored` for the external
`../data3` corpus. Nordic fixtures use the corresponding country context.
See [ADR 61](../../docs/adr/0061-independent-retailer-graphql-integrations.md)
for ownership and [ADR 57](../../docs/adr/0057-offer-details-v2-raw-sources.md)
for the dump contract.

Regional US/EU/FR/NZ/PL/VN raw models also retain inventory, metafields and
expanded variant-reference selections from their saved full-catalog captures.
These optional source fields are local to those retailers; the initial-fill
query documents and extracted representations are unchanged.
