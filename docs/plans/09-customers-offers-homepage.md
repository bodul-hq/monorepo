# Item 9: Customers offers homepage

Status: completed and verified on 2026-10-25.

## Goal and dependency

Show real offers from customer-facing on the customers homepage, newest first,
with 30 offers per page. Consume the exact
[item 8 API contract](08-customer-facing-offers-api.md), agreed before either
implementation. Use server-rendered HTML and local CSS without JavaScript.

## Implementation

- Private async REST client: `CUSTOMER_FACING_URL`, default
  `http://localhost:3003`, five-second timeout, no automatic retries or
  redirects. Invalid configuration fails startup.
- `/` and `/?offset=N` request `/api/offers?limit=30&offset=N`. Validate offset
  against the API bounds. Invalid input produces an HTML 400 response.
- Previous subtracts 30 with a floor of zero. Next adds 30 only when `has_more`
  is true and the offset remains representable.
- Light Bodul page, Latest offers heading, teal accents and responsive
  three/two/one-column cards with retailer, full title, price and View offer
  links. Use system fonts, semantic HTML, visible focus and sufficient contrast.
- Format decimal-string minor units using integer arithmetic and the existing
  currency exponent definitions. Handle negatives, zero and full i64 range.
- Escape dynamic text and attributes. Only absolute HTTP(S) offer URLs become
  links; other offers remain visible without an outbound link.
- Successful empty pages show an empty state and relevant navigation.
  Upstream failures show a styled 503 with a retry link, without diagnostics.
- Keep the client private to the app. Share reqwest and serde dependency
  declarations with the development app through the apps workspace.
- Document local/container configuration and tests in a new app README.

Defaults: English copy. No product images (not present in the projection),
search, filters, currency conversion, background processing or backfill.

## Acceptance and verification

Test outbound HTTP requests against a local fixture; rendering 30 items;
pagination bounds; empty, malformed and error responses; timeouts; escaping;
unsafe URLs; exact currency formatting; and absence of scripts.

Run apps formatting and CI, audit changed dependencies, and serve with
`CUSTOMER_FACING_URL=http://localhost:3103 PORT=3104 make serve`.
Check desktop and narrow layouts. With 31 fixtures processed through
customer-facing's inbox and command queues in an isolated test schema, verify
30 then one item and an updated title/price after refresh. Do not alter
operational data for verification. Update README, changelog, plan index and
TODO only with verified completion evidence.

## Completion evidence

- Implemented the private REST client, responsive cards, ordinary pagination,
  exact currency formatting, escaped links, and empty/error pages. Added app
  setup instructions and ADR 23. No JavaScript or frontend tooling was added.
- `make -C apps fmt` and `make -C apps ci` passed. The 13 customers tests cover
  real HTTP fixtures and rendering. Reviewed Firefox screenshots at 1440×1100
  and 390×844 and corrected mobile introductory text spacing.
- Live checks against an isolated customer-facing test schema verified 30/1
  pagination, visible projected updates and HTML 503 when the upstream stopped.
  The development app also returned HTTP 200 at `/` with `PORT=3105 make serve`
  after its dependency declarations moved to workspace inheritance.
- Apps audit ran and reports existing `rustls-webpki` 0.103.15
  (RUSTSEC-2026-0382) and `webpki-root-certs` 1.0.9 (RUSTSEC-2026-0383)
  vulnerabilities, plus existing fxhash, kuchiki, rand and tokio warnings.
  The lockfile adds only customers' edges to already-resolved packages; no
  dependency version changed and no new advisory was introduced. Audit is not
  clean; dependency remediation remains separate work.
- Local evidence: `/tmp/bodul-offers-apps-ci.log`,
  `/tmp/bodul-customers-final-ci.log`, `/tmp/bodul-offers-apps-audit.log`,
  `/tmp/bodul-offers-live.log`, `/tmp/bodul-offers-desktop.png`, and
  `/tmp/bodul-offers-mobile.png`.
