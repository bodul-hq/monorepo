# ADR 24: Append-only offer images

## Status

Accepted. Supersedes only ADR 19's exclusion of images and ADR 21/22's
restriction to three offer event types. Other decisions remain in force.

## Date

2026-10-25

## Context

Structuralized variations already contain images, but exporting an offer dropped
them. Customers need the first image while immutable exports and historical
messages must keep their original identities and payloads. Event delivery can
arrive out of order.

## Decision

New `ProcessOfferForExport` commands select schema 2 explicitly. Commands without
`schema_version` select schema 1. Both versions remain readable and retryable;
row, snapshot and event versions must agree. V1 payloads omit `images` forever;
v2 includes a string array, even when empty. Schema version remains part of the
export uniqueness key. Pending bytes and all snapshot/entry identities survive
retries unchanged. No historical rewrite or automatic backfill is performed.

Sourcing and offers accept only absolute HTTP(S) image URLs with a host, no
credentials, padding or control characters, and at most 2048 UTF-8 bytes (the
existing offer URL storage limit). Invalid strings are skipped and counted
without logging their URLs; wrong JSON types fail. Keep the first occurrence
of each exact string without trimming, resolving, probing or canonicalizing it.
Missing images in an import means an empty incoming list, never removal.

The offers aggregate emits `OfferCreated` first, then `OfferImageAdded` for each
image in input order. Updates emit title and price changes first, followed by
new images only. The public payload is
`{"id":"<offer UUID>","image_url":"https://shop.example/front.jpg","position":0}`;
the stored Testimony tag is `OfferImageAdded.v1`. Position is `u32`, persisted
as PostgreSQL `BIGINT` constrained to 0–4294967295. History rejects duplicate
URLs and noncontiguous image positions. Capacity is checked before mutation.

Both projections store images in child tables with foreign keys and unique
`(offer_id, position)` and `(offer_id, image_url)` constraints using exact URL
comparison. Identical additions succeed; conflicts and missing parents fail and
retry. Reads sort by position; gaps from delayed messages are temporary.
Customer-facing commits additions and original-message receipts together and
returns non-null `images` arrays in REST and GraphQL. A single page query fetches
images without multiplying offer rows or changing pagination.

Customers accepts older responses without `images`, validates and escapes the
first URL and title, and renders a lazy image with a fixed aspect ratio and
`object-fit: contain`. Missing/unsafe images leave an empty area. Browser image
failures preserve title, price and purchase links. Retailer images load directly
in the browser; no server downloads, JavaScript or carousel is introduced.

## Consequences

Existing offers acquire images on their next image-bearing import. Reordering,
omission and repeat imports cannot remove or change the first stored image.
Title and price still have no revision ordering.

Deploy customers, customer-facing, offers, then sourcing. Apply both additive
image migrations before processing image events. Do not roll offers back to a
binary unable to read stored image history, or customer-facing back while image
messages are pending. Retain v1 export handling and projection receipts.

The user explicitly kept this checkout's current offers handler in scope for
item 11. Durable offers import receipts are not implemented here. Its existing
history-to-Mulac publication crash window and identity-projection race remain;
this decision adds no guarantee across those boundaries. Customer-facing's
existing transactional original-message receipts remain intact.
