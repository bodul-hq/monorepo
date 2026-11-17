# 77. Tokenless access for Minisforum JP

Status: Accepted

Supersedes ADR 72 for `minisforumjp` only. ADRs 75 and 76 remain in effect.

## Decision

Set `acquire_token` to false; `token_required` remains false. New fills start
with catalog commands. Queued acquisition retries skip discovery and overrides;
catalog/detail requests ignore historical tokens.

## Reason and rollout

The discovered token lacks `unauthenticated_read_product_pickup_locations`,
causing `storeAvailability` to fail across product-detail requests. Anonymous
requests using the full current query with JP/JA context succeeded for products
6805343436966 and 8513011417254: HTTP 200, no GraphQL errors, and empty
`storeAvailability` connections. This verifies those requests, not a full fill.

Deploy updated workers before retrying failed commands. Existing queued detail
commands omit their old token on execution; no queue rewrite is needed.
