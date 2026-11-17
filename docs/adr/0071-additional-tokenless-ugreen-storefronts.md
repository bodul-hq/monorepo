# 71. Tokenless access for additional Ugreen storefronts

Status: Accepted

Extends ADR 70 and supersedes ADR 60's optional-token acquisition policy for
ugreenin, ugreenuk and ugreennl.

## Decision

Set acquire_token to false for these three retailers; token_required remains
false. Apply the existing tokenless behavior: skip discovery and overrides,
start new fills at inventory market discovery, and ignore historical queued
tokens when issuing GraphQL requests.

## Reason and rollout

These storefronts have user-reported token issues matching the earlier group.
Deploy updated workers before retrying failed commands. No queue rewrite is
needed. Existing policy and historical-token tests now cover all eight stores.
