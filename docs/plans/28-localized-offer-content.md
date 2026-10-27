# Localized offer content

Status: implemented on 2026-10-27. Planned against monorepo `9fb43bc` and Mulac `fd29f72`.
The user confirmed offers and downstream scope, merge by language, and retained
flat title/language fields for display, search and compatibility.

## Contract

Offers and downstream represent `content` as a nonempty list of
`Content { language: Option<String>, title: String, description: Option<String> }`.
Use a new shared `retailer_sauce::offer_content` module; keep the existing
description-only structuralized type and source/export/import contracts intact.
Source exports continue writing v8 and reading v1–v8.

Entries have unique exact language keys, including one optional unknown/null
bucket. Preserve historical tags verbatim and retain existing normalization for
new source observations. Sort unknown first, then exact language string. Guard
the whole change before mutation: valid optional language, nonblank/NUL-free
title, NUL-free nullable description and an existing selected language entry.
Preserve decoded text, HTML, Unicode, whitespace and empty descriptions.

Each import upserts one language while retaining the others. Omitted localization
uses the current selection; present localization with missing/null language uses
the unknown bucket. Missing content preserves only that destination entry's
description, or initializes it to null for a new language. Explicit null
description clears it. No translation or cross-language copying is inferred.

## History and persistence

Write `OfferCreated.v9`, retaining separate legacy v1–v8 creation decoding.
Routing remains `OfferCreated`; downstream readers accept old missing/object
content and new list content. Add `OfferContentsChanged.v1`, routed as
`OfferContentsChanged`, with `{id, content, language}`. The nullable language key
is required and selects an entry. Emit when either the list or selection changes.
Derive the scalar title, language and stored description from that selection.
New imports stop emitting separate title/description changes and publish the
full list before any remaining market-only localization change.

Legacy creation becomes a singleton. Legacy title/description changes modify
the selected entry; a legacy language change moves it to the new key, replacing
any destination collision and removing the old key. Preserve other languages.
Market-only updates leave the list untouched. Share reconciliation between
replay and projections to retain the old single-tuple meaning.

Add nonnull JSONB content to both projections, transactionally backfilling a
singleton from exact stored scalar values and checking for a nonempty array.
Retain scalar columns and the title-search index. New snapshots update content
and selection fields atomically. Legacy read/modify/write uses a row lock;
customer-facing retains its receipt in the same transaction. In-memory writers
follow the same rules under their existing lock.

## Read contracts and rollout

REST/GraphQL content becomes a nonnull list with nullable language/description
and required title. Customers accepts old object/absent content and new arrays;
cards/search continue using the flat title and descriptions remain data only.
Deploy the compatible customers reader, then migrated customer-facing readers,
then migrated offers writers. Pause affected projection writers during backfill.
Down migrations retain selected scalar values; additional languages require
backup/replay. Record the decision in [ADR 38](../adr/0038-localized-offer-content.md).

## Validation

Test language upserts, unknown language, omission/clear semantics, selection-only
changes, identical imports, guards and sequence capacity before mutation. Replay
real legacy creation/change fixtures and new records, including key collisions
and mixed versions. Check populated migrations, atomic projection/receipt writes,
row locking, retries, exact payload identities, REST/GraphQL browse/search parity
and unchanged card rendering. Run required fmt/CI, PostgreSQL suites and live
health/import/delivery checks; record unavailable runtime prerequisites honestly.

## Completion

Shared content helpers, v9 creation and plural change events, legacy replay,
both projection migrations/writers, REST/GraphQL arrays and the customers dual
reader are implemented. See the [verification record](28-31-verification.md)
for passing checks and database/live execution still requiring an unrestricted
runtime and configured databases.
