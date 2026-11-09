# shared

Code shared across the monorepo.

`shared::health()` returns the liveness response (`"ok"`) that every service
serves at `/api/health` and through the GraphQL `health` query. It reports that
the process is running; it doesn't check dependencies such as the database.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

`shared::images::is_valid` checks exact absolute HTTP(S) image URLs without
credentials, padding or controls, up to 2048 bytes. `images::filter` preserves
source order, removes exact duplicates, and returns the number of invalid URLs
skipped. It does not contact image servers or rewrite URLs.

## Retailer and extraction types

`shared::retailer` provides retailer codes, slug conversions, and the retailer
list. `shared::link::LinkKind` classifies sitemap links. `SitemapConfig` contains
sitemap URLs; `NormalizedData` carries normalized JSON and related offer URLs.
`shared::prelude` re-exports normalized data and retailer conversion types.

These APIs and their tests were imported from the resolved `bodul_shared` 0.1.1
package in the `null-labs-hq` registry (package SHA-256
`10192019c3cee94b7cd4c8cc57e79c4cec86f1f11a506e163e9990faa5988544`).
The upstream [MIT notice](LICENSE-MIT) is retained. Serialization, retailer slugs,
and link labels remain compatible, including the existing `not_intersted` label.
See [ADR 46](../../docs/adr/0046-local-shared-library.md) for local ownership.
