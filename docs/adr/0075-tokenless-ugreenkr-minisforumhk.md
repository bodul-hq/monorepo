# 75. Tokenless access for Ugreen KR and Minisforum HK

Status: Accepted

Supersedes ADR 72 for `ugreenkr` and `minisforumhk` only.

## Decision

Disable token acquisition for both stores; tokens remain optional. New fills
start directly with catalog commands for their inventory locales. Existing
tokenless handling skips discovery and environment overrides on queued
acquisition retries and ignores supplied tokens in catalog/detail requests.

## Reason and rollout

Discovery fails on a Ugreen KR script URL ending in
`/&default_configuration_id=2896822505` and Minisforum HK's missing
`custom-fields-widgets.js` asset. Both return 404 and block collection.

Deploy updated workers before retrying failed commands. No queue rewrite is
needed. Anonymous API errors still fail normally; skipping discovery does not
guarantee that a live catalog or detail request succeeds.
