# Regional and Nordic browser token configuration

Captured on 2026-11-09:

- `app.js`: unmodified `https://www.anker.com/_next/static/chunks/pages/_app-5b4c0e10ae40069f.js`.
  The `/fr`, `/eu-pl`, `/nz`, `/vn`, `/eu-en` and root homepages all referenced
  this bundle. Module 68590 imports `L = n(34914)` and configures
  `storeTokens: L.qj`. Module 34914 exports `qj` as `_`, a literal JSON.parse
  map. Its values are used by `getStorefrontToken(locale)` for the public
  `X-Shopify-Storefront-Access-Token` request header.
- `nordics-app.js`: unmodified
  `https://www.ankernordics.com/_next/static/chunks/pages/_app-1022739065c72234.js`,
  referenced by the Nordic homepage. Module 89185 imports `x = n(85103)` and
  uses `storefrontAccessToken: x.ut[locale]`. Module 85103 exports `ut` as `o`,
  a literal JSON.parse map for se/no/fi/is/dk. Its Shopify domain is
  `ankerneu.myshopify.com`; this is not the global frontend's Swedish store.

`expected.json` and `nordics-expected.json` independently transcribe the selected
public browser token-map entries. These are public Storefront credentials, not
Admin API credentials. Production never loads these fixtures or hardcodes their
values. Tests compare StorefrontToken wrappers so failures redact the values.

The full bundles retain minified code around the configuration and exercise the
lexer. Test HTML is a minimal reference to the observed script, not a captured
homepage. The resolver must follow the explicit import/export relationship;
unrelated JSON maps, variable names and tokens from other modules are not inputs.

Read-only live probes using these discovered tokens succeeded for all six
regional storefronts with one product including tags and each configured
country/language/currency. Nordic probes retrieved `t90r0121` with SE/SEK,
NO/NOK, FI/EUR, IS/ISK and DK/DKK. These checks verify token acceptance and market
selection, not full-catalog completion or durable pipeline execution.

Run offline regressions from `services/retailer_sourcing`:

```sh
cargo test --bin retailer_sourcing regional_bundle_
cargo test --bin retailer_sourcing nordic_acquisition_
```
