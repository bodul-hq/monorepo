Shipping data: `Shipping countries` retains the existing documented coverage.
The separate Shopify shipping-configuration survey remains in the
[research evidence](research/retailer-shipping-countries-2026-11-13.json), rather
than in the retailer entity or inventory tables. Configured countries do not
guarantee checkout eligibility or delivery for every product/address.
The Anker EU shipping row was previously reconciled with this configuration and
retains its explicit policy-conflict note. Dated checks supersede older claims of
unknown Ugreen UK/US endpoints; older research prose remains historical evidence.

Main locale tables contain existing storefront locales and explicitly verified
contexts. Advertised-only tables identify unconfirmed entries through their header;
they are shown only when nonempty. Main does not imply a selected collection
country or verified pricing. Both tables retain their evidence annotations.

Verification update (2026-11-13): [per-locale evidence and results](research/retailer-context-verification-2026-11-13.json).
Eight previously unselected rows have matching page language, Shopify backend,
page country and explicit Storefront API context responses. They are marked
`context verified 2026-11-13`. This verifies localization, not shipping eligibility,
price uniqueness, complete translation, or collection priority. Other rows remain
unselected: blocked requests are not evidence that a locale is unsupported.
Earlier research notes below describe the original survey; the dated evidence
supersedes them only for rows explicitly verified here. Runtime initial-fill collection uses only the main-table locales through
`StorefrontConfig::supported_locales()`; advertised-only entries remain research. A main-table
locale without a country is collected with country omitted, using its declared
GraphQL language. Currency is observed in responses. See
[ADR 69](adr/0069-inventory-initial-fill-contexts.md).

## Anker Australia

Implementation: [`retailers_anker::prelude::get_storefront_config("ankerau")`](../lib/retailers_anker/src/ankerau/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                      |
| ---------------------- | ---------------------------------------------------------- |
| Retailer code          | `ankerau`                                                  |
| Storefront website     | https://www.anker.com/au                                   |
| Shopify shop value     | oneanker-au.myshopify.com                                  |
| Storefront GraphQL API | https://oneanker-au.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Australia (AU)                                             |
| Shipping countries     | AU                                                         |
|                        |                                                            |
|                        |                                                            |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                      | GraphQL country/region | GraphQL language |
| ----------------- | ------------------------ | ---------------------- | ---------------- |
| en-AU             | https://www.anker.com/au | AU                     | EN               |

## Anker Canada

Implementation: [`retailers_anker::prelude::get_storefront_config("ankerca")`](../lib/retailers_anker/src/ankerca/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                       |
| ---------------------- | ----------------------------------------------------------- |
| Retailer code          | `ankerca`                                                   |
| Storefront website     | https://www.anker.com/ca                                    |
| Shopify shop value     | anker-canada.myshopify.com                                  |
| Storefront GraphQL API | https://anker-canada.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Canada (CA)                                                 |
| Shipping countries     | CA                                                          |
|                        |                                                             |
|                        |                                                             |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                         | GraphQL country/region | GraphQL language |
| ----------------- | --------------------------- | ---------------------- | ---------------- |
| en-CA             | https://www.anker.com/ca    | CA                     | EN               |
| fr-CA             | https://www.anker.com/ca-fr | CA                     | FR               |

## Anker Germany

Implementation: [`retailers_anker::prelude::get_storefront_config("ankerde")`](../lib/retailers_anker/src/ankerde/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                      |
| ---------------------- | ---------------------------------------------------------- |
| Retailer code          | `ankerde`                                                  |
| Storefront website     | https://www.anker.com/de                                   |
| Shopify shop value     | oneanker-de.myshopify.com                                  |
| Storefront GraphQL API | https://oneanker-de.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Germany (DE)                                               |
| Shipping countries     | DE                                                         |
|                        |                                                            |
|                        |                                                            |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales                 | URL                      | GraphQL country/region | GraphQL language |
| --------------------------------- | ------------------------ | ---------------------- | ---------------- |
| de (HTML), de-DE (alternate link) | https://www.anker.com/de | DE                     | DE               |

Note: https://www.anker.com/eu-de redirects to https://www.anker.com/de and uses the same Shopify shop, oneanker-de.myshopify.com. It is an alias of `ankerde`, not a separate retailer.

GraphQL context (verified live on 2026-11-09): `@inContext(country: DE, language: DE)`. The API rejects `country: EU` as an invalid `CountryCode`.

## Anker Europe

Implementation: [`retailers_anker::prelude::get_storefront_config("ankereu")`](../lib/retailers_anker/src/ankereu/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                                                          |
| ---------------------- | ---------------------------------------------------------------------------------------------- |
| Retailer code          | `ankereu`                                                                                      |
| Storefront website     | https://www.anker.com/eu-en                                                                    |
| Shopify shop value     | ankereu.myshopify.com                                                                          |
| Storefront GraphQL API | https://ankereu.myshopify.com/api/2026-10/graphql.json                                         |
| Target markets         | Europe (EU)                                                                                    |
| Shipping countries     | AT, BE, BG, CZ, DE, DK, EE, ES, FI, FR, GR, HR, HU, IT, LT, LU, LV, NL, PL, PT, RO, SE, SI, SK |
|                        |                                                                                                |
|                        |                                                                                                |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                         | GraphQL country/region | GraphQL language |
| ----------------- | --------------------------- | ---------------------- | ---------------- |
| en-EU             | https://www.anker.com/eu-en | AT                     | EN               |

**Pricing decision:** Use Austria's country code (`AT`) to retrieve all Anker
Europe catalog prices from `ankereu.myshopify.com`, with
`@inContext(country: AT, language: EN)`. Use these returned EUR prices for the
Anker Europe price feed.

Croatia has observed price differences, but we intentionally disregard them
and will not collect a separate Croatian price feed. Poland returns prices in
Polish złoty (`PLN`); those prices will be handled separately, outside this
Austria-based EUR feed. This is our chosen collection policy, not a claim that
every country's actual prices are identical.

The GraphQL country/region and GraphQL language columns specify the intended collection context; Supported locales retains the website locale tags. Here, `AT` / `EN` is the selected pricing context for the `en-EU` storefront. Contexts elsewhere in this inventory are not claimed to be live-verified unless explicitly noted.


**Shipping verification (2026-11-13):** The storefront's embedded
`props.pageProps.shop.shipsToCountries` and a direct Storefront API
`shop { shipsToCountries }` query both return the 24 destinations above.
[Saved response](research/ankereu-shipping-countries-2026-11-13.json).
The [English shipping policy](https://www.anker.com/eu-en/policies/shipping-policy)
still lists only eight countries; it conflicts with the structured shipping
configuration. The list above uses that configuration rather than expanding
localization markets or interpreting product exclusions as shipping coverage.
It does not guarantee delivery of every product to every address. The Austria
pricing-context decision is independent and remains unchanged.

## Anker South Korea

Implementation: none. This retailer has no storefront integration, so it has no `StorefrontConfig` record; the inventory tests skip it.

| Field                  | Value                                            |
| ---------------------- | ------------------------------------------------ |
| Retailer code          | `ankerkr`                                        |
| Storefront website     | https://ankerkorea.co.kr/                        |
| Shopify shop value     | Unknown                                          |
| Storefront GraphQL API | Unknown                                          |
| Target markets         | South Korea (KR)                                 |
| Shipping countries     | Unknown                                          |
| GraphQL API status     | Unknown; no endpoint established in the research |
|                        |                                                  |
|                        |                                                  |

**Shipping configuration checked 2026-11-13:** Cafe24 platform indicators found; no Shopify endpoint established. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales      | URL                       | GraphQL country/region | GraphQL language |
| ---------------------- | ------------------------- | ---------------------- | ---------------- |
| ko                     | https://ankerkorea.co.kr/ | KR                     | KO               |
| ko-KR (alternate link) | https://ankerkorea.co.kr/ | KR                     | KO               |

**Shipping evidence:** No reliable enumerated shipping-country definition established in the reviewed sources.

## Anker United States

Implementation: [`retailers_anker::prelude::get_storefront_config("ankercom")`](../lib/retailers_anker/src/ankercom/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                  |
| ---------------------- | ------------------------------------------------------ |
| Retailer code          | `ankercom`                                             |
| Storefront website     | https://www.anker.com/                                 |
| Shopify shop value     | ankerus.myshopify.com                                  |
| Storefront GraphQL API | https://ankerus.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United States (US)                                     |
| Shipping countries     | US                                                     |
|                        |                                                        |
|                        |                                                        |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| en-US             | https://www.anker.com/ | US                     | EN               |

## Anker Poland

Implementation: [`retailers_anker::prelude::get_storefront_config("ankerpl")`](../lib/retailers_anker/src/ankerpl/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                                                          |
| ---------------------- | ---------------------------------------------------------------------------------------------- |
| Retailer code          | `ankerpl`                                                                                      |
| Storefront website     | https://www.anker.com/eu-pl                                                                    |
| Shopify shop value     | ankereu.myshopify.com                                                                          |
| Storefront GraphQL API | https://ankereu.myshopify.com/api/2026-10/graphql.json                                         |
| Target markets         | Poland (PL)                                                                                    |
| Shipping countries     | AT, BE, BG, HR, CZ, DK, EE, FI, FR, GR, ES, NL, LT, LU, DE, PL, PT, RO, SE, SK, SI, HU, IT, LV |
|                        |                                                                                                |
|                        |                                                                                                |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                         | GraphQL country/region | GraphQL language |
| ----------------- | --------------------------- | ---------------------- | ---------------- |
| pl-PL             | https://www.anker.com/eu-pl | PL                     | PL               |

## Anker France

Implementation: [`retailers_anker::prelude::get_storefront_config("ankerfr")`](../lib/retailers_anker/src/ankerfr/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                   |
| ---------------------- | ------------------------------------------------------- |
| Retailer code          | `ankerfr`                                               |
| Storefront website     | https://www.anker.com/fr                                |
| Shopify shop value     | ankerfra.myshopify.com                                  |
| Storefront GraphQL API | https://ankerfra.myshopify.com/api/2026-10/graphql.json |
| Target markets         | France (FR)                                             |
| Shipping countries     | FR, IT, ES                                              |
|                        |                                                         |
|                        |                                                         |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                      | GraphQL country/region | GraphQL language |
| ----------------- | ------------------------ | ---------------------- | ---------------- |
| fr-FR             | https://www.anker.com/fr | FR                     | FR               |

## Anker New Zealand

Implementation: [`retailers_anker::prelude::get_storefront_config("ankernz")`](../lib/retailers_anker/src/ankernz/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                  |
| ---------------------- | ------------------------------------------------------ |
| Retailer code          | `ankernz`                                              |
| Storefront website     | https://www.anker.com/nz                               |
| Shopify shop value     | ankernz.myshopify.com                                  |
| Storefront GraphQL API | https://ankernz.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Unknown (conflicting evidence)                         |
| Shipping countries     | Unknown                                                |
|                        |                                                        |
|                        |                                                        |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                      | GraphQL country/region | GraphQL language |
| ----------------- | ------------------------ | ---------------------- | ---------------- |
| en-NZ             | https://www.anker.com/nz | NZ                     | EN               |

**Shipping evidence:** No reliable enumerated shipping-country definition established in the reviewed sources.

**Research notes:** Conflicting localization: /nz and html[lang]=en-NZ, but footer/selector label says Australia / English. Target market unresolved. Shipping policy returns 404.

## Anker United Kingdom

Implementation: [`retailers_anker::prelude::get_storefront_config("ankeruk")`](../lib/retailers_anker/src/ankeruk/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                      |
| ---------------------- | ---------------------------------------------------------- |
| Retailer code          | `ankeruk`                                                  |
| Storefront website     | https://www.anker.com/uk                                   |
| Shopify shop value     | oneanker-uk.myshopify.com                                  |
| Storefront GraphQL API | https://oneanker-uk.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United Kingdom (GB)                                        |
| Shipping countries     | GB                                                         |
|                        |                                                            |
|                        |                                                            |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                      | GraphQL country/region | GraphQL language |
| ----------------- | ------------------------ | ---------------------- | ---------------- |
| en-GB             | https://www.anker.com/uk | GB                     | EN               |

## Anker Vietnam

Implementation: [`retailers_anker::prelude::get_storefront_config("ankervn")`](../lib/retailers_anker/src/ankervn/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                         |
| ---------------------- | ------------------------------------------------------------- |
| Retailer code          | `ankervn`                                                     |
| Storefront website     | https://www.anker.com/vn                                      |
| Shopify shop value     | ankervnm.myshopify.com                                        |
| Storefront GraphQL API | https://ankervnm.myshopify.com/api/2026-10/graphql.json       |
| Target markets         | Unknown                                                       |
| Shipping countries     | Unknown                                                       |
| GraphQL API status     | Responded with GraphQL error: Online Store channel is locked. |
|                        |                                                               |
|                        |                                                               |

**Shipping configuration checked 2026-11-13:** Online Store channel is locked; shipping configuration unavailable. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                      | GraphQL country/region | GraphQL language |
| ----------------- | ------------------------ | ---------------------- | ---------------- |
| vi-VN             | https://www.anker.com/vn | VN                     | VI               |

**Shipping evidence:** No reliable enumerated shipping-country definition established in the reviewed sources.

**Research notes:** html[lang]=vi-VN establishes the declared language. No explicit target-market label independently confirmed.

## Anker Italy

Implementation: [`retailers_anker::prelude::get_storefront_config("ankeritalycom")`](../lib/retailers_anker/src/ankeritalycom/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                      |
| ---------------------- | ---------------------------------------------------------- |
| Retailer code          | `ankeritalycom`                                            |
| Storefront website     | https://www.ankeritaly.com/                                |
| Shopify shop value     | anker-italy.myshopify.com                                  |
| Storefront GraphQL API | https://anker-italy.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Italy (IT)                                                 |
| Shipping countries     | IT, FR, ES, DE, MT                                         |
|                        |                                                            |
|                        |                                                            |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                         | GraphQL country/region | GraphQL language |
| ----------------- | --------------------------- | ---------------------- | ---------------- |
| it-IT             | https://www.ankeritaly.com/ | IT                     | IT               |

**Research notes:** Italian html[lang] and storefront title conflict with English label in some cross-site selectors; English support is not confirmed.

## Anker Japan

Implementation: [`retailers_anker::prelude::get_storefront_config("ankerjapancom")`](../lib/retailers_anker/src/ankerjapancom/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                                    |
| ---------------------- | ------------------------------------------------------------------------ |
| Retailer code          | `ankerjapancom`                                                          |
| Storefront website     | https://www.ankerjapan.com/                                              |
| Shopify shop value     | anker-japan-official-site.myshopify.com                                  |
| Storefront GraphQL API | https://anker-japan-official-site.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Japan (JP)                                                               |
| Shipping countries     | JP                                                                       |
|                        |                                                                          |
|                        |                                                                          |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                         | GraphQL country/region | GraphQL language |
| ----------------- | --------------------------- | ---------------------- | ---------------- |
| ja                | https://www.ankerjapan.com/ | JP                     | JA               |

## Anker Nordics

Historical combined retailer code retained for reading existing records. Its
processing integration is retired, including queued work. Use the five
country-specific entries below; existing data is not remapped. See
[ADR 86](adr/0086-country-owned-legacy-anker-integrations.md).

Implementation: none. This retailer has no storefront integration, so it has no `StorefrontConfig` record; the inventory tests skip it.

| Field                  | Value                                                              |
| ---------------------- | ------------------------------------------------------------------ |
| Retailer code          | `ankernordicscom`                                                  |
| Storefront website     | https://www.ankernordics.com/                                      |
| Shopify shop value     | ankerneu.myshopify.com                                             |
| Storefront GraphQL API | https://ankerneu.myshopify.com/api/2026-10/graphql.json            |
| Target markets         | Denmark (DK), Finland (FI), Iceland (IS), Norway (NO), Sweden (SE) |
| Shipping countries     | SE, NO, DK, FI, IS                                                 |
|                        |                                                                    |
|                        |                                                                    |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales      | URL                           | GraphQL country/region | GraphQL language |
| ---------------------- | ----------------------------- | ---------------------- | ---------------- |
| en-SE                  | https://www.ankernordics.com/ | SE                     | EN               |
| en-dk (alternate link) | https://www.ankernordics.com/ | DK                     | EN               |
| en-fi (alternate link) | https://www.ankernordics.com/ | FI                     | EN               |
| en-is (alternate link) | https://www.ankernordics.com/ | IS                     | EN               |
| en-no (alternate link) | https://www.ankernordics.com/ | NO                     | EN               |

## Anker Sweden

Implementation: [`retailers_anker::prelude::get_storefront_config("ankerse")`](../lib/retailers_anker/src/ankerse/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                   |
| ---------------------- | ------------------------------------------------------- |
| Retailer code          | `ankerse`                                               |
| Storefront website     | https://www.ankernordics.com/                           |
| Shopify shop value     | ankerneu.myshopify.com                                  |
| Storefront GraphQL API | https://ankerneu.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Sweden (SE)                                             |
| Shipping countries     | SE, NO, DK, FI, IS                                      |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                           | GraphQL country/region | GraphQL language |
| ----------------- | ----------------------------- | ---------------------- | ---------------- |
| en-SE             | https://www.ankernordics.com/ | SE                     | EN               |

This is the Sweden market of the shared Nordic store, using explicit `SE` / `EN`
collection context. Shipping countries describe the shared store's documented
coverage, independently of this retailer's target market. The locale spelling
retains the Nordic inventory evidence. No country URL prefix is introduced.
See [ADR 47](adr/0047-nordic-retailer-markets.md) and the
[existing storefront configuration](../lib/retailers_anker/src/ankerse/mod.rs).

## Anker Norway

Implementation: [`retailers_anker::prelude::get_storefront_config("ankerno")`](../lib/retailers_anker/src/ankerno/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                   |
| ---------------------- | ------------------------------------------------------- |
| Retailer code          | `ankerno`                                               |
| Storefront website     | https://www.ankernordics.com/                           |
| Shopify shop value     | ankerneu.myshopify.com                                  |
| Storefront GraphQL API | https://ankerneu.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Norway (NO)                                             |
| Shipping countries     | SE, NO, DK, FI, IS                                      |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales      | URL                           | GraphQL country/region | GraphQL language |
| ---------------------- | ----------------------------- | ---------------------- | ---------------- |
| en-no (alternate link) | https://www.ankernordics.com/ | NO                     | EN               |

This is the Norway market of the shared Nordic store, using explicit `NO` / `EN`
collection context. Shipping countries describe the shared store's documented
coverage, independently of this retailer's target market. The locale spelling
retains the Nordic inventory evidence. No country URL prefix is introduced.
See [ADR 47](adr/0047-nordic-retailer-markets.md) and the
[existing storefront configuration](../lib/retailers_anker/src/ankerno/mod.rs).

## Anker Finland

Implementation: [`retailers_anker::prelude::get_storefront_config("ankerfi")`](../lib/retailers_anker/src/ankerfi/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                   |
| ---------------------- | ------------------------------------------------------- |
| Retailer code          | `ankerfi`                                               |
| Storefront website     | https://www.ankernordics.com/                           |
| Shopify shop value     | ankerneu.myshopify.com                                  |
| Storefront GraphQL API | https://ankerneu.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Finland (FI)                                            |
| Shipping countries     | SE, NO, DK, FI, IS                                      |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales      | URL                           | GraphQL country/region | GraphQL language |
| ---------------------- | ----------------------------- | ---------------------- | ---------------- |
| en-fi (alternate link) | https://www.ankernordics.com/ | FI                     | EN               |

This is the Finland market of the shared Nordic store, using explicit `FI` / `EN`
collection context. Shipping countries describe the shared store's documented
coverage, independently of this retailer's target market. The locale spelling
retains the Nordic inventory evidence. No country URL prefix is introduced.
See [ADR 47](adr/0047-nordic-retailer-markets.md) and the
[existing storefront configuration](../lib/retailers_anker/src/ankerfi/mod.rs).

## Anker Iceland

Implementation: [`retailers_anker::prelude::get_storefront_config("ankeris")`](../lib/retailers_anker/src/ankeris/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                   |
| ---------------------- | ------------------------------------------------------- |
| Retailer code          | `ankeris`                                               |
| Storefront website     | https://www.ankernordics.com/                           |
| Shopify shop value     | ankerneu.myshopify.com                                  |
| Storefront GraphQL API | https://ankerneu.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Iceland (IS)                                            |
| Shipping countries     | SE, NO, DK, FI, IS                                      |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales      | URL                           | GraphQL country/region | GraphQL language |
| ---------------------- | ----------------------------- | ---------------------- | ---------------- |
| en-is (alternate link) | https://www.ankernordics.com/ | IS                     | EN               |

This is the Iceland market of the shared Nordic store, using explicit `IS` / `EN`
collection context. Shipping countries describe the shared store's documented
coverage, independently of this retailer's target market. The locale spelling
retains the Nordic inventory evidence. No country URL prefix is introduced.
See [ADR 47](adr/0047-nordic-retailer-markets.md) and the
[existing storefront configuration](../lib/retailers_anker/src/ankeris/mod.rs).

## Anker Denmark

Implementation: [`retailers_anker::prelude::get_storefront_config("ankerdk")`](../lib/retailers_anker/src/ankerdk/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                   |
| ---------------------- | ------------------------------------------------------- |
| Retailer code          | `ankerdk`                                               |
| Storefront website     | https://www.ankernordics.com/                           |
| Shopify shop value     | ankerneu.myshopify.com                                  |
| Storefront GraphQL API | https://ankerneu.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Denmark (DK)                                            |
| Shipping countries     | SE, NO, DK, FI, IS                                      |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales      | URL                           | GraphQL country/region | GraphQL language |
| ---------------------- | ----------------------------- | ---------------------- | ---------------- |
| en-dk (alternate link) | https://www.ankernordics.com/ | DK                     | EN               |

This is the Denmark market of the shared Nordic store, using explicit `DK` / `EN`
collection context. Shipping countries describe the shared store's documented
coverage, independently of this retailer's target market. The locale spelling
retains the Nordic inventory evidence. No country URL prefix is introduced.
See [ADR 47](adr/0047-nordic-retailer-markets.md) and the
[existing storefront configuration](../lib/retailers_anker/src/ankerdk/mod.rs).

## Minisforum Australia

Implementation: [`retailers_minisforum::prelude::get_storefront_config("minisforumau")`](../lib/retailers_minisforum/src/minisforumau/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `minisforumau`                                           |
| Storefront website     | https://au.minisforum.com/                               |
| Shopify shop value     | j94nqi-71.myshopify.com                                  |
| Storefront GraphQL API | https://j94nqi-71.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Australia (AU)                                           |
| Shipping countries     | AU                                                       |
|                        |                                                          |
|                        |                                                          |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| en                | https://au.minisforum.com/ | AU                     | EN               |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Minisforum Canada

Implementation: [`retailers_minisforum::prelude::get_storefront_config("minisforumca")`](../lib/retailers_minisforum/src/minisforumca/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `minisforumca`                                               |
| Storefront website     | https://ca.minisforum.com/                                   |
| Shopify shop value     | seaminisforum.myshopify.com                                  |
| Storefront GraphQL API | https://seaminisforum.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Canada (CA)                                                  |
| Shipping countries     | CA                                                           |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| en                | https://ca.minisforum.com/ | CA                     | EN               |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Minisforum Hong Kong

Implementation: [`retailers_minisforum::prelude::get_storefront_config("minisforumhk")`](../lib/retailers_minisforum/src/minisforumhk/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                       |
| ---------------------- | ----------------------------------------------------------- |
| Retailer code          | `minisforumhk`                                              |
| Storefront website     | https://minisforum.hk/                                      |
| Shopify shop value     | minisforumcn.myshopify.com                                  |
| Storefront GraphQL API | https://minisforumcn.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Hong Kong (HK)                                              |
| Shipping countries     | HK                                                          |
|                        |                                                             |
|                        |                                                             |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| zh-TW             | https://minisforum.hk/ | HK                     | ZH_TW            |

**Shipping evidence:** Status: published partial. Homepage explicitly offers Hong Kong delivery, excluding outlying islands/remote areas. Other destinations not established. Source: https://minisforum.hk/.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. html[lang]=zh-TW identifies a language variety, not a Taiwan target market. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The intended language code ZH_TW corresponds to the observed zh-TW locale; the reviewed language is Traditional Chinese (zh-Hant).

## Minisforum South Korea

Implementation: [`retailers_minisforum::prelude::get_storefront_config("minisforumkr")`](../lib/retailers_minisforum/src/minisforumkr/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                       |
| ---------------------- | ----------------------------------------------------------- |
| Retailer code          | `minisforumkr`                                              |
| Storefront website     | https://minisforum.kr/                                      |
| Shopify shop value     | minisforumkr.myshopify.com                                  |
| Storefront GraphQL API | https://minisforumkr.myshopify.com/api/2026-10/graphql.json |
| Target markets         | South Korea (KR)                                            |
| Shipping countries     | KR                                                          |
|                        |                                                             |
|                        |                                                             |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| ko                | https://minisforum.kr/ | KR                     | KO               |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Minisforum Russia

Implementation: [`retailers_minisforum::prelude::get_storefront_config("minisforumru")`](../lib/retailers_minisforum/src/minisforumru/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                              |
| ---------------------- | ------------------------------------------------------------------ |
| Retailer code          | `minisforumru`                                                     |
| Storefront website     | https://minisforum.ru/                                             |
| Shopify shop value     | store-minisforum-ru.myshopify.com                                  |
| Storefront GraphQL API | https://store-minisforum-ru.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Russia (RU)                                                        |
| Shipping countries     | Unknown                                                            |
|                        |                                                                    |
|                        |                                                                    |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| ru                | https://minisforum.ru/ | RU                     | RU               |

**Shipping evidence:** Status: unbounded claim. FAQ claims worldwide shipping without an enumerated country list; do not expand to all ISO countries. Source: https://minisforum.ru/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. html[lang] conflicts with the reviewed language declaration; retained as raw evidence only. Shopify.country=US and locale=en conflict with official Russia/Russian table; not accepted as target-market or English-language evidence. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Minisforum Europe

Implementation: [`retailers_minisforum::prelude::get_storefront_config("minisforumeu")`](../lib/retailers_minisforum/src/minisforumeu/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                                                                                                      |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Retailer code          | `minisforumeu`                                                                                                                             |
| Storefront website     | https://minisforumpc.eu/                                                                                                                   |
| Shopify shop value     | minisforum-eu.myshopify.com                                                                                                                |
| Storefront GraphQL API | https://minisforum-eu.myshopify.com/api/2026-10/graphql.json                                                                               |
| Target markets         | EU Countries (EU)                                                                                                                          |
| Shipping countries     | CZ, DK, AT, BE, BG, HR, EE, FI, FR, DE, GR, IE, IT, LV, LT, LU, NL, PL, PT, RO, SK, SI, ES, SE, HU, CH, AL, BY, MK, IS, BA, VA, LI, MC, SM |
|                        |                                                                                                                                            |
|                        |                                                                                                                                            |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                      | GraphQL country/region | GraphQL language |
| ----------------- | ------------------------ | ---------------------- | ---------------- |
| en                | https://minisforumpc.eu/ | Not selected           | EN               |

| Advertised-only locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------------- | -------------------------- | ---------------------- | ---------------- |
| de                      | https://minisforumpc.eu/de | Not selected           | DE               |

**Shipping evidence:** Status: conflicting lists. Where to Buy table; Spanish island exclusions. Another shipping section additionally lists NO and GB: retain conflict instead of merging. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. A collection country has not been selected. EU labels the regional storefront and is not a Shopify CountryCode; the Anker Europe AT pricing decision does not apply here. Rows in the advertised-only tables are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Minisforum France

Implementation: [`retailers_minisforum::prelude::get_storefront_config("minisforumfr")`](../lib/retailers_minisforum/src/minisforumfr/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `minisforumfr`                                               |
| Storefront website     | https://minisforumpc.fr/                                     |
| Shopify shop value     | minisforum-fr.myshopify.com                                  |
| Storefront GraphQL API | https://minisforum-fr.myshopify.com/api/2026-10/graphql.json |
| Target markets         | France (FR)                                                  |
| Shipping countries     | BE, FR, LU                                                   |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                      | GraphQL country/region | GraphQL language |
| ----------------- | ------------------------ | ---------------------- | ---------------- |
| fr                | https://minisforumpc.fr/ | FR                     | FR               |

| Advertised-only locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------------- | -------------------------- | ---------------------- | ---------------- |
| en                      | https://minisforumpc.fr/en | Not selected           | EN               |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Shopify.country=HK on France store; not accepted as target market. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Rows in the advertised-only tables are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Minisforum United States

Implementation: [`retailers_minisforum::prelude::get_storefront_config("minisforumus")`](../lib/retailers_minisforum/src/minisforumus/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                                                                                                                      |
| ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Retailer code          | `minisforumus`                                                                                                                                             |
| Storefront website     | https://store.minisforum.com/                                                                                                                              |
| Shopify shop value     | minisforum.myshopify.com                                                                                                                                   |
| Storefront GraphQL API | https://minisforum.myshopify.com/api/2026-10/graphql.json                                                                                                  |
| Target markets         | United States and others (not enumerated) (US)                                                                                                             |
| Shipping countries     | US, CA, AE, AU, SA, PH, NZ, AG, BS, BB, BZ, BM, KY, DO, EC, SV, GD, HN, JM, LR, MX, NI, PA, KN, LC, TT, TC, VE, FJ, PG, WS, SB, TO, VU, SG, ID, MY, IL, KW |
|                        |                                                                                                                                                            |
|                        |                                                                                                                                                            |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales                   | URL                                | GraphQL country/region | GraphQL language |
| ----------------------------------- | ---------------------------------- | ---------------------- | ---------------- |
| en                                  | https://store.minisforum.com/      | US                     | EN               |
| en-CA (context verified 2026-11-13) | https://store.minisforum.com/en-ca | CA                     | EN               |
| en-PE (context verified 2026-11-13) | https://store.minisforum.com/en-os | PE                     | EN               |

| Advertised-only locales | URL                                | GraphQL country/region | GraphQL language |
| ----------------------- | ---------------------------------- | ---------------------- | ---------------- |
| en-CL                   | https://store.minisforum.com/en-os | Not selected           | EN               |

**Shipping evidence:** Status: published. Official supported-region table; US territories and military addresses excluded. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Rows in the advertised-only tables are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Minisforum Japan

Implementation: [`retailers_minisforum::prelude::get_storefront_config("minisforumjp")`](../lib/retailers_minisforum/src/minisforumjp/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `minisforumjp`                                               |
| Storefront website     | https://www.minisforum.jp/                                   |
| Shopify shop value     | minisforum-jp.myshopify.com                                  |
| Storefront GraphQL API | https://minisforum-jp.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Japan (JP)                                                   |
| Shipping countries     | JP                                                           |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| ja                | https://www.minisforum.jp/ | JP                     | JA               |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Minisforum United Kingdom

Implementation: [`retailers_minisforum::prelude::get_storefront_config("minisforumuk")`](../lib/retailers_minisforum/src/minisforumuk/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `minisforumuk`                                               |
| Storefront website     | https://www.minisforum.uk/                                   |
| Shopify shop value     | minisforum-uk.myshopify.com                                  |
| Storefront GraphQL API | https://minisforum-uk.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United Kingdom (GB)                                          |
| Shipping countries     | IE, GB                                                       |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| en                | https://www.minisforum.uk/ | GB                     | EN               |

**Shipping evidence:** Status: published. Official Where to Buy table explicitly lists supported destinations. Source: https://minisforumpc.eu/pages/faqs.

**Research notes:** Official Primary Market / Language table; not inferred from Shopify.country. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Ugreen Canada

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreenca")`](../lib/retailers_ugreen/src/ugreenca/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreenca`                                               |
| Storefront website     | https://ca.ugreen.com/                                   |
| Shopify shop value     | ugreen-ca.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-ca.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Canada (CA)                                              |
| Shipping countries     | Unknown                                                  |
|                        |                                                          |
|                        |                                                          |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales                | URL                      | GraphQL country/region | GraphQL language |
| -------------------------------- | ------------------------ | ---------------------- | ---------------- |
| en                               | https://ca.ugreen.com/   | CA                     | EN               |
| fr (context verified 2026-11-13) | https://ca.ugreen.com/fr | CA                     | FR               |

**Shipping evidence:** Status: conflict. Canada-branded storefront policy instead says United States. Do not accept US or infer CA as verified delivery coverage. Source: https://ca.ugreen.com/policies/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Rows in the advertised-only tables are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Ugreen Germany

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreende")`](../lib/retailers_ugreen/src/ugreende/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreende`                                               |
| Storefront website     | https://de.ugreen.com/                                   |
| Shopify shop value     | ugreen-de.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-de.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Germany (DE)                                             |
| Shipping countries     | DE, AT, DK, FR, NL, PL                                   |
|                        |                                                          |
|                        |                                                          |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| de                | https://de.ugreen.com/ | DE                     | DE               |

**Shipping evidence:** Status: published.  Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Ugreen Europe

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreeneu")`](../lib/retailers_ugreen/src/ugreeneu/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                                                                  |
| ---------------------- | ------------------------------------------------------------------------------------------------------ |
| Retailer code          | `ugreeneu`                                                                                             |
| Storefront website     | https://eu.ugreen.com/                                                                                 |
| Shopify shop value     | ugreen-eu.myshopify.com                                                                                |
| Storefront GraphQL API | https://ugreen-eu.myshopify.com/api/2026-10/graphql.json                                               |
| Target markets         | Europe (EU)                                                                                            |
| Shipping countries     | BG, HR, EE, HU, PT, RO, SI, SE, AT, BE, CZ, FR, DE, IT, MT, NL, PL, ES, DK, LV, FI, LU, SK, CY, LT, CH |
|                        |                                                                                                        |
|                        |                                                                                                        |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales                   | URL                         | GraphQL country/region | GraphQL language |
| ----------------------------------- | --------------------------- | ---------------------- | ---------------- |
| en                                  | https://eu.ugreen.com/      | Not selected           | EN               |
| bg-BG (context verified 2026-11-13) | https://eu.ugreen.com/bg-bg | BG                     | BG               |
| cs-CZ (context verified 2026-11-13) | https://eu.ugreen.com/cs-cz | CZ                     | CS               |
| da-DK (context verified 2026-11-13) | https://eu.ugreen.com/da-dk | DK                     | DA               |
| de-AT (context verified 2026-11-13) | https://eu.ugreen.com/de-at | AT                     | DE               |

| Advertised-only locales | URL                         | GraphQL country/region | GraphQL language |
| ----------------------- | --------------------------- | ---------------------- | ---------------- |
| bg                      | https://eu.ugreen.com/bg    | Not selected           | BG               |
| cs                      | https://eu.ugreen.com/cs    | Not selected           | CS               |
| da                      | https://eu.ugreen.com/da    | Not selected           | DA               |
| de                      | https://eu.ugreen.com/de    | Not selected           | DE               |
| de-BE                   | https://eu.ugreen.com/de-be | Not selected           | DE               |
| de-CH                   | https://eu.ugreen.com/de-ch | Not selected           | DE               |
| el                      | https://eu.ugreen.com/el    | Not selected           | EL               |
| el-GR                   | https://eu.ugreen.com/el-gr | Not selected           | EL               |
| en-AT                   | https://eu.ugreen.com/en-at | Not selected           | EN               |
| en-BE                   | https://eu.ugreen.com/en-be | Not selected           | EN               |
| en-BG                   | https://eu.ugreen.com/en-bg | Not selected           | EN               |
| en-CH                   | https://eu.ugreen.com/en-ch | Not selected           | EN               |
| en-CZ                   | https://eu.ugreen.com/en-cz | Not selected           | EN               |
| en-DK                   | https://eu.ugreen.com/en-dk | Not selected           | EN               |
| en-EE                   | https://eu.ugreen.com/en-ee | Not selected           | EN               |
| en-ES                   | https://eu.ugreen.com/en-es | Not selected           | EN               |
| en-FI                   | https://eu.ugreen.com/en-fi | Not selected           | EN               |
| en-FR                   | https://eu.ugreen.com/en-fr | Not selected           | EN               |
| en-GR                   | https://eu.ugreen.com/en-gr | Not selected           | EN               |
| en-HR                   | https://eu.ugreen.com/en-hr | Not selected           | EN               |
| en-HU                   | https://eu.ugreen.com/en-hu | Not selected           | EN               |
| en-IT                   | https://eu.ugreen.com/en-it | Not selected           | EN               |
| en-LV                   | https://eu.ugreen.com/en-lv | Not selected           | EN               |
| en-NL                   | https://eu.ugreen.com/en-nl | Not selected           | EN               |
| en-PL                   | https://eu.ugreen.com/en-pl | Not selected           | EN               |
| en-PT                   | https://eu.ugreen.com/en-pt | Not selected           | EN               |
| en-RO                   | https://eu.ugreen.com/en-ro | Not selected           | EN               |
| en-SE                   | https://eu.ugreen.com/en-se | Not selected           | EN               |
| en-SI                   | https://eu.ugreen.com/en-si | Not selected           | EN               |
| en-SK                   | https://eu.ugreen.com/en-sk | Not selected           | EN               |
| es                      | https://eu.ugreen.com/es    | Not selected           | ES               |
| es-ES                   | https://eu.ugreen.com/es-es | Not selected           | ES               |
| et                      | https://eu.ugreen.com/et    | Not selected           | ET               |
| et-EE                   | https://eu.ugreen.com/et-ee | Not selected           | ET               |
| fi                      | https://eu.ugreen.com/fi    | Not selected           | FI               |
| fr                      | https://eu.ugreen.com/fr    | Not selected           | FR               |
| fr-BE                   | https://eu.ugreen.com/fr-be | Not selected           | FR               |
| fr-FR                   | https://eu.ugreen.com/fr-fr | Not selected           | FR               |
| hr                      | https://eu.ugreen.com/hr    | Not selected           | HR               |
| hr-HR                   | https://eu.ugreen.com/hr-hr | Not selected           | HR               |
| hu                      | https://eu.ugreen.com/hu    | Not selected           | HU               |
| hu-HU                   | https://eu.ugreen.com/hu-hu | Not selected           | HU               |
| it                      | https://eu.ugreen.com/it    | Not selected           | IT               |
| it-IT                   | https://eu.ugreen.com/it-it | Not selected           | IT               |
| lv                      | https://eu.ugreen.com/lv    | Not selected           | LV               |
| lv-LV                   | https://eu.ugreen.com/lv-lv | Not selected           | LV               |
| nl                      | https://eu.ugreen.com/nl    | Not selected           | NL               |
| nl-BE                   | https://eu.ugreen.com/nl-be | Not selected           | NL               |
| nl-NL                   | https://eu.ugreen.com/nl-nl | Not selected           | NL               |
| pl                      | https://eu.ugreen.com/pl    | Not selected           | PL               |
| pl-PL                   | https://eu.ugreen.com/pl-pl | Not selected           | PL               |
| pt                      | https://eu.ugreen.com/pt    | Not selected           | PT               |
| pt-PT                   | https://eu.ugreen.com/pt-pt | Not selected           | PT               |
| ro                      | https://eu.ugreen.com/ro    | Not selected           | RO               |
| ro-RO                   | https://eu.ugreen.com/ro-ro | Not selected           | RO               |
| sk                      | https://eu.ugreen.com/sk    | Not selected           | SK               |
| sk-SK                   | https://eu.ugreen.com/sk-sk | Not selected           | SK               |
| sl                      | https://eu.ugreen.com/sl    | Not selected           | SL               |
| sl-SI                   | https://eu.ugreen.com/sl-si | Not selected           | SL               |
| sv                      | https://eu.ugreen.com/sv    | Not selected           | SV               |
| sv-SE                   | https://eu.ugreen.com/sv-se | Not selected           | SV               |

**Shipping evidence:** Status: published. Portugal and Spain have named island/enclave exclusions. Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. A collection country has not been selected. EU labels the regional storefront and is not a Shopify CountryCode; the Anker Europe AT pricing decision does not apply here. Rows in the advertised-only tables are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Ugreen France

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreenfr")`](../lib/retailers_ugreen/src/ugreenfr/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreenfr`                                               |
| Storefront website     | https://fr.ugreen.com/                                   |
| Shopify shop value     | ugreen-fr.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-fr.myshopify.com/api/2026-10/graphql.json |
| Target markets         | France (FR)                                              |
| Shipping countries     | FR, AT, BE, DE, IT, ES                                   |
|                        |                                                          |
|                        |                                                          |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| fr                | https://fr.ugreen.com/ | FR                     | FR               |

**Shipping evidence:** Status: published.  Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Ugreen South Korea

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreenkr")`](../lib/retailers_ugreen/src/ugreenkr/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreenkr`                                               |
| Storefront website     | https://kr.ugreen.com/                                   |
| Shopify shop value     | ugreen-kr.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-kr.myshopify.com/api/2026-10/graphql.json |
| Target markets         | South Korea (KR)                                         |
| Shipping countries     | Unknown                                                  |
|                        |                                                          |
|                        |                                                          |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| ko                | https://kr.ugreen.com/ | KR                     | KO               |

**Shipping evidence:** Status: unknown. No reliable enumerated shipping-country definition established in the reviewed sources.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Ugreen NAS Australia

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennasau")`](../lib/retailers_ugreen/src/ugreennasau/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasau`                                                |
| Storefront website     | https://nas-au.ugreen.com/                                   |
| Shopify shop value     | nas-ugreen-au.myshopify.com                                  |
| Storefront GraphQL API | https://nas-ugreen-au.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Australia (AU)                                               |
| Shipping countries     | Unknown                                                      |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| en                | https://nas-au.ugreen.com/ | AU                     | EN               |

**Shipping evidence:** Status: conflict. Australia storefront policy instead says United States. Coverage unresolved. Source: https://ai-au.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Related selector URL: https://ai-au.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Canada

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennasca")`](../lib/retailers_ugreen/src/ugreennasca/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasca`                                                |
| Storefront website     | https://nas-ca.ugreen.com/                                   |
| Shopify shop value     | ugreen-nas-ca.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas-ca.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Canada (CA)                                                  |
| Shipping countries     | CA                                                           |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| en                | https://nas-ca.ugreen.com/ | CA                     | EN               |

**Shipping evidence:** Status: published. Most areas; no exhaustive excluded-postcode list. Source: https://ai-ca.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Related selector URL: https://ai-ca.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Germany

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennasde")`](../lib/retailers_ugreen/src/ugreennasde/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasde`                                                |
| Storefront website     | https://nas-de.ugreen.com/                                   |
| Shopify shop value     | ugreen-de-nas.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-de-nas.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Germany (DE)                                                 |
| Shipping countries     | DE, AT                                                       |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| de (selector)     | https://nas-de.ugreen.com/ | DE                     | DE               |

**Shipping evidence:** Status: conflicting lists. Delivery-area section and product pages agree on DE/AT. Later legacy text lists additional European countries; do not merge. Source: https://ai-de.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-de.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Spain

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennases")`](../lib/retailers_ugreen/src/ugreennases/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennases`                                                |
| Storefront website     | https://nas-es.ugreen.com/                                   |
| Shopify shop value     | ugreen-nas-es.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas-es.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Spain (ES)                                                   |
| Shipping countries     | ES                                                           |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| es (selector)     | https://nas-es.ugreen.com/ | ES                     | ES               |

**Shipping evidence:** Status: published. Canary Islands excluded. Source: https://ai-es.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-es.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Europe

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennaseu")`](../lib/retailers_ugreen/src/ugreennaseu/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                              |
| ---------------------- | ------------------------------------------------------------------ |
| Retailer code          | `ugreennaseu`                                                      |
| Storefront website     | https://nas-eu.ugreen.com/                                         |
| Shopify shop value     | ugreen-nas-eu.myshopify.com                                        |
| Storefront GraphQL API | https://ugreen-nas-eu.myshopify.com/api/2026-10/graphql.json       |
| Target markets         | Europe (EU)                                                        |
| Shipping countries     | DE, FR, IT, NL, ES, BE, PT, RO, CZ, PL, HR, AT, BG, LT, IE, SE, GR |
|                        |                                                                    |
|                        |                                                                    |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| en (selector)     | https://nas-eu.ugreen.com/ | Not selected           | EN               |

**Shipping evidence:** Status: published. France overseas territories and Spain Canary Islands excluded. Source: https://ai-eu.ugreen.com/en-es/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. A collection country has not been selected. EU labels the regional storefront and is not a Shopify CountryCode; the Anker Europe AT pricing decision does not apply here. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-eu.ugreen.com/. Retained here as notes rather than separate retailer tables. Additional paths with unverified language support: https://nas-eu.ugreen.com/de, https://nas-eu.ugreen.com/en-be, https://nas-eu.ugreen.com/en-es, https://nas-eu.ugreen.com/en-fr, https://nas-eu.ugreen.com/en-it, https://nas-eu.ugreen.com/en-nl, https://nas-eu.ugreen.com/en-pt, https://nas-eu.ugreen.com/en-ro, https://nas-eu.ugreen.com/en-se, https://nas-eu.ugreen.com/es-es, https://nas-eu.ugreen.com/fr-be, https://nas-eu.ugreen.com/fr-fr, https://nas-eu.ugreen.com/it-it, https://nas-eu.ugreen.com/nl-be, https://nas-eu.ugreen.com/nl-nl, https://nas-eu.ugreen.com/pt-pt, https://nas-eu.ugreen.com/ro-ro, https://nas-eu.ugreen.com/sv-se. Their path names are not treated as confirmed locales.

## Ugreen NAS France

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennasfr")`](../lib/retailers_ugreen/src/ugreennasfr/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasfr`                                                |
| Storefront website     | https://nas-fr.ugreen.com/                                   |
| Shopify shop value     | nas-ugreen-fr.myshopify.com                                  |
| Storefront GraphQL API | https://nas-ugreen-fr.myshopify.com/api/2026-10/graphql.json |
| Target markets         | France (FR)                                                  |
| Shipping countries     | FR                                                           |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| fr (selector)     | https://nas-fr.ugreen.com/ | FR                     | FR               |

**Shipping evidence:** Status: published.  Source: https://ai-fr.ugreen.com/policies/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-fr.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Italy

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennasit")`](../lib/retailers_ugreen/src/ugreennasit/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasit`                                                |
| Storefront website     | https://nas-it.ugreen.com/                                   |
| Shopify shop value     | ugreen-nas-it.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas-it.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Italy (IT)                                                   |
| Shipping countries     | IT                                                           |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| it (selector)     | https://nas-it.ugreen.com/ | IT                     | IT               |

**Shipping evidence:** Status: published.  Source: https://ai-it.ugreen.com/policies/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-it.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS United Kingdom

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennasuk")`](../lib/retailers_ugreen/src/ugreennasuk/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasuk`                                                |
| Storefront website     | https://nas-uk.ugreen.com/                                   |
| Shopify shop value     | ugreen-nas-uk.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas-uk.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United Kingdom (GB)                                          |
| Shipping countries     | GB                                                           |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------- | -------------------------- | ---------------------- | ---------------- |
| en (selector)     | https://nas-uk.ugreen.com/ | GB                     | EN               |

**Shipping evidence:** Status: published.  Source: https://ai-uk.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://ai-uk.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS United States

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennas")`](../lib/retailers_ugreen/src/ugreennas/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                     |
| ---------------------- | --------------------------------------------------------- |
| Retailer code          | `ugreennas`                                               |
| Storefront website     | https://nas.ugreen.com/                                   |
| Shopify shop value     | ugreen-nas.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United States (US)                                        |
| Shipping countries     | US                                                        |
|                        |                                                           |
|                        |                                                           |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                     | GraphQL country/region | GraphQL language |
| ----------------- | ----------------------- | ---------------------- | ---------------- |
| en                | https://nas.ugreen.com/ | US                     | EN               |

**Shipping evidence:** Status: published. Excludes Alaska, Hawaii, Rhode Island, listed territories and military addresses. Source: https://ai.ugreen.com/pages/shipping.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Related selector URL: https://ai.ugreen.com/. Retained here as notes rather than separate retailer tables.

## Ugreen NAS Japan

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennasjp")`](../lib/retailers_ugreen/src/ugreennasjp/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Retailer code          | `ugreennasjp`                                                |
| Storefront website     | https://nas.ugreen.jp/                                       |
| Shopify shop value     | ugreen-nas-jp.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nas-jp.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Japan (JP)                                                   |
| Shipping countries     | JP                                                           |
|                        |                                                              |
|                        |                                                              |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| ja (selector)     | https://nas.ugreen.jp/ | JP                     | JA               |

**Shipping evidence:** Status: published. Some Okinawa, outlying islands and restricted delivery areas excluded. Source: https://store.ugreen.jp/policies/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research. Related selector URL: https://store.ugreen.jp/. Retained here as notes rather than separate retailer tables.

## Ugreen Netherlands

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreennl")`](../lib/retailers_ugreen/src/ugreennl/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreennl`                                               |
| Storefront website     | https://nl.ugreen.com/                                   |
| Shopify shop value     | ugreen-nl.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-nl.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Netherlands (NL)                                         |
| Shipping countries     | NL, BE, FR, DE, ES                                       |
|                        |                                                          |
|                        |                                                          |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| nl (selector)     | https://nl.ugreen.com/ | NL                     | NL               |

**Shipping evidence:** Status: published.  Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research.

## Ugreen Japan

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreenjp")`](../lib/retailers_ugreen/src/ugreenjp/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreenjp`                                               |
| Storefront website     | https://ugreen.jp/                                       |
| Shopify shop value     | ugreen-jp.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-jp.myshopify.com/api/2026-10/graphql.json |
| Target markets         | Japan (JP)                                               |
| Shipping countries     | Unknown                                                  |
|                        |                                                          |
|                        |                                                          |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                | GraphQL country/region | GraphQL language |
| ----------------- | ------------------ | ---------------------- | ---------------- |
| ja                | https://ugreen.jp/ | JP                     | JA               |

**Shipping evidence:** Status: unknown. No reliable enumerated shipping-country definition established in the reviewed sources.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query.

## Ugreen United Kingdom

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreenuk")`](../lib/retailers_ugreen/src/ugreenuk/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                            |
| ---------------------- | ---------------------------------------------------------------- |
| Retailer code          | `ugreenuk`                                                       |
| Storefront website     | https://uk.ugreen.com/                                           |
| Shopify shop value     | ugreen-uk.myshopify.com                                          |
| Storefront GraphQL API | https://ugreen-uk.myshopify.com/api/2026-10/graphql.json         |
| Target markets         | United Kingdom (GB)                                              |
| Shipping countries     | GB                                                               |
| GraphQL API status     | Shipping query verified 2026-11-13; storefront confirms backend. |
|                        |                                                                  |
|                        |                                                                  |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| en (selector)     | https://uk.ugreen.com/ | GB                     | EN               |

**Shipping evidence:** Status: published. Northern Ireland is included with a separate delivery timeframe. Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** No Shopify shop domain was verified for this URL in the reviewed sources. This does not establish whether the storefront uses Shopify. No backend is inherited from a parent retailer or inferred from a URL, language or market. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research.

## Ugreen United States

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreenus")`](../lib/retailers_ugreen/src/ugreenus/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                            |
| ---------------------- | ---------------------------------------------------------------- |
| Retailer code          | `ugreenus`                                                       |
| Storefront website     | https://us.ugreen.com/                                           |
| Shopify shop value     | ugreenmall.myshopify.com                                         |
| Storefront GraphQL API | https://ugreenmall.myshopify.com/api/2026-10/graphql.json        |
| Target markets         | United States (US)                                               |
| Shipping countries     | US                                                               |
| GraphQL API status     | Shipping query verified 2026-11-13; storefront confirms backend. |
|                        |                                                                  |
|                        |                                                                  |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                    | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------- | ---------------------- | ---------------- |
| en (selector)     | https://us.ugreen.com/ | US                     | EN               |

**Shipping evidence:** Status: published. Excludes Alaska, Hawaii, listed territories and military addresses. Source: https://us.ugreen.com/pages/shipping-policy.

**Research notes:** No Shopify shop domain was verified for this URL in the reviewed sources. This does not establish whether the storefront uses Shopify. No backend is inherited from a parent retailer or inferred from a URL, language or market. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The base language is declared by the storefront selector; direct page-language confirmation is unavailable in the research.

## Ugreen Main Site

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreencom")`](../lib/retailers_ugreen/src/ugreencom/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                           |
| ---------------------- | --------------------------------------------------------------- |
| Retailer code          | `ugreencom`                                                     |
| Storefront website     | https://www.ugreen.com/                                         |
| Shopify shop value     | ugreen-brandsite.myshopify.com                                  |
| Storefront GraphQL API | https://ugreen-brandsite.myshopify.com/api/2026-10/graphql.json |
| Target markets         | United States (US)                                              |
| Shipping countries     | Unknown                                                         |
|                        |                                                                 |
|                        |                                                                 |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales                   | URL                          | GraphQL country/region | GraphQL language |
| ----------------------------------- | ---------------------------- | ---------------------- | ---------------- |
| en                                  | https://www.ugreen.com/      | US                     | EN               |
| en-US                               | https://www.ugreen.com/      | US                     | EN               |
| en-CA (context verified 2026-11-13) | https://www.ugreen.com/en-ca | CA                     | EN               |

| Advertised-only locales | URL                          | GraphQL country/region | GraphQL language |
| ----------------------- | ---------------------------- | ---------------------- | ---------------- |
| ar                      | https://www.ugreen.com/ar-ae | Not selected           | AR               |
| ar-SA                   | https://www.ugreen.com/ar-sa | Not selected           | AR               |
| de-DE                   | https://www.ugreen.com/de-de | Not selected           | DE               |
| en                      | https://www.ugreen.com/en-ae | Not selected           | EN               |
| en-AU                   | https://www.ugreen.com/en-au | Not selected           | EN               |
| en-DE                   | https://www.ugreen.com/en-de | Not selected           | EN               |
| en-ES                   | https://www.ugreen.com/en-es | Not selected           | EN               |
| en-AT                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-BE                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-BG                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-CH                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-CY                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-CZ                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-DK                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-EE                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-FI                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-HR                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-HU                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-IS                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-LT                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-LU                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-LV                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-MT                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-NO                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-PL                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-PT                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-RO                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-SE                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-SI                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-SK                   | https://www.ugreen.com/en-eu | Not selected           | EN               |
| en-FR                   | https://www.ugreen.com/en-fr | Not selected           | EN               |
| en-GB                   | https://www.ugreen.com/en-gb | Not selected           | EN               |
| en-ID                   | https://www.ugreen.com/en-id | Not selected           | EN               |
| en-IT                   | https://www.ugreen.com/en-it | Not selected           | EN               |
| en-MY                   | https://www.ugreen.com/en-my | Not selected           | EN               |
| en-NL                   | https://www.ugreen.com/en-nl | Not selected           | EN               |
| en-PH                   | https://www.ugreen.com/en-ph | Not selected           | EN               |
| en-SA                   | https://www.ugreen.com/en-sa | Not selected           | EN               |
| en-SG                   | https://www.ugreen.com/en-sg | Not selected           | EN               |
| en-TH                   | https://www.ugreen.com/en-th | Not selected           | EN               |
| en-TR                   | https://www.ugreen.com/en-tr | Not selected           | EN               |
| en-VN                   | https://www.ugreen.com/en-vn | Not selected           | EN               |
| es-ES                   | https://www.ugreen.com/es-es | Not selected           | ES               |
| fr-FR                   | https://www.ugreen.com/fr-fr | Not selected           | FR               |
| id-ID                   | https://www.ugreen.com/id-id | Not selected           | ID               |
| it-IT                   | https://www.ugreen.com/it-it | Not selected           | IT               |
| nl-US                   | https://www.ugreen.com/nl    | Not selected           | NL               |
| nl-NL                   | https://www.ugreen.com/nl-nl | Not selected           | NL               |
| th-TH                   | https://www.ugreen.com/th-th | Not selected           | TH               |
| tr-US                   | https://www.ugreen.com/tr    | Not selected           | TR               |
| tr-TR                   | https://www.ugreen.com/tr-tr | Not selected           | TR               |
| vi-US                   | https://www.ugreen.com/vi    | Not selected           | VI               |
| vi-VN                   | https://www.ugreen.com/vi-vn | Not selected           | VI               |

**Shipping evidence:** Status: unknown. No reliable enumerated shipping-country definition established in the reviewed sources.

**Research notes:** Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Rows in the advertised-only tables are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.

## Ugreen India

Implementation: [`retailers_ugreen::prelude::get_storefront_config("ugreenin")`](../lib/retailers_ugreen/src/ugreenin/mod.rs). Keep this table and the `STOREFRONT_CONFIG` data synchronized.

| Field                  | Value                                                    |
| ---------------------- | -------------------------------------------------------- |
| Retailer code          | `ugreenin`                                               |
| Storefront website     | https://www.ugreenindia.com/                             |
| Shopify shop value     | 990f9c-b7.myshopify.com                                  |
| Storefront GraphQL API | https://990f9c-b7.myshopify.com/api/2026-10/graphql.json |
| Target markets         | India (IN)                                               |
| Shipping countries     | IN                                                       |
|                        |                                                          |
|                        |                                                          |

**Shipping configuration checked 2026-11-13:** Shop configuration retrieved successfully. [Recorded result](research/retailer-shipping-countries-2026-11-13.json).

| Supported locales | URL                          | GraphQL country/region | GraphQL language |
| ----------------- | ---------------------------- | ---------------------- | ---------------- |
| en                | https://www.ugreenindia.com/ | IN                     | EN               |

| Advertised-only locales | URL                        | GraphQL country/region | GraphQL language |
| ----------------------- | -------------------------- | ---------------------- | ---------------- |
| en                      | https://ugreenindia.com/   | Not selected           | EN               |
| hi                      | https://ugreenindia.com/hi | Not selected           | HI               |

**Shipping evidence:** Status: published. India-wide B2C policy; postcode/carrier restrictions apply. Source: https://ugreenindia.com/policies/shipping-policy.

**Research notes:** Store explicitly labels India; Hindi alternate is separately advertised. Source: docs/research/retailer-localization-2026-11-09.json (2026-11-09). Target markets above retain the source-declared storefront markets; they are not an API country enumeration or a shipping-derived market list. GraphQL country/region and GraphQL language are intended collection contexts, not verified API combinations. No catalog-price comparison has been performed for this retailer in this extension. The research verified the GraphQL QueryRoot only; available country/language combinations still need a localization query. Rows in the advertised-only tables are alternate links, not verified locale pages or confirmed extra content requests. Their country contexts remain unselected, and the base Shopify backend has not been established for those URLs.
