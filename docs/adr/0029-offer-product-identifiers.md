# 29. Offer product identifier propagation

## Status

Accepted. Extends ADRs 19, 25, 26 and 28's export/creation contracts.

## Date

2026-10-26

## Context

Structuralized variations already contain product identifiers, but export and
customer-facing projections discarded them. They must remain associated with the
correct variation without changing the retailer/URL offer identity.

## Decision

Reuse `Gtin`, `Sku`, `ShopifyProductGid`, `ShopifyProductVariationGid` and
`ShopifyHandle`; add `ExternalIdentifiers` with three optional Shopify fields.
Validate constructor semantics and reject NUL at the new boundaries. Preserve
case, whitespace, leading zeros and complete strings. No new checksum/GID-format
rules or extraction heuristics are imposed.

Export v5 carries optional GTIN/SKU and a present external-identifiers object.
Earlier schemas omit these fields, preserving immutable payloads and delivery IDs.
Creation history advances to v5 while retaining v1–v4 readers. Incoming non-null
values replace individual known fields; omitted/null values retain them. Emit one
`OfferIdentifiersChanged.v1` containing the complete resulting set after other
updates. History replay and projections apply this set without merging again.

Both projections add five nullable text columns, with no product-identifier
uniqueness constraints. REST/GraphQL return GTIN/SKU strings and a non-null
external-identifiers object. Customers deserializes and validates these fields
without displaying them or changing card behavior.

## Consequences

Different offers may share identifiers. Clearing, matching and deduplication are
outside this contract. Deploy clients and downstream consumers before producers.
No backfill or queue reset is needed. Existing publication and processing-order
limitations remain unchanged.
