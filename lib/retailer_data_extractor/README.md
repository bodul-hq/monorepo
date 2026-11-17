# retailer_data_extractor

Facade for retailer extraction, sitemap discovery and initial fills.

Anker, Minisforum and Ugreen use the retailer crates published to the `bodul` registry at version 0.2.0. Mi remains disconnected.
All supported Anker GraphQL storefronts use the shared complete extracted model
in `retailers_anker::extracted`. It preserves every modeled raw field using the
AU/CA-style grouping. Minisforum and Ugreen continue typed raw v2 parsing.
Localization and JSON pretty printing remain available.

The five Nordic Anker country codes route their legacy downloads and processing
through `retailers_anker::prelude::legacy_offer_source_for`. The historical
`ankernordicscom` code still deserializes, but has no extraction or sitemap
integration. Download requests and validation fail explicitly for that code,
including queued work; there is no fallback HTTP request or country remapping.
See [ADR 86](../../docs/adr/0086-country-owned-legacy-anker-integrations.md).

`get_storefront_config(RetailerCode)` loads a retailer's `StorefrontConfig`, its
inventory record and initial-fill settings, from the brand crates' `retailer`
modules. It returns `None` for retailers without a storefront integration. Its
tests compare every record with `docs/retailer.md`.

Run `make fmt` and `make ci` here to verify the facade.

See the [Anker README](../retailers_anker/README.md#shared-complete-extracted-model) for the output contract and validation.
