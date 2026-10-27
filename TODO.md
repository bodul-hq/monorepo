
GOAL: create full flow from structuralize (which is last point in `retailer-sourcing` app implement)

[x] Upgrade `retailer-sourcing` to be using local `mulac` and utilize new changes (examples can be seen on `offers` service) ([plan](docs/plans/01-retailer-sourcing-local-mulac.md))
[x] Start setting up `customer-facing` service by setting up `mulac's` queue runners and health endpoint ([plan](docs/plans/02-customer-facing-queue-runners.md))
[x] Add flow for converting data from `structurize` to `offer_export`: name it `processing for export` ([plan](docs/plans/03-processing-for-export.md))
[x] Add listener on `offer_export` event and write down message into `outbox` ([plan](docs/plans/04-offer-export-outbox-listener.md))
[x] For now, `outbox publisher` on `retailer-sourcing` should use `offers` api ({offers}/api/inbox) ([plan](docs/plans/05-retailer-sourcing-http-outbox.md))
[x] For now, `outbox publisher` on `offers` should use `customer-facing` api ({customer-facing}/api/inbox) ([plan](docs/plans/06-offers-http-outbox.md))
[x] Create `offers` entity and record on `customer-facing` that has same properites as `read_offers` in `offers` service ([plan](docs/plans/07-customer-facing-offer-projection.md))
[x] Add `offers` api (both rest and graphql) to retrieve limited number of offers with offset ([plan](docs/plans/08-customer-facing-offers-api.md))
[x] Set up `customers` app in a way that on homepage shows 30 offers, using nice design and built using only html (no javascript ATM) ([plan](docs/plans/09-customers-offers-homepage.md))
[x] Set up `dashboard` on `development` to include data from customer facing too ([plan](docs/plans/10-customer-facing-dashboard.md))
[x] offer should also support having a list of images besides title price url. Needs to be implemented on retailer_sourcing->offers->customer_facing->customer. in app customer, show first image in the view. initial images are included in OfferCreated; updates emit image-added events only for images not already saved ([plan](docs/plans/11-offer-images.md))
[x] Offers in `customer` should open new tab ([plan](docs/plans/12-customers-offers-new-tab.md))
[x] In `customer` put retailer code over the image with a semi-transparent grey background ([plan](docs/plans/13-customers-retailer-image-overlay.md))
[x] In `customer` align 'view offer' with price, let it sit in right corner .. ([plan](docs/plans/14-customers-price-offer-link-alignment.md))
[x] In `customer` show price in their format and use currency sign insted of text (beware that different currencies are formatted differenty between amount and sign) ([plan](docs/plans/15-customers-currency-formatting.md))
[x] In `customer` when showing titles, remove any html tags ([plan](docs/plans/16-customers-plain-text-titles.md))
[x] Add `scraped_at` to `OfferPageWasDownloaded` as Option<DateTime<Utc>> and forward it all down to customer_facing ([plan](docs/plans/17-offer-scraped-at.md))
[x] In `offers` store sequence as metadata of every event ([plan](docs/plans/18-offers-event-sequence-metadata.md))
[x] In extract process on `retailer_sourcing` extract what region is that offer meant for and what language is written in. Do not guess, data must come from source. Forward that information all the way to `customers` app ([plan](docs/plans/19-offer-market-and-language.md))
[x] We need identifiers all the way from structuralized to `customer` app. From `OfferImport` we should have gtin as Option<Gtin> and sku as Option<Sku> and external_identifiers as ExternalIdentifiers that have optional values for shopify_product_gid, shopify_product_variation_gid & shopify_handle ([plan](docs/plans/20-offer-identifiers.md))
[x] In `customer_facing` expose search endpoint where fuzzy search can be ran against offer title. It should support limit and offset only. `customers` app should show search field in the middle of header and point to /search page ([plan](docs/plans/21-offer-title-search.md))
[x] When `target_market` is missing, use a source locale’s supported country region to populate it (`it-IT` → country `IT`). Normalize `language` to its primary language for every new localization observation (`it-IT` → `it`). ([plan](docs/plans/22-locale-market-fallback.md))
[x] In `customer_facing`, replace the three Shopify identifier columns in `offers` with one JSON object containing three nullable identifiers. Preserve existing values during migration and keep GTIN/SKU separate. ([plan](docs/plans/23-customer-facing-shopify-identifiers-json.md))
[x] Detect the European Union regional market and English language from the `en-EU` source locale on `/eu-en/` pages ([decision](docs/adr/0033-european-union-locale-market.md); [verification](docs/plans/22-locale-market-fallback.md#completion-verification-2026-10-26))
[x] Lets include `was_price` when item is on sale/has discount and so on ([plan](docs/plans/24-offer-comparison-pricing.md))
[x] From extraction forward, include offer availability, item_condition ([plan](docs/plans/25-offer-availability-condition.md))
[x] Introduce content struct from structuralize on wards ([plan](docs/plans/26-offer-content.md))
[x] In `offers`, store all gtin,sku and other identifiers as new columns ([plan](docs/plans/27-identity-product-identifiers.md))
[x] Reorganize `content` in `offers`. Property in `offer` should be a list of Content structs that holds language, title and description. ([plan](docs/plans/28-localized-offer-content.md))
[x] When booting `retailer_sourcing`, start worker with x number of threads that will run commands ([plan](docs/plans/29-sourcing-command-workers.md))
[x] `DownloadOfferPage` handler should sleep for X seconds (define as const, set to 6 for now) ([plan](docs/plans/30-offer-download-delay.md))
[x] In `RetrieveSitemap` handler, between each sitemap to fetch, sleep X to Y seconds (set as const, put as 1-5 for now) ([plan](docs/plans/31-sitemap-fetch-delay.md))
[>] In `retailer_sourcing`, add a development endpoint that returns statistics for inbox, command, event and outbox entries scheduled for today (UTC), grouped by current status. In the `development` app, create a new dashboard that shows this data. ([plan](docs/plans/32-daily-queue-statistics-dashboard.md))



Status follows the [TODO workflow](AGENTS.md#todo-workflow):

| Marker | Phase |
| --- | --- |
| `[ ]` | Pending |
| `[.]` | Planning in progress |
| `[>]` | Planning complete |
| `[~]` | Implementation in progress |
| `[x]` | Implemented |

Flow: `[ ]` → `[.]` → `[>]` → `[~]` → `[x]`.

Items 24–27 are marked done at the user's request. Their implementation
and planned acceptance tests are present; database, object-storage and live
execution remain pending. The 2026-10-27
follow-up records passing portable checks and the environmental blockers in the
[verification record](docs/plans/24-27-offer-facts-verification.md#completion-follow-up-2026-10-27).

Items 28–31 are implemented. Their [verification record](docs/plans/28-31-verification.md)
lists passing portable checks and the PostgreSQL/live checks blocked by this
environment.

NOTES:
- We are aware that `For now` is going against ADR-16, but this is shortterm
- depending on local mulac will be needed in this phase and possibly next ones
- `customer-facing` will take given events and persist them into `offers` table



BACKLOG: leave this alone
- Refactor how offer duplication is handled: use gtin or sku per retailer?
- Refactor commanding by introducing priority column, first command starts with priority 100 and every after it does -1 on current priority. If we reach 10, we stop with -1 and use last one. That way we fast track first batch to finish point. When reserving commands, it should also order by priority asc
- Drop descripton column on `customer_facing`'s 'offer' table
