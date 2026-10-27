# ADR 35: Source availability and item condition

Status: Accepted; extends ADRs 19, 21 and 22.

Date: 2026-10-26

## Context

Retailers expose different purchase, inventory and condition evidence, and some
pages expose none. A purchasable Shopify variant does not prove it is in stock.

## Decision

Carry optional availability and item condition per variant. Purchase booleans
map to `Available` / `Unavailable`; explicit schema stock states map to
`InStock`, `OutOfStock` or `SoldOut`. Preserve supported raw unknown states as
`Other`. Condition uses explicit source `New` or `Other`, never a default new
condition. Keep missing or conflicting evidence unknown and do not copy one
variant's evidence to siblings. Parse exact HTTP and HTTPS schema.org URLs on
new processing; deserializing historical Other values does not reinterpret them.

Export v7 and `OfferCreated.v7` add both fields. Omitted fields are legacy no-op
observations; explicit null clears. New `OfferAvailabilityChanged` and
`OfferItemConditionChanged` events require their respective keys, including
when null, and use `.v1` event records. Both projections store nullable JSONB.
REST and GraphQL expose nullable `{kind, raw}` objects. Customers carry these
facts without adding availability or condition labels to cards.

## Consequences

Coverage is evidence-dependent, not a promise that every retailer supplies a
value. The common shape is available for every implemented structuralizer.
Older exports and event histories remain readable. Database migrations are
additive; population requires new observations.
