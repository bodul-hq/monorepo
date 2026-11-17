# ADR 54: Discover Anker browser tokens through imported configuration

- Status: Accepted
- Date: 2026-11-09
- Supersedes: ADR 53's mandatory environment-only Nordic initial-fill token acquisition.
  ADR 47's environment requirement for the separate sitemap offer-download flow remains.

## Context

The FR, PL, NZ, VN, EU and US homepages reference a regional Webpack bundle.
Its Storefront token map is a literal exported by another module and supplied as
`storeTokens: L.qj`; it is not attached directly to the property. The original
extractor therefore reports no token even though the browser publishes it.
The Nordic browser similarly imports a locale-indexed `storefrontAccessToken`
map for its existing headless Shopify store.

## Decision

Read the literal configuration through its explicit numeric module import,
export getter and module-scope JSON.parse assignment. Never evaluate JavaScript
or select a token solely because it resembles a credential. Retain exact market
selection, validation, conflict rejection and allowed script hosts.

For Nordic initial fill, use ANKER_NORDICS_STOREFRONT_TOKEN when configured.
Otherwise discover the selected se/no/fi/is/dk browser token from the Nordic
storefront. An invalid configured value fails explicitly. Missing discovery
still fails: there is no anonymous fallback. Other retailer authentication,
Shopify endpoints, language/currency context and queued event contracts remain.

## Consequences

Nordic initial fill no longer requires manual token configuration. The separate
sitemap-driven Nordic offer-download flow still requires the environment setting.
The decoder intentionally recognizes a limited static bundle shape; a frontend
build with a different representation needs fresh captured evidence and tests.
Whole captured bundles cover real minified input, and synthetic tests cover
unrelated maps, variable shadowing, malformed values and conflicts. No dependency,
database migration or queued payload change is needed.

Live read-only probes confirmed the discovered tokens for all eleven affected
market codes. They do not establish full pipeline completion. See the
[capture provenance](../../services/retailer_sourcing/tests/fixtures/storefront_token/regional/README.md).
