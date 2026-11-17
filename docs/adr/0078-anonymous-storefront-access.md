# 78. Anonymous access for nine additional storefronts

Status: Accepted

Supersedes ADR 72 for the stores below. ADRs 75–77 remain in effect.

## Decision

Disable token acquisition for `minisforumkr`, `ugreeneu`, `ugreenin`,
`ugreennas`, `ugreennasca`, `ugreennaseu`, `ugreennl`, `ugreenuk` and
`ugreenus`. Tokens remain optional. New fills start at catalog collection;
queued acquisition retries skip discovery and overrides. Catalog/detail
requests ignore historical tokens.

## Evidence and rollout

Acquired tokens lack the pickup-location scope required by `storeAvailability`.
Anonymous checks on 2026-11-15 passed for all 13 configured locale contexts:
13 catalog requests and 26 initial product-detail requests returned HTTP 200
without GraphQL errors. Pickup connections were present and empty. Five exact
failed product/locale combinations from Ugreen EU and India also passed.
These sampled requests do not verify complete catalog fills or all pagination.

Deploy updated workers before retrying failed commands. No queue rewrite is
needed. NAS AU/UK retain token acquisition and their template-parser fix.
