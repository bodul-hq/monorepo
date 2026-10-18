# retailers_anker

Integration for the Anker retailer.

Each storefront has an offer details pipeline in `offer_details/v1`:

- `architecture()` extracts the page's data islands: the `ld+json` scripts
  plus, depending on the storefront, the Next.js `__NEXT_DATA__` script, the
  App Router flight payload, or the Liquid `ProductJson` record.
- `normalize()` reads them into a strict raw schema, which rejects unknown
  fields, and then into a normalized document.
- `structuralize()` turns the normalized document into the structuralized one.

`ankerkr` runs on Cafe24, has no data island to read, and produces no document
yet.

The AU and US normalizers skip product pages whose URLs identify another known
Anker storefront; older crawls placed regional pages in these folders. Missing
payloads on native pages still fail. German products can omit their featured
image, and EU/Polish specification rows can have text with no separate label
(`name: null`). Japanese PowerBag records without Product JSON-LD retain their
Liquid variants in JPY and use their storefront handle for offer URLs.
Canadian and German structuralized variations retain Shopify identifiers when
their merchant SKU is blank, with `sku: null` instead of rejecting the page.

The `ankerau` tests run the pipeline over saved product pages in
`../data/ankerau`, next to the monorepo checkout, and skip when that folder is
missing. They select `au-products-*.html`, matching the URL-path filenames
written by the export tool, because older crawls also saved other regions in
that folder. The other storefronts are covered by unit tests. The facade's
`normalization_report` integration test also checks saved extracted pages from
the storefronts with previously reported normalization failures.

`ankerau` is the reference implementation in the
[retailer integration playbook](../../docs/retailer-integration.md), which
describes how to add or finish a storefront.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.
