# retailer_sauce

Retailer integrations.

- `shopify`: classifies Shopify storefront URLs as product, catalog or
  content pages by their path.
- `availability`: `Availability`, read from a schema.org stock-state URL.
- `identifier`: the validated identifier types a structuralizer builds
  (`Gtin`, `Sku`, `ShopifyProductGid`, `ShopifyProductVariationGid`,
  `ShopifyHandle`) and their `IdentifierError`.
- `json`: strict numeric deserialization for fields inside tagged or untagged
  enums, including when serde_json enables `arbitrary_precision`.
- `item_condition`: `ItemCondition`, read from a schema.org condition URL.
- `structuralized`: `StructuralizedData`, what a retailer's structuralizer
  returns, and the shared `Offer` shape.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

`localization` defines validated target-market and language observations.
`identifier` also provides `ExternalIdentifiers` and a validated complete identifier
set for exact-string propagation and enrichment. Existing identifier constructors
retain their semantics; new propagation boundaries additionally reject embedded NUL.
