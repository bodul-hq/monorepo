# 60. Optional public Storefront token acquisition

Date: 2026-11-10

Status: Accepted

Supersedes the token-free start policy for Minisforum/Ugreen in ADR 59 and
Anker Italy/Japan's direct catalog start policy.

## Decision

New initial fills attempt AcquireStorefrontToken first, including all 31
Minisforum/Ugreen retailers and Anker Italy/Japan. Existing required-token
Anker configurations remain strict and retain regional map selection.

For the previously token-free stores, first check the optional environment
variable `<UPPERCASE_RETAILER_CODE>_STOREFRONT_TOKEN`. Otherwise inspect inline
and allowed external storefront scripts for explicit public fields:
storefrontAccessToken, storefrontApiAccessToken, storefrontToken and
storefront_access_token. Ignore empty fields and references; reject conflicting
nonempty values. Read literal data without evaluating scripts. Do not use
unrelated keys such as adminToken. Credentials remain redacted in logs.

If a successful lookup finds no token, continue token-free as requested. Lookup
network/parse errors, invalid configured credentials, conflicting candidates,
and subsequent API authentication errors remain explicit failures rather than
being silently interpreted as absence.

Keep the existing StorefrontTokenAcquired event name for compatibility, but its
storefront_token is now nullable for optional-token stores. Historical string
values still deserialize. Tokens propagate through market discovery, catalog
pagination and detail commands. Acquisition events for multi-market retailers
queue discovery only; fixed-market retailers queue their catalog directly.
Historical discovery commands/events without a token remain readable and use
public access. Upgrade all consumers before producing nullable-token events.

Query fields, page sizes and pacing remain unchanged. Shopify documents a 1,000
complexity limit for tokenless access; a discovered token must be verified live
before larger authenticated query sizes are adopted. Token discovery from saved
HTML proves the extraction shape, not present-day validity or scopes.

## Validation and rollout

Sanitized captured script fixtures cover nine stores with nonempty token fields.
Tests cover empty/no-token fallback, required-token failures, conflicting values,
and token propagation through discovery and catalog fan-out. New fills use the
new start policy; already queued public work is not rewritten. No database
migration or private/Admin API credentials are required.

Reference: https://shopify.dev/docs/api/storefront/latest
