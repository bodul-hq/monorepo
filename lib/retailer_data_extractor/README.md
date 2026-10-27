# retailer_data_extractor

Extracting data from retailers.

Looks up each retailer across the `retailers_*` crates by its `RetailerCode`:

- `architecture_for` and `extract_for`: the offer page's extraction
  architecture, and the page extracted with it into JSON.
- `normalize_for` and `structuralize_for`: the extracted JSON normalized, and
  the normalized JSON structuralized. Each returns `None` for a retailer that
  has no such step.
- `sitemap_config_for` and `classify_link_for`: where a retailer's sitemaps are
  and what kind of page each URL is.
- `pretty_json`: JSON text pretty-printed with its keys in their original
  order, as the stage files are written.

The `process_offer_details` binary runs every saved offer page in
`<data_dir>/<retailer>/offer_details/sources` through extraction, normalization
and structuralization, and writes each stage's output as pretty-printed JSON
into `extracted/`, `normalized/` and `structurized/` next to the sources. It
prints how many pages reached each stage and why the rest failed, and exits with
status 1 when any page failed:

```sh
make process-offer-details
make process-offer-details RETAILERS="ankerau ankerjapancom"
```

`DATA_DIR` defaults to `../../../data`, the data checkout next to the monorepo.

With no retailers named, it processes every retailer folder that has sources. A
page that fails a stage has no file for that stage or later ones; files an
earlier run left there are removed.

To add a retailer storefront, or finish one, follow the [retailer integration playbook](../../docs/retailer-integration.md).

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

`make ci` includes normalization regressions over saved extractions in the data
checkout, skipping absent storefront folders. Tests enable serde_json's
`arbitrary_precision` feature to match the app and service dependency graph;
this covers decimal prices, ratings and media aspect ratios inside enum schemas.

The facade captures localization evidence after retailer islands are extracted,
then carries its sidecar through normalization and structuralization without
changing strict retailer raw schemas. See the [source survey](../../docs/localization-source-coverage.md)
for accepted declarations, exclusions, portable fixtures and corpus verification.

New localization observations use the selected locale's lowercase primary
language. If no applicable explicit market is declared, a supported country
region supplies it (`it-IT` → IT / it). The locale region EU supplies the regional
market `European Union` (`en-EU` → European Union / en), as recorded in
[ADR 33](../../docs/adr/0033-european-union-locale-market.md).
Invalid, dynamic or conflicting explicit
markets block fallback; valid explicit markets win. Equivalent locale casing
agrees, while full regional conflicts remain unknown. Raw tags and winning
source paths remain in evidence with selection diagnostics. Only extraction
applies this policy: later stages carry old sidecars unchanged. Unsupported
extlang/grandfathered tags remain unknown, as do unsupported country regions,
numeric macroregions and regions appearing only in Unicode extensions.

The opt-in, read-only `offer_facts_corpus` integration test checks canonical comparison
prices, availability, item condition, and descriptions across the saved source
corpus. See [source coverage](../../docs/offer-facts-source-coverage.md) for source
semantics, nullable fields, and the survey command.
