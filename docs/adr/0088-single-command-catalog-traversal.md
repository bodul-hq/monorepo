# 88. Single-command catalog traversal

Status: Accepted

ADR 74's command identities are unchanged: the catalog command keeps the key
`["FetchProductCatalogIdentifiers", country, language, null]`.

## Context

Each locale's catalog was walked as a chain of commands. The
`FetchProductCatalogIdentifiers` handler fetched one batch of up to 500 product
IDs and emitted `ProductCatalogIdsRetrieved` with a `next_cursor`. A reaction
then queued another `FetchProductCatalogIdentifiers` for that cursor. A large
catalog therefore re-sent the same command many times. The queue held one row
per batch, and the traversal was spread over many events.

## Decision

`FetchProductCatalogIdentifiers` is queued once per locale. Its handler calls
the integration's `catalog_for_locale` in a loop until no cursor remains. It
carries the resolved context between batches, keeps the first-seen order of
IDs and drops IDs repeated across batches. It fails the command if a cursor
appears twice.

The handler emits one `ProductCatalogIdsRetrieved` per locale. The event holds
every product ID, `product_count` (unique IDs) and `loop_count` (batches
fetched). The command's `after` field, the event's `after` and `next_cursor`
fields and the `fetch_next_catalog_page` reaction are removed. Every traversal
starts at the beginning of the catalog.

## Compatibility and rollout

- The command key keeps its trailing `null` start cursor, so command IDs are
  unchanged.
- A continuation command queued before the deploy still deserializes. Serde
  ignores its `after`, so it walks the whole catalog from the start. Detail
  commands deduplicate, so this only repeats catalog requests.
- Stored events still deserialize. Serde ignores `after` and `next_cursor`, and
  the new counts read as 0.
- Events stored before the deploy no longer queue a continuation. Drain any
  initial fills still in flight before deploying, or restart them afterwards.

## Consequences

- One command and one event per locale make a run easier to follow.
- Retries are coarser. A failure in any batch retries the whole traversal for
  that locale. Detail command IDs are stable per product, so the queue drops
  repeated detail work.
- The event grows with the catalog: one ID string per product.
