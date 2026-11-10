# ADR 56: Operation-specific Anker initial-fill page sizes

- Status: Accepted
- Date: 2026-11-10
- Supersedes: ADR 55's initial-query and variant-continuation sizes.
  Page sizes remain independent of authentication.

## Context

Live Italian and Japanese probes rejected the broad initial query at its former
sizes (cost 1525, maximum 1000) and the continuation requesting 20 variants
(cost 1017). Initial connection pages of 5 succeeded at cost 919, image pages
of 25 at cost 26, and variant pages of 10 at cost 814. The separate operations
need different sizes while preserving their selected fields.

## Decision

For every supported retailer, use these page sizes:

| Operation                                       | Page size                                         |
| ----------------------------------------------- | ------------------------------------------------- |
| Initial OfferDetails                            | 5 per selected connection, including nested lists |
| ProductCollections, ProductImages, ProductMedia | 25                                                |
| ProductVariants                                 | 10 variants, with nested pages of 5               |
| ProductSellingPlanGroups                        | 20 groups, with selling-plan pages of 5           |
| Variant connection continuations                | 5                                                 |
| SellingPlans                                    | 5 plans within one selected group                 |

Catalog limits remain 250 IDs per HTTP request and 500 IDs per command.
Retain all fields, complete cursor pagination, authentication and request pacing.
Query documents select their appropriate size variables; the shared builder
supplies the same values for every retailer.

## Consequences

Initial requests have smaller pages, while dedicated continuations fetch more
items per request. The cross-market regression verifies the exact page-size
variables for every operation. Existing queued commands use these settings
after workers restart; no payload or database migration is needed. The live
probes establish acceptance for the tested products, not every possible catalog.
