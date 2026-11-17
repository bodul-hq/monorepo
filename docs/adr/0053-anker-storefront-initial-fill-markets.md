# ADR 53: Initial fill for the active Anker storefronts

- Status: Accepted
- Date: 2026-11-09
- Supersedes: ADR 52's AU/CA-only scope and fixed-English query policy.
  Canada remains English-only. ADR 51's raw-offer bucket remains in force.

## Context

Initial fill must support every active Anker retailer except Korea (Cafe24).
The captured API research in `data3` supplies verified Shopify hosts, available
country/language pairs and currencies. The existing Nordic integration also
shows that its public browser token exposes products omitted by anonymous access.
The old `AnkerNordicsCom` aggregate is historical; its five replacement markets
are the executable retailer codes.

## Decision

Configure 17 active retailers through the existing facade and initial-fill API.
Keep versioned AU/CA entrypoints and a shared version-1 market configuration table
in `retailers_anker::shopify::initial_fill::markets`. Every typed operation,
including all continuations, receives country and language variables from trusted
configuration. Each retailer uses its storefront language; Canada uses EN.
Nordic stores advertise EN and retain their separate country/currency contexts.
The broad English EU retailer uses its recorded default NL/EUR context; it is
not an instruction to fetch every European shipping destination.

AU, CA, US, UK, DE, EU, PL, FR, NZ and VN acquire their selected `storeTokens`
entry from the configured storefront and scripts. Nordic acquisition uses the
existing `ANKER_NORDICS_STOREFRONT_TOKEN` setting and emits the same acquired event;
missing configuration fails before fetching. Italy and Japan use their verified
public tokenless APIs and start with catalog identifiers, skipping token
acquisition. No fabricated token, fallback credential or automatic auth downgrade
is used.

Catalog/detail commands and catalog events represent authentication with
`Option<StorefrontToken>`. Existing JSON string tokens still read/write identically;
new tokenless messages use null. Successful token-acquired events still require a
real token. HTTP POSTs omit the token header only for explicitly tokenless stores,
retain no-redirect behavior, and use the existing pacing, routing and diagnostics.
The transport checks that command authentication matches retailer configuration.

Tokenless detail operations request one node per nested connection to bound
[tokenless query complexity](https://shopify.dev/docs/api/storefront/2026-10#authentication).
Pagination still exhausts every selected connection. Catalog
pages stay at up to 250 GIDs, with events batching up to 500 and continuing until
the catalog ends. Keep the existing full field selection, including product tags:
captured public Italian/Japanese responses contain tags. Any HTTP, GraphQL or
schema/access error fails the command; no field is silently removed. Artifacts
retain full request/response evidence and the existing schema, bucket and terminal
event without a subscriber.

## Consequences

StorefrontConfig adds explicit browser-map/environment token sources. This is
an internal Rust interface change. No dependency or database migration is needed.
Deploy updated queue consumers before starting tokenless runs: older workers
cannot deserialize null tokens. Existing AU/CA queued messages and command
identities remain compatible.

Small endpoint/localization evidence and one captured Japanese product are
checked into the shared query fixtures. Tests cover every configuration and
operation, authentication, language/currency, 500-ID fan-out, pagination, API
acceptance and terminal events. PostgreSQL and live tests remain opt-in. Captured
responses establish implementation inputs, not a guarantee of current network,
credentials, permissions or catalog stability.
