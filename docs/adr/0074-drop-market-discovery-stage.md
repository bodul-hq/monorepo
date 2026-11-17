# 74. Drop the market-discovery stage

Status: Accepted

Supersedes the DiscoverStorefrontMarkets / StorefrontMarketsDiscovered stage kept
by ADR 69. ADR 69's other decisions are unchanged: requested contexts still come
from `Retailer.supported_locales`, use an optional country and the selected language,
and are deduplicated in declaration order.

## Context

Every connected integration set `all_markets = true`, so the flag never selected a
different flow. Since ADR 69 the discovery stage made no HTTP request. It only loaded
the entity's locales and fanned out catalog commands. The command, event, process
step and flag added branching without choosing anything.

## Decision

Remove the `all_markets` integration field, `retailer_data_extractor::all_market_contexts`,
`Step::DiscoverMarkets` and the discovery command, event and handlers. The initial-fill
process is now `[AcquireStorefrontToken] → ProductCatalogIdentifiers → OfferDetails`,
with the token step only when the storefront attempts token acquisition.

The catalog slice fans out one `FetchProductCatalogIdentifiers` per entity context:
- after `StorefrontTokenAcquired`, carrying the acquired or absent token;
- directly from the start API for storefronts that do not attempt token acquisition.
  Each first command is dispatched under its own stable ID.

Command identities are unchanged: the first catalog page keeps the key
`["FetchProductCatalogIdentifiers", country, language, null]` that discovery produced.
The start API resolves the contexts before queueing anything, so missing inventory,
empty locales or regional countries reject the request with `400`.

`retailer_sauce 0.1.1`'s published `InitialFillIntegration` trait still requires
`all_market_contexts()` and `discover_markets()`. Brand engines keep them as stubs:
the first returns `true`, and the second returns an error without making a request.
Nothing in this repository calls them. A future `retailer_sauce` release can remove them.

## Consequences

Contexts are read from the entity when the token event is handled, not snapshotted
in a separate event. Replaying a token event after an inventory change therefore uses
the current inventory.

## Rollout

Drain in-flight initial fills before deploying. Queued `DiscoverStorefrontMarkets`
commands and unprocessed `StorefrontMarketsDiscovered` events no longer have a
handler. No database migration or start-endpoint change is required.
