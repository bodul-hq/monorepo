# ADR 50: Anker AU initial offer fill

- Status: Accepted
- Date: 2026-11-09

## Context

Anker AU needs an initial catalog fill independent of the existing sitemap and
HTML-processing pipeline. The existing feature scaffolds name commands and events,
but have no handlers. The WIP Shopify branch (`c3f5872`) demonstrates a broad AU
GraphQL selection and generated clients, but hardcodes public tokens and fetches
only the first page of each connection. Its saved product has more collections
than its first page contains.

## Decision

Enable only Anker AU through a new `POST /api/initial-offer-fills` route. Keep the
three declared steps and their existing command/event names. Obtain the public
browser token from `storeTokens.au` in the AU storefront's inline or referenced
first-party JavaScript. Carry it in durable commands/events until the final fetch;
redact its Debug representation and omit it from artifacts and the terminal event.

Use `graphql_client` 0.16 and the WIP AU schema/query selection in a versioned
Anker integration, exposed through preludes, the registry, and extractor facade.
HTTP and storage remain service capabilities. Do not import the WIP branch's
parallel client experiment or its conflicting ADR numbering.

Catalog commands request at most 250 identifiers per GraphQL page and collect
batches of up to 500 distinct product GIDs. Each batch event fans out independently
queued product commands and, where necessary, the next catalog command. Fetch
all pages of every connection selected by the AU details query before declaring
a product complete. The full product and raw request/response pages are stored
as one versioned object in `initial_offer_fill_details`; `OfferDetailsFetched`
contains its storage reference and has no subscriber or outbox registration.

Use keyed multi-command bus reactions. Command IDs are standard UUIDv5 values
scoped to the UUIDv7 run ID and command/cursor or command/retailer/product identity.
The implementation uses RustCrypto SHA-1 and uuid's UUIDv5 builder. Mulac already
inserts command IDs with conflict-ignore behavior; partial fan-out can be retried
without adding duplicate commands. Existing single-command reactions are unchanged.
Retain existing command workers, manual event processing, proxy routing, six-second
GraphQL pacing, and queue retries. Authenticated GraphQL requests do not redirect.

## Consequences

This change reads AU/EN Storefront API 2026-10, expects AUD product prices, and
performs no normalization, structuralization, export, or downstream delivery.
Other retailers' process declarations are scaffolds, not executable support.
MiCom and Anker Korea continue to have `todo!()` process definitions; the new API
rejects unsupported retailers before invoking those functions.

No new domain tables or schema migration are needed. Message payloads gain
required fields; these messages were previously unregistered scaffolds. Existing
sitemap and offer-discovery message contracts are unchanged. New run IDs permit
intentional subsequent fills. The pipeline has at-least-once execution: a handler
retry may repeat a fetch/upload or emit another terminal event. A crash after an
upload can leave an unreferenced artifact; this is not an exactly-once snapshot
or a transactional snapshot of the changing remote catalog.

Live acquisition, API access, and integration smoke tests require reachable
network, PostgreSQL, and object storage. Offline tests exercise extraction,
operation schemas, batch limits, pagination, failure propagation, and replay.
