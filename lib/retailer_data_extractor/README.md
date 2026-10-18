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
