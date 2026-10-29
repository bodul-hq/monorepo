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
signed i64 range. Symbols, placement and separators follow a fixed convention
per currency, independent of retailer country and browser language. The visible
price uses a symbol; assistive technology receives a single ungrouped amount
with its ISO currency code. No currency conversion occurs.

A comparable `was_price` appears above the current price in a semantic `<del>`
using the same currency formatter. Its currency must match and its amount must
exceed the current price; other comparisons are hidden. Assistive technology
receives one “Was …; now …” description including both ISO currency codes.
Legacy responses may omit `was_price` and retain the original price display.

| Currency | Example display |
| --- | --- |
| USD | `$1,234.56` |
| EUR | `1.234,56 €` |
| CAD | `CA$1,234.56` |
| AUD | `A$1,234.56` |
| GBP | `£1,234.56` |
| JPY | `¥1,234` |
| KRW | `₩1,234` |
| HKD | `HK$1,234.56` |
| PLN | `1 234,56 zł` |
| SEK | `1 234,56 kr` |
| VND | `1.234 ₫` |
| NZD | `NZ$1,234.56` |
| INR | `₹1,23,456.78` |

Suffix spacing and PLN/SEK group separators use U+00A0 nonbreaking spaces.
Fractional digits are retained even for whole amounts; JPY, KRW and VND have
none. A negative amount has a leading `−`, such as `−$0.01`.

Titles are parsed as HTML fragments for display: tags, comments and
script/style/template content are removed, character references are decoded
once, and whitespace and block boundaries become single spaces. Encoded
literal tags remain plain text. The resulting title is shared by the heading,
image alt text and link description, with “Untitled offer” used if it is empty.
The original API title is not changed.

Dynamic values, including the cleaned title, are HTML-escaped; only absolute
HTTP(S) URLs without credentials become store links. Offers with other URLs
remain visible.
Each card renders only the first image, after URL validation and HTML escaping,
with offer-title alt text, lazy loading, a 4:3 aspect ratio and object containment.
The browser requests retailer images directly. Missing or unsafe images use an
empty area; a failed load leaves title, price and purchase navigation usable.
The retailer code overlays the top-left of the image area with a semi-transparent grey
background, including when no image is available. Price and the offer action
share a footer row, wrapping on narrow cards when needed. “View offer” opens a
new tab with opener isolation and an accessible announcement; pagination stays
in the current tab.
Older APIs may omit `images`. Desktop/mobile layouts support light and dark
preferences. Search, filters, conversion and background fetching are not included.

## Checks

```sh
make fmt
make ci
# When dependencies or Cargo.lock change:
make -C .. audit
```

Rust tests use local HTTP fixtures and Poem's test client to cover outbound
requests, prefixes, 30/1/empty pages, pagination, timeouts, redirects, malformed
data, error pages, escaping, unsafe URLs, all currency conventions and exact
integer limits. Title tests cover malformed markup, excluded content, entity
decoding, Unicode and consistent heading/alt/link text. Rendering tests check
the absence of scripts and the accessible price description. CSS is embedded
with `include_str!`; review the served page at desktop and mobile widths after
layout changes. There is no separate CSS test runner.

For a combined check, process 31 unique `OfferCreated` inbox messages through
customer-facing's inbox and command queues in an isolated test schema. Verify
30 cards at `/`, one at `/?offset=30`, and changed values after processing
`OfferTitleChanged` and `OfferPriceChangedEvent` and refreshing. Stop the upstream
to verify the 503 page. Do not populate or reset operational databases for tests.

See [ADR 23](../../docs/adr/0023-offer-browsing-and-customers-homepage.md).

Image behavior and rollout: [ADR 24](../../docs/adr/0024-append-only-offer-images.md),
[combined walkthrough](../../docs/offer-images-walkthrough.md).

The response model also accepts source-backed localization, GTIN, SKU and optional
Shopify identifiers. Older API responses default to unknown values. These fields
are validated but not displayed; card HTML, page language and currency formatting
are unchanged. See the [offer facts walkthrough](../../docs/offer-facts-walkthrough.md).

Availability and item condition accept nullable typed `{kind, raw}` API values;
only `other` has raw source text. `content` accepts a nonempty list of `{language, title, description}` entries.
The selected `localization.language` must exist; duplicate language keys and
malformed entries are rejected. Nullable language and description fields are
required in each list entry. Lists retain exact language spelling and sort the
unknown bucket first, then other keys lexically. Old responses with missing
content or the singular `{description}` object are adapted to one entry using
the flat title and localization language. Descriptions preserve raw HTML or
text as data and are never inserted into page HTML. Cards continue using the
flat title; unselected localized titles are also data only. Deploy this
compatibility reader before customer-facing starts returning content arrays. The same model and sale-price
presentation apply to browse and search results.

## Search

The shared header has a labelled GET search form. `/search?q=charger&offset=0`
requests 30 relevance-ranked offers from customer-facing's `/api/offers/search`.
The centered desktop form occupies a full second row below 960px. It uses no
JavaScript. New submissions reset pagination; Previous/Next and retries preserve
the normalized, URL-encoded query. Cards retain plain-text titles, ordered images,
retailer currencies and safe new-tab links.

Missing/blank text shows a prompt without an API call. Other text must contain
1–200 Unicode scalar values including a letter or number; whitespace collapses
and non-whitespace controls are rejected. Invalid text/offset returns HTML 400;
upstream errors, timeouts or invalid responses return HTML 503. All pages use
`Cache-Control: no-store`. Empty first results and past-end pages have distinct
messages. The existing five-second server-side timeout and configured base URL
prefix handling also apply to search. Deploy the service's search endpoint and
`pg_trgm` prerequisite first; see the customer-facing README and ADR 30.

Localized content entries now accept and retain their own URL. During rolling
upgrades, entries without a URL inherit the response's flat offer URL; legacy
absent/object content remains supported. The selected entry must agree with the
flat title and URL used by cards. Null, malformed and overlong entry URLs are
rejected, and unrelated unknown entry fields remain invalid. Search now covers
the selected language's title and description, with title weighted five times.

European Union target markets now use `{"kind":"region","code":"eu"}`.
Historical labels remain readable; exact `European Union` labels canonicalize
at current read/write boundaries. Countries and other historical labels are
not reinterpreted. Deploy compatible readers before new producers and pause
projection writers during the regional-code migrations (ADR 40). Source exports
now write v9, Offers creation v10 and localization changes v2; old immutable
versions retain their original readers and encodings.
