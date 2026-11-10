# Anker AU storefront token inputs

`ankerau.html` is the unmodified response downloaded by the user from
`https://www.anker.com/au` with the service's browser User-Agent. It is real input,
not a synthetic storefront. It contains no `storeTokens` configuration; the
`storefront_key` and Judge.me `public_token` values are not verified Shopify
Storefront access tokens.

`assets.json` lists the script URLs referenced by this captured HTML and their
local filenames. Fetch those exact assets from the repository root:

```sh
curl --parallel --parallel-max 6 --create-dirs \
  --config services/retailer_sourcing/tests/fixtures/storefront_token/download-assets.curl
```

The configuration follows the acquisition handler's allowed script hosts. The
user captured all 42 scripts. `assets/16.js` (the `1b2878kmn7hql.js` bundle) sets
`storeTokens: JSON.parse('...')`, whose `au` entry is
`ca89549ed256ec3f2c3a12d113b9f29e`. The same bundle reads that map in
`getStorefrontToken(locale)` and sends the value in the
`X-Shopify-Storefront-Access-Token` header. `assets/01.js` contains the same AU
value in another configuration map. This is a public browser token, not a private
Admin API credential. Its presence in these scripts does not verify that Shopify
currently accepts it.

The handler regression replays the captured HTML and scripts without networking,
asserts this independently identified token in `StorefrontTokenAcquired`, and
checks it is forwarded to the catalog command. The HTML-only regression verifies
that the unrelated Basic and Judge.me credentials are not selected.

Run the captured-HTML regression from `services/retailer_sourcing`:

```sh
cargo test --bin retailer_sourcing downloaded_au_
```
