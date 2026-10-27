# ADR 34: Source comparison pricing

Status: Accepted; extends ADRs 19, 21, 22 and 23.

Date: 2026-10-26

## Context

Retailer variants can declare a comparison price alongside the payable price.
The pipeline previously discarded it. Historical payable prices cannot establish
that a retailer currently advertises a discount.

## Decision

Carry optional `was_price: Money` from each structuralized variant. Accept only
source-declared, same-variant prices in the payable currency and strictly above
the current price. Preserve the existing current-price selection. Do not infer
comparison prices from scrape history, coupons, savings percentages or other
variants.

Export v6 adds this observation; readers retain v1–v5 support. Omission means no
observation; explicit null clears the value. A legacy import changing current
price retains a compatible comparison price, and clears an incompatible one.
Use `OfferPricingChanged` with required `pricing: {price, was_price}` to update
both atomically. Store it as `OfferPricingChanged.v1`. Legacy
`OfferPriceChanged.v1` and its `OfferPriceChangedEvent` routing key remain
price-only on replay. Creation adds the field in `OfferCreated.v6`; later
creation versions include it too.

Both projections use paired nullable `was_price_amount` and
`was_price_currency` columns. Customer-facing suppresses incompatible stored
comparison prices in responses without rewriting history. Customers use the
existing currency formatter and show valid comparison prices in a smaller
`del` element with accessible wording.

## Consequences

Retry payloads and historical events keep their existing meanings. Delivery
receipts still protect customer-facing updates against duplicate messages.
Readers and migrations must be deployed before upstream writers. Existing
snapshots gain no inferred facts; fresh observations populate the fields.
