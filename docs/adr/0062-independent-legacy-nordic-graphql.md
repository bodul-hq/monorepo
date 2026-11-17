# 62. Independent legacy Nordic GraphQL sources

Date: 2026-11-10

Status: Accepted

Supersedes the shared query/raw implementation ownership in ADR 47.
Complements ADR 61's independent initial-fill integrations.

## Context

The older sitemap-driven Nordic download path still shared one query and raw
model after initial-fill integrations were separated. That would continue to
couple Sweden, Norway, Finland, Iceland and Denmark when changing a source.

## Decision

Each Nordic retailer owns `offer_details/v1/graphql`: its endpoint, query,
concrete raw types, parser, validation and mappings, plus its own captured
response. Queries begin as identical physical copies, with no shared imports or
build-time synchronization. The existing `NordicMarket` API is a compatibility
router delegating to those independent implementations.

The existing extracted and normalized wire formats stay unchanged. The
normalized `Document` and `Variant` representations remain shared: they are
outputs, not GraphQL source types. Sitemap classification remains outside this
GraphQL ownership change. Source isolation tests cover both legacy and v2 raw
models and reject shared query/raw imports.

## Consequences

The older download flow now has the same retailer independence as initial fill.
A country can change its query or response schema without changing another
country's implementation. Existing download identities, endpoints, token
configuration, normalized prices and localization stay compatible.
