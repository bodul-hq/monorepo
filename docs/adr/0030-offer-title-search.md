# ADR 30: PostgreSQL offer title search

Status: Accepted; extends ADR 23.

Date: 2026-10-26

## Context

Customers need typo-tolerant title search with relevance ranking and bounded
pagination over the existing customer-facing projection.

## Decision

Use PostgreSQL `pg_trgm` word similarity on `lower(title)`, with a matching GIN
index. Provision the extension separately in the trusted
`customer_facing_extensions` schema. Startup and migration check that location;
application code never relocates or installs database-wide extensions.

Bind all query values. Set word similarity threshold to 0.5 and statement timeout
to three seconds locally in the search transaction. Order by case-insensitive
full-title equality, word similarity, creation time descending and ID descending.
Fetch one extra offer after ranking, then attach ordered images. Browse and
search share record mapping and public DTOs.

REST and GraphQL share validation: normalize whitespace, require 1–200 Unicode
scalar values including an alphanumeric character, and reject non-whitespace
controls. Only query text, limit and offset are accepted as search options.
Customers submits a centered GET form to its server-rendered `/search` page,
with a fixed upstream page size of 30 and the existing five-second timeout.

## Consequences

Existing rows become searchable when the index is built, without replay or a
worker. Transactional index creation blocks writes; provision and migrate the
service before deploying the app. Rollback removes only the index, retaining
the extension and projection. Pagination can shift as offers change.

Stored titles are searched before display HTML cleanup. Punctuation is text,
but trigram extraction ignores non-word characters; no accent folding,
stemming or guaranteed correction of every typo is promised. See the
[PostgreSQL pg_trgm reference](https://www.postgresql.org/docs/16/pgtrgm.html).
