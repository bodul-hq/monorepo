# ADR 52: English-only Anker Canada initial fill

- Status: Accepted
- Date: 2026-11-09
- Supersedes: ADR 50's AU-only retailer scope and country-specific query ownership.
  ADR 51's raw-offer storage bucket decision remains in force.

## Context

The Canadian initial fill needs the existing token → catalog identifiers →
complete offer details flow. Only English is requested; French will follow in a
later phase. The captured Anker multi-market script (`storefront_token/assets/16.js`)
identifies `storeDomains.ca` as `anker-canada.myshopify.com` and includes
`storeTokens.ca`. The Canadian storefront is `https://www.anker.com/ca`.

## Decision

Add versioned `ankerca::initial_fill` configuration for `/ca`, the `ca` token key,
Storefront API 2026-10, country CA, language EN and currency CAD. Enable `ankerca`
on the existing API. Keep tokens acquired at runtime, with no production token
constant. Reject acquisition commands pointing at `/ca-fr`.

Share the generated queries and schema under `retailers_anker::shopify::initial_fill`.
Use a typed country variable and a fixed `language: EN` directive in every catalog,
detail and continuation operation. AU continues to use AU/EN. Validate reported
response context against the configured country/language, and validate all
assembled product money against the configured currency before storage.

Preserve 500-ID batches, full catalog and nested connection pagination, keyed
fan-out, six-second pacing, routing, retries, storage and the terminal
`OfferDetailsFetched` event without downstream listeners. No language option,
French request, new payload field, database migration or dependency is added.

## Consequences

Canadian runs retrieve English data with CAD prices. Recorded artifact requests
include the country variable and explicit EN directive. Raw response evidence
is retained. Existing immutable AU artifacts remain readable.

Offline tests cover the captured Canadian token entry, synthetic HTML/script
acquisition, 500-ID fan-out, CAD storage, every continuation and French context
rejection. The existing PostgreSQL flow is exercised for both retailers and
opt-in live tests read one complete offer from each. A captured AU script is
configuration evidence, not proof of a successful live Canadian API response.
