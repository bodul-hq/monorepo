# retailer_sauce

Retailer integrations.

- `shopify`: classifies Shopify storefront URLs as product, catalog or
  content pages by their path.
- `availability`: explicit stock states plus distinct purchase availability;
  HTTP/HTTPS schema.org constructors preserve unrecognized source strings.
- `identifier`: the validated identifier types a structuralizer builds
  (`Gtin`, `Sku`, `ShopifyProductGid`, `ShopifyProductVariationGid`,
  `ShopifyHandle`) and their `IdentifierError`.
- `json`: strict numeric deserialization for fields inside tagged or untagged
  enums, including when serde_json enables `arbitrary_precision`.
- `item_condition`: `ItemCondition`, read from a schema.org condition URL.
- `structuralized`: `StructuralizedData`, what a retailer's structuralizer
  returns, the shared `Offer` scaffold, and `content::Content` for an exact
  optional description. Present content requires a JSON object and a description
  key; `{ "description": null }` explicitly clears it.
- `observation`: distinguishes a missing legacy field from a present nullable
  observation, with strict object decoders for pricing and content boundaries.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

`localization` defines validated target-market and language observations.
Its `source_locale` helper parses a complete ICU Unicode BCP47 locale for new
extractions, exposing canonical identity, primary language and an optional
supported country region. It does not maximize likely subtags or use extension
regions. `Localization` deserialization still preserves historical full tags;
interpretation is opt-in at extraction time. See [ADR 31](../../docs/adr/0031-source-locale-market-fallback.md).
`identifier` also provides `ExternalIdentifiers` and a validated complete identifier
set for exact-string propagation and enrichment. Existing identifier constructors
retain their semantics; new propagation boundaries additionally reject embedded NUL.

`offer_content::Content` is the localized offers/read-model contract, separate
from source `structuralized::content::Content`. Each entry requires nullable
`language`, `title`, and nullable `description` fields. Shared guards require
nonempty lists with unique exact language keys and an existing selected key.
The module sorts unknown language first, then exact tags lexically; it never
normalizes historical text or language tags. Import upserts preserve a missing
description only in the destination bucket. Legacy title/description updates
change the selected bucket; legacy language corrections move it, replacing any
destination collision without copying a translation.

Regional markets use `RegionCode` (`eu` for the European Union). Historical
label-based regions decode to `LegacyRegion` to preserve immutable encoding;
`canonicalized()` maps only the exact European Union label. Country codes remain
separate, and unknown regional codes are rejected. See ADR 40.
