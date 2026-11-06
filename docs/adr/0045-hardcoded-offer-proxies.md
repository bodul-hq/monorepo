# ADR 45: Hardcoded offer proxies

- Status: Accepted
- Date: 2026-11-06
- Supersedes: ADR 44's environment-based proxy configuration

## Decision

Keep offer proxy URLs in the application's `OFFER_PROXIES` list, initially
`http://10.1.0.102:8888`. Adding proxies requires editing this list and rebuilding.
There is no environment override. Preserve ADR 44's routing and retry behavior.
