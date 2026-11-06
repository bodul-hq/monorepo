# ADR 44: Offer download proxies

- Status: Accepted
- Date: 2026-11-06

## Decision

Offer downloads rotate through direct access and each URL in
`OFFER_DOWNLOAD_PROXIES` (comma-separated HTTP(S) proxy URLs, optionally
containing credentials). Unset or blank selects direct-only. Restart to change
the list. Rotation is shared across workers within one process and starts direct.
Offer routing ignores ambient proxy settings. Failed attempts use normal queue
retries; proxies are not automatically removed. The six-second delay remains.
Sitemaps keep their existing routing. Timing logs identify routes by index.
