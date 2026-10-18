# retailers_minisforum

Integration for the Minisforum retailer.

Each storefront has a sitemap module in `sitemap/v1` that uses the shared
Shopify link rule.

`minisforumau`, `minisforumca`, `minisforumeu`, `minisforumfr`, `minisforumhk`,
`minisforumjp`, `minisforumkr`, `minisforumru`, `minisforumuk` and
`minisforumus` also have an offer details pipeline in `offer_details/v1`. The
other storefronts' offer details are still stubs.

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
- `structuralize()`, so far in `minisforumau`, `minisforumeu`, `minisforumfr`,
  `minisforumhk` and `minisforumjp`, turns the normalized
  product into one variation per variant, with its identifiers, title, price,
  URL and images. `registry::structuralize_for` returns `None` for the other
  storefronts. `minisforumca` stops at `normalize()`.

Those five also have data-backed tests in `offer_details/v1/mod.rs` that run
every saved page through the whole pipeline; they skip when the data folder is
missing.

Each pipeline's `offer_details/v1/stage_files_tests.rs` runs it over saved
product pages in `../data/<storefront>`, next to the monorepo checkout, and
writes each stage's output there. Its `mod` line is commented out, so enable it
to regenerate those files.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.
