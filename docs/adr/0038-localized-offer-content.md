# ADR 38: Localized offer content lists

Status: Accepted; supersedes ADR 36's single-content model in offers and downstream.

Date: 2026-10-27

## Context

An offer needs to retain observed titles and descriptions in multiple languages.
Existing single-language source observations and immutable exports must stay
readable, while cards and title search retain their current selected title.
The user selected merge-by-language behavior from offers through customers and
retained flat title/language compatibility fields.

## Decision

Introduce `retailer_sauce::offer_content::Content` with required title and
nullable language/description. An offer's `content` is a nonempty list with
unique exact language keys, including at most one null/unknown entry. Sort
unknown first, then exact language string. Preserve exact text and historical
language spelling. New observations retain existing source normalization.
Keep source structuralized Content and source export/import formats unchanged.

An import upserts its observed language and retains other languages. Omitted
localization retains the current selection; a present localization with absent
or null language selects unknown. Missing description observation retains only
the destination entry's value; explicit null clears it. A new language without
a description initializes unknown, never copying another language's text.

Flat title, localization language and stored description describe the selected
entry. Write `OfferCreated.v9` with list content and preserve separate v1–v8
creation decoding. New changes use `OfferContentsChanged.v1`, routing
`OfferContentsChanged`, carrying the complete `content` list and required
nullable `language` selection. Validate before mutation and update the list and
selected fields together. Selection-only changes emit the event too. Publish
it before any remaining market-only localization change; new imports do not
emit separate title/description events.

Historical creation maps to one entry. Historical title/description changes
update the selected entry. A historical language change moves that entry to the
new key, replacing a destination collision and removing the old key. This
retains the former single-tuple behavior without inventing translations. Other
entries remain, and market-only changes do not alter content. Share this
reconciliation across replay and projection writers.

Both projections add nonnull JSONB list storage, backfilled transactionally from
the exact existing scalar tuple. Retain scalar columns and title-search indexes.
Full snapshots write atomically. Legacy read/modify/write locks the row, and
customer-facing keeps its applied-message receipt in the projection transaction.

REST/GraphQL expose list content. Customers reads both old object/absent content
and new arrays. It continues rendering the selected flat title; descriptions
remain data only. No language selector or multilingual search is introduced.

## Consequences

Deploy the compatible customers reader before customer-facing starts returning
arrays, then migrate/enable the offers writer. Pause projection writers during
backfill. Preserve queued message identities and payloads and all event history.
An old client expecting an object at `content` needs the compatible reader;
retaining flat fields alone does not make that shape change transparent.

Dropping the new columns retains the latest selected scalar values but loses
other languages from that projection. Preserve backups or rebuild from history
for recovery; down/up is not a lossless rollback of multilingual data. Existing
retailer/URL offer identity remains unchanged, so distinct locale URLs are not
merged into one offer.
