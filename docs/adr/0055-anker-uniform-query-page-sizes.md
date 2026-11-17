# ADR 55: Uniform Anker initial-fill query page sizes

- Status: Accepted
- Date: 2026-11-09
- Supersedes: ADR 53's one-item page sizes for tokenless detail queries.

## Context

The shared query builder reduced every nested page size to one when a retailer
used tokenless access. Italy and Japan therefore required a separate request
for each additional image, variant and other connection item. Each request also
incurred the transport's six-second delay. Authentication alone should not
determine pagination sizes.

## Decision

Use the existing query-specific sizes for every retailer: 25 for product
collections/images/media, 20 for variants/selling-plan groups, and 5 for nested
variant connections and selling plans. Selling-plan continuation still selects
one owning group at a time. Catalog requests remain capped at 250 IDs per HTTP
request and 500 IDs per command.

Keep the selected fields, cursor pagination, authentication and pacing intact.
There is no automatic page-size reduction based on access mode. Actual API errors
continue to fail the command; this change does not assert that every product's
query will be accepted by Shopify.

## Consequences

Italy and Japan no longer fetch one item per continuation by default. The
cross-market regression checks that each operation has the same page sizes as
the AU reference while preserving each market's country and language. Queued
commands use the new sizes when executed; no payload or database migration is
needed.

Live read-only probes on 2026-11-09 for Italian product 8182981525641 and Japanese
product 5917111746721 rejected the standard initial query with
MAX_COMPLEXITY_EXCEEDED (cost 1525, maximum 1000), and the 20-variant continuation
with cost 1017. Initial pages of 5 succeeded at cost 919, images of 25 at cost 26,
and variants of 10 at cost 814. These probes are evidence for a subsequent
operation-specific sizing change; this decision only removes the
authentication-based override and retains the existing standard query sizes.
