# Customers

Poem app showing 30 offers per page from customer-facing, newest first, with
Previous/Next links. It renders HTML and local responsive CSS without JavaScript,
external fonts, browser API calls or frontend build tools.

## Run

Start customer-facing with its own database, then run:

```sh
CUSTOMER_FACING_URL=http://localhost:3003 PORT=3004 make serve
curl http://localhost:3004/
curl 'http://localhost:3004/?offset=30'
```

`PORT` defaults to 3000. `CUSTOMER_FACING_URL` defaults to
`http://localhost:3003`; explicit blank or invalid values stop startup.
Use an HTTP(S) base URL without credentials, query or fragment. Path prefixes
are preserved. In the parent Compose deployment, set the customers container's
environment to `CUSTOMER_FACING_URL=http://customer_facing:3000`;
localhost inside that container does not refer to customer-facing.

Each page makes one server-side `GET /api/offers?limit=30&offset=N` request.
The shared HTTP client has a five-second timeout, no automatic retries and no
redirects. The app has no database connection and does not execute queues.
See the [customer-facing walkthrough](../../services/customer_facing/README.md#browsing-offers)
for making offers and updates visible through inbox and command processing.

## Behavior

Offset defaults to zero and accepts integers from 0 through 2147483647.
Invalid offsets produce HTML 400 responses. Previous subtracts 30 with a floor
of zero; Next requires `has_more` and a representable next offset. New offers
can shift pages between requests.

Successful empty reads return 200 with an empty state and relevant navigation.
Connection failures, timeouts, non-200 responses and malformed API data return
a styled 503 with a retry link and no upstream diagnostics. All pages use
`Cache-Control: no-store` so refresh sees the latest projection.

Prices retain their currency and use integer arithmetic with the existing
`money::Currency::exponent()`, including zero-decimal currencies and the full
signed i64 range. Dynamic values are HTML-escaped; only absolute HTTP(S) URLs
without credentials become store links. Offers with other URLs remain visible.
Product images, search, filters, conversion and background fetching are not included.

## Checks

```sh
make fmt
make ci
# When dependencies or Cargo.lock change:
make -C .. audit
```

Rust tests use local HTTP fixtures and Poem's test client to cover outbound
requests, prefixes, 30/1/empty pages, pagination, timeouts, redirects, malformed
data, error pages, escaping, unsafe URLs and exact price formatting. Rendering
tests check the absence of scripts. CSS is embedded with `include_str!`; review
the served page at desktop and mobile widths after layout changes. There is no
separate CSS test runner.

For a combined check, process 31 unique `OfferCreated` inbox messages through
customer-facing's inbox and command queues in an isolated test schema. Verify
30 cards at `/`, one at `/?offset=30`, and changed values after processing
`OfferTitleChanged` and `OfferPriceChangedEvent` and refreshing. Stop the upstream
to verify the 503 page. Do not populate or reset operational databases for tests.

See [ADR 23](../../docs/adr/0023-offer-browsing-and-customers-homepage.md).
