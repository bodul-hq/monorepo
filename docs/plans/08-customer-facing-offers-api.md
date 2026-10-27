# Item 8: Customer-facing offers API

Status: completed and verified on 2026-10-25.

## Goal and shared contract

Expose the existing customer-facing `offers` projection through REST and
GraphQL. [Item 9](09-customers-offers-homepage.md) consumes this contract.
Both plans were aligned before implementation at monorepo revision
`b0688687f184617a5b3501dd89f5104f3ae53c9e`; local Mulac revision is
`9474319fff2237d81828a23d97df5d2f97741858`.

- REST: `GET /api/offers?limit=30&offset=0`.
- GraphQL: `offers(limit: Int! = 30, offset: Int! = 0): OfferPage!`.
- Limit is 1–100, offset is 0–2147483647. Reject invalid values.
- Order by `created_at DESC, id DESC`; fetch one extra row to determine
  `has_more`, returning at most the requested limit. No total count.
- Page fields: `items`, `limit`, `offset`, `has_more` (`hasMore` in GraphQL).
- Offer fields: `id`, `retailer_code`, `title`, `price`, `url`, `created_at`.
  GraphQL uses `ID`, `retailerCode`, and `createdAt`.
- Price contains `amount`, a decimal string of signed i64 minor units, and
  `currency`, an uppercase ISO code. Retailer codes are lowercase. Timestamps
  are UTC RFC3339. Titles and URLs preserve stored values.
- Empty and out-of-range pages succeed. Invalid pagination returns REST 400
  or GraphQL errors. Read failures return sanitized REST 500 or GraphQL errors.
- Public read routes use the existing unauthenticated access convention.
  Offset pagination is deterministic within a query, not a snapshot across
  requests: newly inserted offers may shift subsequent pages.

## Implementation

Use one service-owned read implementation with an injectable repository and
`spawn_blocking` for Diesel work. Add an additive, reversible composite index.
Do not change projection writes, queue processing, or upstream services.
Document the contract in OpenAPI, the service README and Bruno examples.
Record the API and server-rendered consumer boundary in a new ADR.

## Acceptance and verification

Test defaults, invalid pagination, empty/partial/full pages, equal-timestamp
ordering, REST/GraphQL parity, full i64 fidelity and sanitized failures.
Extend isolated PostgreSQL tests for index migration and visibility only after
inbox and command processing, including subsequent updates.

Run services formatting and CI, customer-facing `make test-postgres`, and
live health, REST and GraphQL checks using `PORT=3103 make serve`. Audit only
if workspace dependencies or its lockfile change. Record actual results before
marking this plan and TODO complete.

## Completion evidence

- Implemented the shared REST/GraphQL contract, blocking PostgreSQL repository,
  and reversible browsing index. Added OpenAPI, README and Bruno documentation
  and ADR 23 without changing projection writes or queue behavior.
- `make -C services fmt` and `make -C services ci` passed. All 13 customer-facing
  PostgreSQL tests passed in isolated test schemas, including the new browsing
  and migration case. Services dependencies and lockfile did not change.
- Started both binaries with `PORT=3103/3104 make serve` against a generated
  schema in `customer_facing_test`. Verified health, OpenAPI, REST, GraphQL,
  queue-stage visibility, 31 projected offers split 30/1, and title/price updates
  on the homepage. Servers stopped and only the generated schema was removed.
- Local verification logs: `/tmp/bodul-offers-services-ci.log`,
  `/tmp/bodul-offers-postgres.log`, and `/tmp/bodul-offers-live.log`.
