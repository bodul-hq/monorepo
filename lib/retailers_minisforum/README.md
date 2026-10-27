# retailers_minisforum

Integration for the Minisforum retailer.

Each storefront has a sitemap module in `sitemap/v1` that uses the shared
Shopify link rule.

`minisforumau`, `minisforumca`, `minisforumeu`, `minisforumfr`, `minisforumhk`,
`minisforumjp`, `minisforumkr`, `minisforumru`, `minisforumuk` and
`minisforumus` implement extraction, normalization and structuralization in
`offer_details/v1`. All ten storefronts have all three stages registered.

- `architecture()` extracts the page's data islands. The storefronts run
  different Shopify themes, so each reads its own set:
  - `minisforumru`: the Apple Pay shop capabilities (for the currency),
    Shopify's analytics `meta`, the `h1` title, the variant picker's legends
    and variant records, and the gallery images. The theme renders no
    `Product` ld+json.
  - `minisforumkr`: the ld+json islands, the analytics `meta`, the variant
    picker and its selected variant, and the gallery images. Only the
    selected variant has a full record.
  - `minisforumau`, `minisforumca`, `minisforumeu`, `minisforumfr`,
    `minisforumjp`, `minisforumuk` and `minisforumus`: the ld+json islands and
    the full Liquid product record an app renders in
    `script#xcotton_pp_variants`. All but `us` read their currency from the
    PayPal metadata tag.
  - `minisforumhk`: the PayPal metadata tag, the ld+json islands, and the
    theme's variant list, selected variant and `seal_product`, plus the
    variant picker's option labels. It has no `xcotton_pp_variants` record.
- `normalize()` reads them into a strict raw schema, which rejects unknown
  fields, and then into one normalized product. It fails when two islands
  disagree on the product or its variants.
- `structuralize()` turns the normalized product into one variation per variant,
  with its identifiers, title, price, URL and images. `registry::structuralize_for`
  dispatches to the implementation for each of the ten storefronts above and
  returns `None` for codes outside this crate.

The AU, EU, FR, HK and JP pipelines also have data-backed tests in
`offer_details/v1/mod.rs` that run every saved page through the whole pipeline;
they skip when the data folder is missing. The facade's `structuralization_report`
integration test checks registration for CA, KR, RU, UK and US even without the
data checkout, and structuralizes their saved normalized documents when present.

Each pipeline's `offer_details/v1/stage_files_tests.rs` runs it over saved
product pages in `../data/<storefront>`, next to the monorepo checkout, and
writes each stage's output there. Its `mod` line is commented out, so enable it
to regenerate those files.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

The October 2026 FR/JP organization metadata, French service/SEO graph and
breadcrumbs, HK FAQs, Japanese website/SEO product, and US webpage islands are
explicitly modelled. Supplemental marketing islands do not replace the checked
Liquid variant data. US SEO names may differ from the product title; identity
checks still require the product handle in both product and offer URLs.
Compact metadata fixtures in each affected storefront's `raw/current-metadata.json`
retain the saved wire shapes with shortened descriptive text. They run in normal
CI without the external data checkout.

Canonical comparison prices, availability, item condition, and descriptions are
documented in [offer facts source coverage](../../docs/offer-facts-source-coverage.md).
