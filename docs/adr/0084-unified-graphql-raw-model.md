# 84. Unified raw Shopify GraphQL model

Status: Accepted

## Decision

Use retailer_sauce::shopify::graphql::raw for all three brands, based on the
Anker superset. Remove brand graphql::raw modules and update callers directly.
Storefront v2 raw facades continue exposing the shared model.

All brands now accept optional inventory, metafield and expanded-reference fields.
Unknown fields remain rejected. Preserve schema/context consistency checks, money
representations, legacy selling-plan handling and recorded response evidence.
Minisforum/Ugreen extraction explicitly ignores the newly accepted optional fields,
retaining its existing output contract. Anker retains its current conversions.

Move all three fixture suites into the shared crate, including Anker expanded data.
This supersedes ADR 83's brand ownership of the raw model; brand extraction and
configuration remain local. No stored artifact migration is required.

Brand money conversion uses a local function because both the shared raw Money
and the destination money::Money are external types. Existing precision, currency
and overflow checks remain unchanged and covered by the original tests.
