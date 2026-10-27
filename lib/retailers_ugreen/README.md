# retailers_ugreen

Integration for the Ugreen retailer.

The EU, France, India, NAS Australia, NAS Canada, NAS Germany and NAS Spain
structuralizers are registered alongside the existing `ugreencom`, Japan and NAS
stages. They emit their variant documents through the facade and
`structuralize-offer-details`.

`ugreenca`, `ugreende`, `ugreennasjp`, `ugreenkr`, `ugreennl`, `ugreenuk` and
`ugreenus` have extraction, normalization and structuralization pipelines in
`offer_details/v1`, with all three stages registered.

`ugreennas` implements extraction, normalization and structuralization in
`offer_details/v1`, with one output variation per Shopify variant. It reads the
Apple Pay currency, Shopify analytics, EcomSend variant and title records,
canonical link and main product gallery. Strict schemas and cross-island checks
reject missing data, duplicate variants and disagreements. Prices remain exact
USD cents, including preorder deposits; availability, compare-at prices and
optional SKU/GTIN identifiers are preserved.

The NAS storefront now serves `ai.ugreen.com`. Its two refurbished products
canonicalize to new-product pages: normalization preserves that canonical URL,
while offer links use the actual Shopify handle so they select the correct
product and variant. Product JSON-LD is not required because some pages omit it.

Run `cargo test -p retailers_ugreen ugreennas::offer_details` for the fixture
and saved-page tests. The corpus tests read
`../data/ugreennas/offer_details/sources` next to the monorepo checkout and skip
when it is absent. From the root, run `make process-offer-details
RETAILERS=ugreennas` to write all three stages.

The extraction and normalization stages read storefront-specific data:

- `architecture()` extracts the page's data islands. The storefronts run
  different Shopify themes, so each reads its own set:
  - `ugreenca`: the ld+json islands, the Apple Pay shop capabilities (for the
    currency), Shopify's analytics `meta`, the title, the featured-product
    attribute, the theme's variant and option records, and the gallery.
  - `ugreende`: the analytics `meta`, the title, the price block, the main
    product form, the variant picker and the gallery. It reads no ld+json,
    because on some pages the theme replaces it with invalid JSON.
  - `ugreennasjp`: the ld+json islands, including judge.me's reviews, and the
    analytics `meta`.
  - `ugreennaseu`, `ugreennasfr`, `ugreennasit` and `ugreennasuk`, which
    share one Dawn theme: PayPal currency, analytics `meta`, Glood's complete
    Liquid product, regional `og:url`, the title, selected variant, optional
    picker, displayed prices and gallery. EU and UK also read the back-in-stock
    variant records. They read no ld+json because some islands are malformed.
  - `ugreenkr`: the selling JSON-LD, analytics `meta`, canonical URL, and the
    main product title, displayed prices and gallery. Shopify's hundredths
    are converted exactly to whole won.
  - `ugreennl`, `ugreenuk`, `ugreenus`: the Horizon main product section,
    analytics `meta`, and valid `Product`/`ProductGroup` JSON-LD. Every
    published variant offer retains its price, availability and available
    identifiers. Review-only products do not supply selling offers. The
    custom US price specifications retain their labels separately from the
    displayed prices, including inconsistent labels in the saved source. US custom offers whose
    price conflicts with the page carry `offer_price_mismatch: true`; the
    analytics base price and the published `offer_price` are both retained.
- `normalize()` reads them into a strict raw schema, which rejects unknown
  fields, and then into one normalized product. It fails when two islands
  disagree on the product or its variants.

The existing `offer_details/v1/stage_files_tests.rs` modules run their pipelines
over saved product pages in `../data/<storefront>`, next to the monorepo
checkout, and writes each stage's output there. Its `mod` line is commented out,
so enable it to regenerate those files.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

The KR, NL, UK and US pipelines have read-only corpus tests in
`offer_details/v1/tests.rs`. They extract and normalize every saved HTML page in
`../data/<storefront>/offer_details/sources`, compare all extractor backends,
and check normalized JSON round trips. Missing external data skips those tests;
an empty directory or a failing page fails the test. Inline fixtures run without
the data checkout. Run them with `cargo test -p retailers_ugreen`; `make ci`
also runs formatting and lint checks. All four pipelines have registered
structuralizers that emit validated offer variations. The facade's
`structuralization_report` integration test checks their registration even
without the data checkout and structuralizes saved normalized documents when
present; it also covers `ugreenca`, `ugreende` and `ugreennasjp`.

The EU, France, Italy and UK NAS pipelines implement all three offer stages.
Strict raw models reject unknown fields. Normalization reads Glood's embedded
product JSON without executing JavaScript and checks its identity, variants,
SKUs and prices against analytics, the picker and back-in-stock records. Glood
also supplies the French HDMI cable's two variants when no picker is rendered.
Prices are exact EUR or GBP minor units; displayed prices are parsed with the
storefront's decimal and grouping separators. The main heading supplies the
product title, which can differ from titles in the scripts.

Structuralization emits each variant's validated identifiers, title, price,
compare-at price, availability, URL and images. A variant image precedes the
remaining gallery images. Regional `og:url` preserves EU locale paths and avoids
canonical links that point at another product or store. EU validation accepts
all 18 locale prefixes in the saved pages, including `en-pt`, `pt-pt` and
`sv-se`, while checking the store host and product handle. Only multi-variant
links append `?variant=<id>`. Missing SKU/GTIN values remain optional;
compare-at prices at or below the selling price become `None`.
Purchase-protection pages normalize but emit no offer variations.

Run `cargo test -p retailers_ugreen -- ugreennaseu::offer_details
ugreennasfr::offer_details ugreennasit::offer_details
ugreennasuk::offer_details` for fixtures and all saved-page checks, including
parity across extractor backends. The EU corpus contains 399 pages across 19
regional URL paths; missing external data skips corpus tests, while an empty
corpus fails. From the root, run `make process-offer-details
RETAILERS="ugreennaseu ugreennasfr ugreennasit ugreennasuk"` to write extracted,
normalized and structurized files.

Current India contact metadata may omit `telephone`. For NAS EU/FR/IT/UK,
an empty extracted gallery falls back to the same product's ordered Liquid
images in Glood, after checking its identity against analytics. URLs are validated
and deduplicated; an empty gallery in both sources still fails.

`offer_download_url` resolves only the verified retired DE/UK
`https://ai-<region>.ugreen.com/products/dxp4800-pro` URLs to
`/products/ugreen-nasync-dxp4800-pro-4-bay-nas`. Both replacement pages returned
HTTP 200 on 2026-10-26 while the short URLs returned 404. All other URLs keep
normal HTTP failure handling. Sourcing uses this resolution for already queued
commands as well as new downloads, preserving original discovery IDs and URLs.

The NAS AU/CA/ES/EU/FR/IT/UK extractors also recognize current
`ug-product-gallery` image slides, as DE already did. Video posters, discount
badges, thumbnails and recommendation sections are excluded. This keeps new
extractions complete while the fallback handles previously saved extractions.
