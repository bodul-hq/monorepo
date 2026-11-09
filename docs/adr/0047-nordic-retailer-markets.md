# ADR 47: Separate Nordic Anker markets

- Status: Accepted
- Date: 2026-11-09

## Context

`AnkerNordicsCom` combines five countries. The store shares a sitemap and
product URLs, while its HTML returns Swedish-default prices even with another
country's cookie. The public Storefront API returns separate market prices.

## Decision

Use `AnkerSe`, `AnkerNo`, `AnkerFi`, `AnkerIs` and `AnkerDk` for new work.
These are markets of the existing Nordic store; Sweden does not switch to the
separate `anker.com/se` catalog. All five use the Nordic sitemap with exact
origin and path guards, without invented country prefixes.

Offer downloads POST the checked-in product query to the public
`ankerneu.myshopify.com` Storefront API with explicit country and English
language context. Configure `ANKER_NORDICS_STOREFRONT_TOKEN` with the public
browser application's Storefront token: anonymous API access omits headless
catalog products. Missing configuration fails Nordic downloads explicitly.
The token is sent only to the fixed API endpoint, without redirects. Request
construction stays in the retailer library; sourcing owns HTTP, proxy
rotation, pacing, retries and storage. Other retailers continue using GET.

Before storage, verify the returned market, language, currencies and handle.
Reject GraphQL errors, missing products, duplicate variants and incomplete
connections. The query requests up to 250 images and variants and fails
explicitly if pagination is required. Never substitute default-market prices.
Store responses verbatim, and extract JSON directly for these five codes.
Offline raw exports retain their existing `.html` filenames but contain JSON.

Normalize decimal prices exactly into minor units and preserve the API's
localization through structuralization. NOK/DKK use two fractional digits;
ISK uses zero. Map the source checkout URL to the verified headless store
origin. Shopper URLs remain shared between countries; market identity resides
in the retailer code and localization rather than an unverified URL parameter.

Retain the old enum spelling, slug and integration for historical records and
queued work, but remove it from active processing lists. Do not rewrite events,
exports or retailer/URL identities, or assign ambiguous old data to a country.

## Rollout and validation

Publish and consume the updated shared-code crate. Deploy compatible readers
(offers, customer-facing and apps, including their money types) before new
sourcing producers. Then start country crawls. Inventory old Nordic queues and
offers before any separately requested data migration; this change runs none.

Portable fixtures contain a real product queried separately in all five
markets. Tests cover prices, localization, market isolation, source identity,
API failures, facade wiring, download handling and customer display.
Historical HTML corpus coverage remains labeled as historical Nordic coverage.

Evidence: the site's country selector and public `@inContext` calls, the saved
`data3/ankernordicscom` localization survey, and
[Shopify's international-pricing contract](https://shopify.dev/docs/storefronts/headless/building-with-the-storefront-api/markets/international-pricing).
