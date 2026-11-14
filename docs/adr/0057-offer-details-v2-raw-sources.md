# 57. Offer details v2 raw sources

Date: 2026-11-10

Status: Accepted

## Context

Initial fill saves complete Shopify GraphQL artifacts in `offer_details_raw`
(ADR 51), with terminal `OfferDetailsFetched` events. The existing development
source listing and offline offer-details stages use the HTML offer projection.
The new flow needs source dumping and raw deserialization, starting with AU and
CA. Extracted representations are outside this change.

## Decision

Treat the GraphQL flow as offer details v2. Add
`GET /api/development/offer-details-v2/sources` and `make dump-offer-details-v2`.
The endpoint reads persisted terminal events regardless of consumer status and
returns only identity, retailer, fetch time and storage metadata, ordered by
fetch time descending with event ID as a tie breaker. It shares development API
authentication and the 1–1000 limit/offset contract. This listing does not require
running event consumers and also finds artifacts fetched before the endpoint
was introduced.

The dumper writes the newest successful artifact per retailer/product to
`<DATA_DIR>/<retailer>/offer_details_v2/sources/<numeric Shopify product ID>.json`.
It validates the object-storage reference and writes the original payload
unchanged, including the merged product and every request/response page. Existing
HTML dumping and processing retain their current paths. V2 names the pipeline;
the persisted artifact's `schema_version` remains 1.

AU and CA expose versioned raw product, GraphQL response and stored-artifact
representations. They share the Shopify source schema, surveyed from the saved
AU/CA product corpus and checked against the query/schema. Raw structs reject
unknown fields, retain decimal money as text and other numeric values as JSON
numbers, and model media/selling-plan unions by `__typename`. The older dump's
optional collection-product connections, media IDs and localization are included.
Original continuation response evidence and diagnostic GraphQL metadata remain
JSON because their selections differ by operation; the merged product is typed.
No extraction or normalization behavior changes.

## Consequences

A developer can inspect raw v2 artifacts independently of HTML processing and
deserialize products without losing source fields. Checked-in real fixtures cover
observed shapes; optional local corpus tests verify all saved AU/CA products and
full product response pages. New source fields require a deliberate raw-schema
update. AU/CA aliases establish tested scope without registering other retailers.

The development listing scans event payloads and depends on event retention;
purging those events removes their discoverability even while stored artifacts
remain. A durable indexed source projection is a future option if retention or
listing volume requires it. Offset pagination is intended for development dumps
with a stable source set; concurrent arrivals can shift page boundaries, so rerun
a dump after the fill finishes for a complete snapshot.
