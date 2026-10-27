# ADR 23: Customer-facing offer browsing and the customers homepage

- Status: Accepted
- Date: 2026-10-25

## Context

Customer-facing owns a projection populated by explicit inbox and command
processing. Customers needs a browsable homepage without JavaScript. The
offers service's database is not a public read dependency.

## Decision

Expose paginated offers through REST and GraphQL using one repository and
validation path. Run Diesel off the asynchronous runtime. Both transports
default to 30 records, allow at most 100, use nonnegative 32-bit offsets, and
sort by creation timestamp then UUID, descending. An additive index supports
that ordering. Fetch one extra row to report more results without a total count.

Return original offer properties. Amounts are decimal strings of signed i64
minor units in both APIs, avoiding precision loss and GraphQL Int limitations.
Retailer codes are lowercase and timestamps use UTC RFC3339. Follow the
service's existing unauthenticated access convention and sanitize read errors.

The customers app owns a private server-side REST client with a configured
base URL, bounded timeout, no automatic retries and no redirects. Each page
renders up to 30 cards with ordinary pagination links, HTML and local CSS.
Format prices with integer arithmetic and shared currency exponents. Escape
dynamic values and omit unsafe outbound links. Distinguish successful empty
results from unavailable upstream responses with separate pages and statuses.

## Consequences

The homepage depends only on customer-facing's HTTP contract. Updates appear
after queue processing and refresh; reads never advance queues. Offset pages
are deterministic within a request but are not snapshots across requests:
new offers can shift later pages, and deep offsets still require skipping rows.

No browser scripts, frontend build pipeline, backfill, imagery, search,
filtering, currency conversion or background worker is added. ADRs 1 and 22
remain valid; this decision supplies the deferred browsing interface.
