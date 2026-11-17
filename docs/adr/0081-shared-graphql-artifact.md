# 81. Shared GraphQL offer-details artifact

Status: Accepted

## Decision

Move the identical Anker, Minisforum and Ugreen artifact types into
`retailer_sauce::graphql::{OfferDetails, Page}`. Brand fetchers construct these
shared types directly; their private artifact modules are removed.

## Compatibility

Preserve schema_version, optional context, product JSON and ordered request/response
pages, including Serde defaults and omission of absent context. GraphqlRequest
and the observed MarketContext retain their contracts. Typed raw product parsing,
extraction and acquisition-time handling remain owned by their existing consumers.
No dependency or version changes are required by the local path integration.
