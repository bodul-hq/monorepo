# 76. Ugreen discovery policies

Status: Accepted

Supersedes ADR 72 for `ugreenfr` only. ADR 75 remains in effect.

## Decision

Disable token acquisition for Ugreen France to bypass conflicting storefront
tokens. New fills start with catalog commands. Queued acquisition retries skip
discovery and overrides; catalog/detail requests ignore historical tokens.

NAS AU/UK retain acquisition. Their shared countdown asset uses nested JavaScript
template literals that confused the token scanner. Parse template interpolations
without evaluating JavaScript or accepting dynamic expressions as credentials.
Malformed input and conflicting literal tokens remain errors.

Deploy updated workers before retrying failed acquisition commands. No queue
rewrite is needed. Anonymous GraphQL failures remain visible.
