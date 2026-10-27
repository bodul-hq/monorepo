# ADR 37: Product identifiers in the offer identity projection

Status: Accepted; extends ADR 29.

Date: 2026-10-26

## Context

The `read_offers_identifiers` Rust module uses SQL table `read_offer_identities`
to locate offers by retailer and URL. Product identifiers previously lived only
in the separate read-offer projection.

## Decision

Add nullable TEXT columns `gtin`, `sku`, `shopify_product_gid`,
`shopify_product_variation_gid` and `shopify_handle` to `read_offer_identities`.
Preserve exact values and existing copies in `read_offers`. Keep retailer/URL
identity lookup, uniqueness and matching rules unchanged.

Backfill only rows whose ID, retailer and URL match the existing projection.
Abort on contradictory identity or invalid source values, verify the copied
values, and report unmatched rows; unmatched identity rows remain null. Apply
existing creation and full `OfferIdentifiersChanged` events to all five columns
atomically. A missing update target fails visibly instead of creating a partial
identity row.

## Consequences

No new export or event version is needed. This does not introduce GTIN/SKU
matching, deduplication or merging. Migration rollback removes only the added
columns; the original projection retains its values.
