# 72. Restore Storefront token acquisition

Status: Accepted

Supersedes ADRs 70 and 71 for storefront access policy.

## Decision

Restore acquire_token to true for minisforumhk, ugreennasau, ugreennasuk,
ugreenfr, ugreeneu, ugreenin, ugreenuk and ugreennl at the user's request.
All storefronts now enable acquisition. Token-required settings are unchanged.
Ugreen US and Minisforum Japan already have acquisition enabled.

New fills acquire tokens before loading inventory markets. Queued acquisition
commands resume normal discovery, and supplied catalog/detail tokens are used.
Deploy updated workers to apply the restored configuration.
