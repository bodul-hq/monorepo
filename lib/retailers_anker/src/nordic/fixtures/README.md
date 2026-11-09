# Nordic market responses

Fetched on 2026-11-09 from the public Nordic Storefront API using
`../product.graphql`, handle `t90r0121` (eufy Indoor Siren E20), English and
each fixture's country. The handle appears in the live product sitemap.
Requests used the browser application's public Storefront access token;
the token is not stored in these fixtures.

The five responses establish independent prices: SE 499 SEK, NO 499 NOK,
FI 44.99 EUR, IS 6490 ISK and DK 349 DKK. `onlineStoreUrl: null` is expected
for this headless-channel product; its shopper URL comes from the sitemap's
verified `/products/<handle>` route. Anonymous API access omitted this product.

These are API fixtures, not five renamed copies of the historical Nordic HTML
corpus. Tests additionally exercise errors and mismatched markets by mutating
copies in memory.
