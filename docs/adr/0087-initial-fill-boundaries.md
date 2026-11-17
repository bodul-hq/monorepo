# 87. Initial-fill boundaries

Status: Accepted

Supersedes process-owned token acquisition in ADRs 60 and 74. Confirms the shared
ownership in ADRs 83 and 84. Per-store access settings from ADRs 70–78 remain in
effect as configuration.

## Context

Ownership of GraphQL code moved between shared and per-brand crates in ADRs 58,
61, 63, 64 and 80–86. Token policy changed in ADRs 60, 70, 71, 72, 75, 77 and 78.
Because the token was a stage of the initial-fill process, each policy change
also changed process steps, command payloads and rollout steps.

## Decision

Initial fill is split into six boundaries. Dependencies point downwards only.

| Boundary             | Owner                                   | Owns                                                                            | Does not know about                         |
| -------------------- | --------------------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------- |
| Consumers            | `offers`, `customer_facing`             | What they build from fill output                                                | Anything before the output contract         |
| Initial-fill process | `retailer_sourcing::initial_offer_fill` | Runs, identities, commands and events, persistence, retries, artifact storage   | Shopify endpoints, credentials, field names |
| Brand extraction     | `retailers_<brand>`                     | Storefront configurations, raw-to-extracted conversion, meaning of brand fields | Transport, credentials, runs                |
| Retailer catalog     | `retailer_sauce::retailers`             | Retailers, target markets, shipping countries, supported locales                | HTTP, Shopify                               |
| Source schema        | `retailer_sauce::shopify::graphql`      | Query documents and the raw Shopify response model                              | Business meaning of fields                  |
| Storefront platform  | `retailer_sauce::shopify`               | Endpoints, API version, credentials, transport, error classification            | Brands, offers, runs                        |

### Credentials belong to the platform

`retailer_sauce::shopify` resolves credentials from `StorefrontAccess` when the
engine sends a request: environment override, token discovery, or anonymous
access, as configured. It caches the result per store, resolves again once after
an authentication failure, and keeps credentials redacted.

The process becomes `ProductCatalogIdentifiers → OfferDetails` for every
storefront. Commands and events no longer carry tokens, and sourcing does not
use Shopify token types. Access policy changes are configuration changes only.

### Queries and the raw model are shared

Query documents and the raw model stay in `retailer_sauce`, as ADRs 83 and 84
decided. Brands keep configuration and extraction. A selection added for one
brand is sent for every brand, so it must pass all brand fixture suites and
stay within Shopify's query-cost limits for anonymous and token access.

### The engine is stateless

`retailer_sauce` fetches pages, follows pagination and checks context and
currency consistency. It keeps no run state. Sourcing decides what to run and
owns persistence, retries and dispatch.

### The fill ends at the raw artifact

The output contract is the schema-2 raw artifact in `offer_details_raw` and the
`OfferDetailsFetched` event. The fill neither extracts offers nor reports
completeness; consumers subscribe to the event.

## Compatibility and rollout

Historical `AcquireStorefrontToken` commands and `StorefrontTokenAcquired` events
remain readable. A queued acquisition command fans out catalog commands as the
start API does. Tokens carried by queued catalog and detail commands are ignored.
Command identities are unchanged. Deploy all workers together.

## Consequences

Access policy can change without touching runs or queues. Runs no longer record
which credential was used; the engine logs the credential kind (environment,
discovered or anonymous) instead. Shared queries make field additions cheap but
couple brand request cost.
