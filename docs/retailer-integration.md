# Retailer integration playbook

How to build, or finish, the integration for one retailer storefront: the code
that tells the pipeline where a store's sitemaps are, which of their URLs are
products, and how to turn a product page into structured offer data.

The reference implementation is `ankerau` (`lib/retailers_anker/src/ankerau/`).
Every step below shows the matching ankerau code. Open it next to this document.

## How to use this document

This document is written so an agent can carry it out end to end.

- Read [`AGENTS.md`](../AGENTS.md) first. Its rules on commits, `make fmt`,
  `make ci` and asking before acting apply to everything here.
- Do the steps in order. Each ends with a **Check**. Don't start the next step
  until the check passes.
- When something doesn't match what a step expects, stop and report what you
  found instead of guessing. Examples: a page has a data island this document
  doesn't describe, the retailer code doesn't exist, or the saved pages are
  missing.
- Keep the work for one storefront to one crate, except where a step says
  otherwise.

## 1. Inputs to gather first

| Input               | Where to find it                                                             | ankerau                                  |
| ------------------- | ---------------------------------------------------------------------------- | ---------------------------------------- |
| Retailer slug       | `RetailerCode` in `bodul_shared` (see below)                                 | `ankerau` (`RetailerCode::AnkerAu`)      |
| Crate               | `lib/retailers_<brand>/`                                                     | `lib/retailers_anker/`                   |
| Storefront folder   | `lib/retailers_<brand>/src/<slug>/`                                          | `lib/retailers_anker/src/ankerau/`       |
| Platform            | Page source of a product page                                                | Next.js Pages Router (Shopify behind it) |
| Root sitemap URLs   | `robots.txt` (`Sitemap:` lines) or `/sitemap.xml`                            | `https://www.anker.com/sitemap.xml`      |
| Saved product pages | `../data/<slug>/offer_details/sources/*.html`, next to the monorepo checkout | `../data/ankerau/offer_details/sources/` |
| Saved sitemaps      | `../data/<slug>/sitemaps/sources/<retrieval id>/`                            | `../data/ankerau/sitemaps/sources/`      |

**Retailer slug.** `RetailerCode` is an enum in the `bodul_shared` crate,
published to the `null-labs-hq` registry (source under
`~/.cargo/registry/src/*/bodul_shared-*/src/retailer.rs`). The slug is the
variant's name in lowercase: `AnkerJapanCom` is `ankerjapancom`.
`RetailerCode::try_from("ankerau")` parses it.

**Starting point.** Find out which case applies:

- **A. The storefront exists as a stub.** The folder exists, the registry
  already maps the code, and some functions are `todo!()`. This is the usual
  case. Fill in the bodies, and skip step 3's registry work for functions that
  are already wired.
- **B. The storefront is new.** There's no folder. If `RetailerCode` already
  has the variant, start at step 2. If it doesn't, stop and ask: adding a
  variant means releasing a new `bodul_shared` version and updating every
  workspace that depends on it, which is outside this repository.

**Saved pages and sitemaps.** The data folder is not part of this repository.
Each `<slug>/` in it is its own git repository. If the pages or sitemaps are
missing, get them first:

- `make export-offer-sources` writes the downloaded offer pages from the
  `retailer_sourcing` database to `<slug>/offer_details/sources/`. It needs
  `DATABASE_URL`.
- `make dump-sitemap-sources` downloads the fetched sitemap files from a
  running `retailer_sourcing` to `<slug>/sitemaps/sources/<retrieval id>/`.

If neither is possible, stop and ask for pages. The offer-details steps can't be
done without real pages.

## 2. Architecture overview

### Where the retailer code runs

`retailer_sourcing` runs one feature slice per step. Each slice handles one
command and emits one event, and the next slice reacts to that event:

| #   | Command              | What it does                                  | Retailer hook it calls                                  |
| --- | -------------------- | --------------------------------------------- | ------------------------------------------------------- |
| 1   | `RetrieveSitemap`    | Downloads the root and child sitemaps         | `sitemap_config`, `sitemap_greylisted`, `sitemap_pause` |
| 2   | `ProcessSitemap`     | Builds one sitemap tree from the stored files | `sitemap_config`, `sitemap_greylisted`                  |
| 3   | `ConsolidateSitemap` | Sorts each URL into a kind of page            | `classify_link`                                         |
| 4   | `DiscoverOffers`     | Turns every product URL into an offer         | none                                                    |
| 5   | `DownloadOfferPage`  | Downloads the product page                    | none                                                    |
| 6   | `ExtractOfferPage`   | Pulls the data islands out of the HTML        | `architecture`                                          |
| 7   | `NormalizeOffer`     | Reads the islands into a typed document       | `normalize`                                             |
| 8   | `StructuralizeOffer` | Turns that into product variations            | `structuralize`                                         |

The same hooks run offline in the development tools (section 5, step 10), on
files in the data folder.

### How a call reaches the storefront

```
services/retailer_sourcing
  sitemap_discovery/mod.rs   struct Retailers   (sitemap hooks)
  offer_discovery/mod.rs     struct Retailers   (extract, normalize, structuralize; panics caught)
        │
        ▼
lib/retailer_data_extractor/src/lib.rs          facade: <hook>_for(code, …)
  tries retailers_anker, then retailers_mi, retailers_minisforum, retailers_ugreen,
  with `or_else`; the first crate that knows the code answers
        │
        ▼
lib/retailers_<brand>/src/registry.rs           <hook>_for: one `match RetailerCode` arm per storefront
        │
        ▼
lib/retailers_<brand>/src/<slug>/prelude.rs     re-exports the v1 functions as <hook>_v1
        │
        ▼
lib/retailers_<brand>/src/<slug>/{sitemap,offer_details}/v1/   the implementation
```

The service and the facade never name a storefront. A storefront is reachable
once its crate's registry has an arm for it.

### Shared crates

| Crate                     | What a storefront uses from it                                                                                                                                      |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `bodul_shared`            | `RetailerCode`, `SitemapConfig { sitemap_url }`, `NormalizedData`, `link::LinkKind`                                                                                 |
| `html_extractor`          | `Architecture`, `collection`, `particle`, `trash`, `Attr`; `Backend` in tests; its `serde_json`                                                                     |
| `retailer_sauce`          | `shopify::shopify_from_location`, `structuralized::StructuralizedData`, `availability::Availability`, `item_condition::ItemCondition`, `identifier::{Gtin, Sku, …}` |
| `money` (`lib/money`)     | `Money::from_raw(amount, currency)`, stored as minor units                                                                                                          |
| `sitemap` (`lib/sitemap`) | Not called by storefronts. It builds the tree using the storefront's `sitemap_greylisted`                                                                           |

## 3. Storefront module layout

```
lib/retailers_anker/src/ankerau/
├── mod.rs                      private stage modules plus `pub mod prelude`
├── prelude.rs                  the storefront's only public surface: v1 functions renamed to *_v1
├── sitemap/
│   ├── mod.rs                  //! Sitemap for this retailer.  pub mod v1;
│   └── v1/mod.rs               sitemap_config, sitemap_greylisted, sitemap_pause, classify_link
├── offer_details/
│   ├── mod.rs                  //! Offer details for this retailer.  pub mod v1;
│   └── v1/
│       ├── mod.rs              architecture, normalize, structuralize, data-backed tests
│       ├── raw.rs              strict typed form of the extracted JSON (root document)
│       ├── raw/ldjsonitem.rs   the ld+json island
│       ├── raw/nextdata.rs     the __NEXT_DATA__ island
│       ├── raw/nextdata/product.rs
│       ├── normalized.rs       the normalized document and NormalizeError
│       ├── structuralize.rs    the structuralized document and StructuralizeError
│       └── stage_files_tests.rs  optional, see step 9
└── catalog_offers/
    ├── mod.rs                  pub mod v1;
    └── v1/mod.rs               //! Version 1 of the catalog offers.  (placeholder, no code yet)
```

Crate-level code shared by a brand's storefronts sits next to them:
`retailers_anker/src/lib.rs` (`anker_from_location`) and `src/shared.rs`
(re-exports the identifier newtypes structuralize uses from
`retailer_sauce::identifier`).

Rules:

- Only `prelude` is public from a storefront. Stage modules are private, and
  each stage folder's `mod.rs` is a one-line doc comment plus `pub mod v1;`.
- Implementation lives in `v1`. The prelude re-exports it with a `_v1`
  suffix, so a future `v2` can sit next to it and the registry chooses which
  one to call.
- The ankerau prelude, which is the exact shape to copy:

  ```rust
  pub use super::offer_details::v1::{
      architecture as offer_detail_architecture_v1, normalize as offer_detail_normalize_v1,
      structuralize as offer_detail_structuralize_v1,
  };
  pub use super::sitemap::v1::{
      classify_link as classify_link_v1, sitemap_config as sitemap_config_v1,
      sitemap_greylisted as sitemap_greylisted_v1, sitemap_pause as sitemap_pause_v1,
  };
  ```

## 4. Hook signatures

Every storefront implements these in `v1`. The left column is the `v1` name, the
middle one the prelude name the registry calls.

| `v1` function                                          | Prelude name                    | Returns                                        |
| ------------------------------------------------------ | ------------------------------- | ---------------------------------------------- |
| `sitemap::v1::sitemap_config()`                        | `sitemap_config_v1`             | `SitemapConfig`                                |
| `sitemap::v1::sitemap_greylisted(url: &str)`           | `sitemap_greylisted_v1`         | `bool`: `true` downloads the child sitemap     |
| `sitemap::v1::sitemap_pause()`                         | `sitemap_pause_v1`              | `Duration` between groups of 3 downloads       |
| `sitemap::v1::classify_link(url, source, image_count)` | `classify_link_v1`              | `LinkKind`                                     |
| `offer_details::v1::architecture()`                    | `offer_detail_architecture_v1`  | `html_extractor::io::Architecture`             |
| `offer_details::v1::normalize(json: &str)`             | `offer_detail_normalize_v1`     | `Result<bodul_shared::NormalizedData, String>` |
| `offer_details::v1::structuralize(json: &str)`         | `offer_detail_structuralize_v1` | `Result<StructuralizedData, String>`           |

The registry functions wrap each in `Option`, `None` meaning "not my
storefront". See `lib/retailers_anker/src/registry.rs`.

## 5. Implementation steps

### Step 1. Module skeleton

Skip what exists already (case A).

1. Create the tree from section 3 with `mod.rs`, `prelude.rs`, and the three
   stage folders. Copy the one-line doc comments.
2. Add `mod <slug>;` to the crate's `src/lib.rs`, in alphabetical order with
   the others.
3. Leave `catalog_offers/v1/mod.rs` as the placeholder doc comment.

**Check:** `cargo build -p retailers_<brand>` in `lib/`. Unused-code warnings
are expected until step 3.

### Step 2. Sitemap rules (`sitemap/v1/mod.rs`)

ankerau, `lib/retailers_anker/src/ankerau/sitemap/v1/mod.rs`:

```rust
pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.anker.com/sitemap.xml".to_string()],
    }
}

/// Whether the crawl downloads the child sitemap at `url`. Root sitemaps are
/// always downloaded; drop one from [`sitemap_config`] instead.
pub fn sitemap_greylisted(_url: &str) -> bool {
    true
}

/// How long the crawl pauses between groups of sitemap downloads.
pub fn sitemap_pause() -> Duration {
    Duration::from_secs(1)
}

pub fn classify_link(url: &str, source: &str, _image_count: usize) -> LinkKind {
    from_location(url, source)
}

pub fn from_location(url: &str, source: &str) -> LinkKind {
    let path = url.to_lowercase();

    if !path.starts_with("https://www.anker.com/au/") {
        return LinkKind::NotInterested;
    }

    match source {
        "https://www.anker.com/sitemap-0.xml" => LinkKind::NotInterested,
        "https://www.anker.com/server-sitemap-index-pages.xml" => LinkKind::NotInterested,
        "https://www.anker.com/server-sitemap-index-products.xml" => LinkKind::Product,
        "https://www.anker.com/server-sitemap-index-collections.xml" => LinkKind::Catalog,
        "https://www.anker.com/server-sitemap-index-blog.xml" => LinkKind::Content,
        _ => anker_from_location(&path),
    }
}
```

Rules:

- **`sitemap_config`**: the root sitemaps only. The crawl follows child
  sitemaps by itself.
- **`sitemap_greylisted`**: return `false` for child sitemaps that should not
  be downloaded. Most Shopify stores skip their metaobject sitemaps:

  ```rust
  pub fn sitemap_greylisted(url: &str) -> bool {
      // we are skipping any sitemap file having `metaobject` in filename
      !url.contains("metaobject")
  }
  ```

- **`sitemap_pause`**: 1 second by default. Use 5 seconds for a store that
  rate-limits or drops requests (`ugreencom`, `ugreeneu`, `ugreennaseu` do).
  The crawl pauses after every 3 downloads.
- **`classify_link`** sorts a URL into a `LinkKind`:

| `LinkKind`      | Meaning                                                     |
| --------------- | ----------------------------------------------------------- |
| `Product`       | A product page. Only these go on to offer discovery.        |
| `Catalog`       | A collection or category listing                            |
| `Content`       | Blog posts, articles, information pages                     |
| `NotInterested` | Pages to ignore, including other regions on a shared domain |
| `Unknown`       | Anything no rule matched. Aim for none on real data.        |

  - `url` is the page URL, `source` the sitemap file that listed it (or
    `"main"` when unknown), `image_count` the number of images the sitemap
    lists for it. Use `source` or `image_count` only when the URL alone can't
    tell. ankerau uses `source` because Anker lists each kind in its own
    sitemap.
  - **Region guard first.** When several regions share a domain, return
    `NotInterested` for every URL outside this storefront's prefix before any
    other rule. ankerau's guard is the `starts_with("https://www.anker.com/au/")`
    check.
  - Compare in lowercase.
  - On Shopify, fall back to `retailer_sauce::shopify::shopify_from_location`
    (`/products/`, `/collections/`, `/pages/`, `/blogs/`) after the
    storefront-specific exclusions. See
    `lib/retailers_minisforum/src/minisforumau/sitemap/v1/mod.rs` for that
    pattern.

Tests in the same file, at least one per rule: one test per `LinkKind` with real
URLs from the saved sitemaps, the region guard, and case-insensitivity. Use URLs
from `../data/<slug>/sitemaps/` rather than invented ones.

**Check:** `cargo test -p retailers_<brand> <slug>::sitemap` passes.

### Step 3. Registry wiring

For each hook in section 4 that the storefront implements:

1. Re-export it in the storefront's `prelude.rs` under its `_v1` name.
2. Add an arm to the matching `*_for` function in
   `lib/retailers_<brand>/src/registry.rs`, in the same position as the other
   storefronts:

   ```rust
   RetailerCode::AnkerAu => Some(ankerau::prelude::sitemap_config_v1()),
   ```

3. In case A, check the existing arms point at your functions and that no
   other arm for the code returns something else.

The facade in `lib/retailer_data_extractor/src/lib.rs` already chains every
crate for every hook except one: **structuralize**. Only `retailers_anker` and
`retailers_minisforum` have `structuralize_for` today, and the facade calls only
those two. When a storefront in another crate gets a structuralizer:

- add `structuralize_for` to that crate's `registry.rs` and `prelude.rs`, with
  the same signature as Anker's (`retailers_minisforum` has one arm per
  storefront that has a structuralizer, and `None` for the rest);
- chain it in `retailer_data_extractor::structuralize_for` with `or_else`,
  like the other facade functions, and update that function's doc comment.

**Check:** `cargo test -p retailer_data_extractor` passes. Every hook you wired
returns `Some` for the storefront's code through the facade; the facade's own
tests in `lib/retailer_data_extractor/src/lib.rs` show how to assert that.

### Step 4. Survey the saved pages

Before writing any schema, find out what the pages contain:

- Which data islands every product page carries: `script[type="application/ld+json"]`,
  `script#__NEXT_DATA__` (Next.js Pages Router), the App Router flight
  payload, a Liquid `product | json` script, meta tags.
- Which keys each island has, and which are present on every page, some
  pages, or with `null`.
- Where the facts are: title, SKU, GTIN or barcode, price and currency, list
  price, availability, variants, images, canonical URL.
- Where two islands disagree. On ankerau, the ld+json offer carries the
  effective price and `__NEXT_DATA__` the list price, so both are read.

Write what you found into the module doc comments, as ankerau does. Its
`offer_details/v1/mod.rs` starts with:

```rust
//! `anker.com/au` is a Next.js Pages Router storefront. Reads the page's two data islands
//! (`ld+json` and `__NEXT_DATA__`); the rendered DOM is not claimed.
```

**Check:** you can name, for every fact above, the island and key it comes from,
and that holds for every saved page.

### Step 5. Extraction (`architecture()`)

ankerau, `offer_details/v1/mod.rs`:

```rust
pub fn architecture() -> Architecture {
    Architecture::new(vec![
        // Must run first: `trash` detaches its match, and svg elements sit inside body, so
        // anything claiming body ahead of this would carry them off first.
        trash(r#"svg"#),
        collection(
            r#"script[type="application/ld+json"]"#,
            "ld-json",
            vec![particle(
                "script",
                "",
                // `Attr::all()` alongside the JSON read: a stray attribute appearing on the tag
                // fails the strict raw schema instead of disappearing silently.
                vec![Attr::all(), Attr::json("", "data")],
            )],
        ),
        particle(
            "script#__NEXT_DATA__",
            "next_data",
            vec![Attr::all(), Attr::json("", "data")],
        ),
    ])
}
```

Rules:

- Order is execution order. Put `trash(r#"svg"#)` first.
- Use `collection` for an island that can repeat (ld+json) and `particle` for
  one that appears once.
- Pair every JSON read with `Attr::all()`, so an unexpected attribute fails
  the raw schema.
- The structure names (`"ld-json"`, `"next_data"`) become the top-level keys
  of the extracted JSON. The raw schema must use exactly those names.
- Platform notes:
  - Next.js Pages Router: ld+json plus `script#__NEXT_DATA__` (ankerau).
  - Shopify App Router (Next.js App Router): start from
    `retailers_anker::shopfiy::shopify_app_router_architecture()`.
  - Shopify Liquid themes: ld+json plus whatever script renders
    `product | json`; for example `minisforumuk` reads
    `script#xcotton_pp_variants` and a PayPal meta tag for the currency.
  - No data island at all (Cafe24, `ankerkr`): the architecture only trashes
    `svg`, `normalize` checks the extraction is empty and returns
    `NormalizedData::just_document_json(None)`, and the registry's
    `structuralize_for` arm returns
    `Some(Ok(StructuralizedData::just_document_json(None)))`. Stop and ask
    before doing this, because it means the storefront produces no offers.

**Check:** run the extraction over every saved page and look at a few outputs:
`make process-offer-details RETAILERS=<slug>` writes
`../data/<slug>/offer_details/extracted/*.json` (normalize can still fail at
this point). Every page has every island.

### Step 6. Raw schema (`raw.rs`, `raw/*.rs`)

A typed, strict mirror of the extracted JSON. Its job is to fail loudly on
anything the survey didn't see.

ankerau, `raw.rs`:

```rust
//! Typed raw document for `ankerau`, deserialized from the extracted JSON. Every struct in this
//! tree is `deny_unknown_fields`, so a key the corpus survey did not see fails a document loudly
//! instead of being dropped silently.

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnkerAuRawDocument {
    // Extracted key is `ld-json`, hyphenated, because that is the architecture's structure name.
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonItem>,

    pub next_data: nextdata::NextDataScript,
}
```

Rules:

- `#[serde(deny_unknown_fields)]` on every struct, without exception.
- Model ld+json as an internally tagged enum on `@type`, so each variant keeps
  `deny_unknown_fields` and an unknown `@type` fails
  (`raw/ldjsonitem.rs`):

  ```rust
  #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
  #[serde(tag = "@type")]
  pub enum LdJson {
      Product(Product),
      Corporation(Corporation),
      BreadcrumbList(BreadcrumbList),
      #[serde(rename = "FAQPage")]
      FaqPage(FaqPage),
  }
  ```

- Keep wire spellings. Use `rename_all = "camelCase"` where the island is
  camelCase, and per-field `rename` for exceptions (ankerau's `OpenGraph`
  keeps `site_name`).
- Large regions that carry no offer fact: name the field and type it as
  `serde_json::Value` (use `html_extractor::serde_json`), with a doc comment
  saying why. ankerau does this for `buildProps`, `metafields` and `shop`.
- `Option<T>` for keys that are present but `null` on some pages;
  `#[serde(default)]` for keys missing on some pages. Say which in a doc
  comment, as ankerau does for `legal_guarantee`.
- Document units and sentinels where they're not obvious (ankerau: `weight`
  in kilograms, `list_price` `0.0` meaning no markdown).
- Name the root `<Slug>RawDocument`, for example `AnkerAuRawDocument`.

Tests in each raw file: both islands are required; the hyphenated key is the one
accepted; an unknown field fails; an unknown `@type` fails; an unexpected script
attribute fails. Copy the ankerau tests in `raw.rs` and `raw/ldjsonitem.rs` as
templates.

**Check:** every saved page's extracted JSON deserializes. A page that fails
tells you a key to model, not a key to drop.

### Step 7. Normalized document (`normalized.rs`) and `normalize()`

Expresses the facts in the system's vocabulary: typed money and instants,
storefront names mapped to plain ones, metadata the system doesn't need dropped.

ankerau, `offer_details/v1/mod.rs`:

```rust
pub fn normalize(json: &str) -> Result<bodul_shared::NormalizedData, String> {
    let raw: AnkerAuRawDocument = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let normalized = AnkerAuNormalizedDocument::from_raw(raw)
        .map_err(|error: NormalizeError| error.to_string())?;

    let document_json = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;

    Ok(bodul_shared::NormalizedData::just_document_json(
        document_json,
    ))
}
```

Rules:

- A `thiserror` enum `NormalizeError` with one variant per way a page can be
  unreadable, each naming the field or key involved
  (`normalized.rs`: `MissingProductIsland`, `NotMoney { field, .. }`, …).
- The document is `<Slug>NormalizedDocument` with `from_raw(raw) ->
  Result<Self, NormalizeError>`, deriving `Serialize` and `Deserialize`
  (structuralize reads it back).
- Money through one helper, so every failure names its field:

  ```rust
  fn money(amount: f64, currency: &str, field: &str) -> Result<Money, NormalizeError> {
      Money::from_raw(amount, currency).map_err(|source| NormalizeError::NotMoney {
          field: field.to_string(),
          source,
      })
  }
  ```

- When an island omits the currency, use a constant with a comment saying
  where the currency comes from:
  `// Variant prices omit currency; AUD is supplied by this storefront's scope.`
- When an island can repeat, pick deterministically and say so
  (`// First island wins; no document in the subset carries two.`). A missing
  required island is an error, not a default.
- Map sentinels to `Option`: ankerau's `compare_at_price` is `None` when
  `list_price <= 0.0`.
- Data you choose not to normalize: leave the type commented out with the
  reason, rather than deleting it. ankerau keeps `FeatureBlock`, `Download`
  and others this way ("the panels are marketing rather than information").
- Reuse `retailer_sauce` for schema.org values:
  `ItemCondition::from_schema_org`, `Availability::from_schema_org`.
- `normalize()` returns `NormalizedData::just_document_json`. Leave
  `recommendation_offers` and `upsell_offers` empty unless asked.

Tests: one per derived field and per error variant. ankerau's `normalized.rs`
tests are the template (first island wins, zero list price, bad `publishedAt`,
variant currency, `NotMoney` naming the wire key).

**Check:** `cargo test -p retailers_<brand> <slug>` passes, and `make
process-offer-details RETAILERS=<slug>` writes a `normalized/` file for every
page.

### Step 8. Structuralized document (`structuralize.rs`) and `structuralize()`

Turns the normalized document into the shape the rest of the system consumes:
one entry per product variation.

ankerau, `structuralize.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnkerAuDocument {
    pub variations: Vec<ProductVariation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation.
    pub title: String,
    /// The variation's own price, before any discount code.
    pub price: Money,
    pub url: String,
    /// The product's images, from the `ld+json` island.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    pub gtin: Option<Gtin>,
    pub sku: Sku,
    pub shopify_product_gid: ShopifyProductGid,
    pub shopify_product_variation_gid: ShopifyProductVariationGid,
    pub shopify_handle: ShopifyHandle,
}
```

Rules:

- One `ProductVariation` per variant, carrying the variant's own price.
- Identifiers go through validating newtypes (`TryFrom<String>`) so a bad
  GTIN or SKU fails the page. They live in `retailer_sauce::identifier`;
  use them from there rather than copying them.
- Variation URL: the canonical URL for a single variant, otherwise
  `canonical?variant=<numeric id>`. Don't use the ld+json offer URL: it may
  carry tracking parameters and a different selected variant.
- `structuralize()` mirrors `normalize()`: parse the normalized document,
  build, serialize, return `StructuralizedData::just_document_json`.
- Outside Anker and Minisforum, wire `structuralize_for` as described in
  step 3; in `retailers_minisforum`, add an arm to its `structuralize_for`.

Tests: the exact output shape for a fixture, identifier failures, per-variant
URLs, the single-variant URL. ankerau's `structuralize.rs` tests are the
template.

**Check:** `make process-offer-details RETAILERS=<slug>` writes a
`structurized/` file for every page, and the tool reports no failures.

### Step 9. Data-backed tests

Unit tests use small inline fixtures. Add the three corpus tests ankerau has at
the bottom of `offer_details/v1/mod.rs`, adjusted to the storefront:

```rust
/// Saved product pages, kept outside the repository next to the monorepo checkout.
const DATA_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../data/ankerau/offer_details/sources"
);
```

- `pages()` returns `None` and prints `skipping: … not found` when the folder
  is missing, so CI without the data still passes. It asserts there is at
  least one `.html` page.
- `extracts_product_and_next_data_from_every_page`: every island is present,
  and the key facts (SKU, name, currency, price) are there.
- `backends_agree_on_every_page`: every `html_extractor::Backend` gives the
  same JSON as `Backend::Scraper`.
- `normalizes_and_structuralizes_every_page`: runs the whole pipeline and
  collects every failure before asserting, so one run shows all of them.

`stage_files_tests.rs`, which writes each stage's output to the data folder, is
optional. ankerau keeps it commented out in `mod.rs`; `make
process-offer-details` does the same job.

**Check:** with the data folder present, `cargo test -p retailers_<brand>
<slug>` runs these tests (no `skipping:` line) and they pass.

### Step 10. Run the pipeline on real data

From the monorepo root (`DATA_DIR` defaults to `../data`):

```bash
make process-offer-details RETAILERS=<slug>   # offer pages: extracted/, normalized/, structurized/
make process-sitemap-sources                   # sitemaps/sources/<id>/ -> sitemaps/processed/<id>.json
make consolidate-sitemap-sources               # sitemaps/processed/ -> sitemaps/consolidate/<id>.json
```

What success looks like:

- `process-offer-details` reports no failed pages for the storefront.
- `process-sitemap-sources` prints `<slug> <id>: N URLs from F files, 0 files
  unused` with N greater than 0 and no `left out of the tree` lines. Files
  unused should be explained by `sitemap_greylisted`.
- `consolidate-sitemap-sources` prints `<slug> <id>: P products, …, U
  unknown` with P greater than 0. Look at every `unknown` URL in
  `sitemaps/consolidate/<id>.json`: add a rule or a test for each group, or
  explain why it stays unknown.

**Check:** all three runs meet the criteria above.

### Step 11. Documentation

- Update the crate's `README.md` if it lists storefronts or what they
  produce.
- Update the root `README.md` only if crates were added or removed.

**Check:** `make ci` in `lib/` (see section 7 for known failures).

## 6. Conventions

- **Errors:** typed `thiserror` enums inside the storefront;
  `Result<_, String>` at the `v1` boundary via `map_err(|e| e.to_string())`.
- **No panics** outside tests: no `unwrap`, `expect`, `panic!` or `todo!()`
  left on a finished storefront's paths. The service catches panics, but
  `process_offer_details` doesn't, and a panic there stops the whole run.
- **Naming:** documents are `<Slug>RawDocument`, `<Slug>NormalizedDocument`,
  `<Slug>Document`; error enums `NormalizeError`, `StructuralizeError`.
- **Comments:** record survey findings and decisions next to the code that
  depends on them, the way the ankerau quotes above do.
- **Dependencies:** a storefront needs only what its crate already has. If a
  new dependency is needed, keep `[dependencies]` sorted and run
  `make audit` (see `AGENTS.md`).
- **Before finishing:** `make fmt`, then `make ci` in `lib/`. Commit one
  change per commit, without signature lines.

## 7. Pitfalls and known gaps

Don't copy these:

- Eight Anker sitemap tests fail today, so `make ci` in `lib/` fails until
  they're fixed. Make sure your storefront's tests are not among them:
  - `unmatched_source_falls_back_to_path` in `ankerau`, `ankerca`, `ankerde`,
    `ankernz`, `ankerpl` and `ankervn`. Its comment says an unmatched source
    skips the region guard, but the code checks the guard first.
  - `ankereu::sitemap::v1::tests::classifies_normal_sitemap_urls`
  - `ankerfr::sitemap::v1::tests::classifies_grouped_sitemap_examples`
- `ankerau/offer_details/v1/structuralize.rs.bak` is a leftover older version.
  Ignore it.
- Stub storefronts return `Some(todo!())` from the registry, so calling them
  panics instead of returning `None`.
- Folder names differ between tools: `export-sitemaps` writes
  `<slug>/sitemap/{sources,processed,consolidated}`, while the development
  tools use `<slug>/sitemaps/{sources,processed,consolidate}`. Offer stages are
  `extracted/`, `normalized/`, `structurized/`. Use the development tools'
  layout.
- A missing child sitemap leaves its whole root out of the processed tree.
  A sitemap run with 0 URLs usually means a download failed, not a rule.
- `LinkKind::as_str()` spells `not_intersted`. Don't match on that string.

## 8. Definition of done

- [ ] Every hook in section 4 is implemented for the storefront, with no
      `todo!()` left.
- [ ] Every hook has a registry arm and a prelude export; structuralize is
      chained in the facade if the crate is not Anker.
- [ ] Unit tests for sitemap rules, raw schema, normalization and
      structuralization pass.
- [ ] Data-backed tests run against the saved pages and pass.
- [ ] `make process-offer-details RETAILERS=<slug>` reports no failures.
- [ ] Sitemap tools give a non-empty tree, more than 0 products, and every
      `unknown` URL is handled or explained.
- [ ] `make fmt` leaves no changes and `make ci` in `lib/` passes apart from
      the known Anker sitemap failures.
- [ ] Crate README updated if needed.
