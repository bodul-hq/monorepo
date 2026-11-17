# 70. Tokenless access for selected storefronts

Status: Accepted

Supersedes ADR 60's optional-token acquisition policy for minisforumhk,
ugreennasau, ugreennasuk, ugreenfr and ugreeneu only.

## Decision

Set acquire_token and token_required to false for these five configurations.
Their new fills start at inventory market discovery without acquiring a token.
The API validates the configured process instead of unconditionally requiring
AcquireStorefrontToken. Required-token configurations must enable acquisition.

For these tokenless configurations, access resolution returns None before reading
overrides or downloading scripts. Brand HTTP execution ignores supplied tokens,
including credentials in historical queued catalog/detail commands. Old queued
acquisition commands therefore retry without repeating discovery failures. Other
retailers retain their existing policies. No runtime environment switch is added.

## Reason and rollout

Discovery encounters a missing Minisforum HK theme asset, parser failures in
third-party Ugreen NAS scripts, and conflicting Ugreen France app tokens. Ugreen
EU's discovered token lacks selling-plan access; the same detail query succeeds
without it. Anonymous access can still fail for other upstream reasons, which
remain visible through the improved GraphQL diagnostics.

Deploy updated workers before retrying failed commands. No queue rows, schemas,
artifacts or schedules are rewritten. New fills and old queued token payloads are
covered by regression tests; PostgreSQL fixtures verify the 48-retailer flow.
