# 26. Offer scrape provenance and monotonic scrape time

## Status

Accepted. Extends ADR 19's immutable exports and ADRs 21/22/25's event contracts.

## Date

2026-10-26

## Context

Queued pipeline work previously read replaceable current artifact rows. Passing
only a download timestamp could therefore associate an old time with a new page.

## Decision

Capture UTC at microsecond precision immediately after a successful fetch.
Publish `OfferPageWasDownloaded.scraped_at` only after object and row storage
succeed. Persist it with the immutable `OfferStorageMetadata`. Extraction,
normalization and structuralization commands pin that metadata, read that object,
and reject conflicting timestamps. Legacy commands read current artifacts with
unknown time. Do not infer historical times from processing timestamps.

New export commands select v3, carrying time in source storage and import payloads.
Keep explicit v1/v2 commands and their original shapes; no new null keys are added
to legacy snapshots. Existing immutable source/version conflicts remain errors.

`OfferCreated.v3` includes optional time; v1/v2 history remains readable.
`OfferScrapedAtUpdated.v1` carries a known time and follows title, price and image
updates. It is emitted even for otherwise unchanged imports when the time advances.
Both projections retain the maximum known time. Customer-facing exposes nullable
`scraped_at` / `scrapedAt` in UTC RFC3339; creation time and ordering are unchanged.

## Consequences

Deploy customer-facing, then offers, then sourcing; upgrade intermediate readers
before generating new downloads. Existing rows remain unknown without a backfill.
Queue processing stays explicit. This does not close the existing Testimony-save /
Mulac-publication failure window or protect titles/prices against stale imports.
