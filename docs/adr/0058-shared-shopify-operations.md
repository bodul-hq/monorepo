# 58. Shared Shopify operations

Date: 2026-11-10

Status: Accepted

## Context

Minisforum and Ugreen need the same Shopify initial-fill queries and raw source
models as Anker, across every advertised country/language combination.

## Decision

Move the reusable query documents, generated response validation, and raw source
models into `retailer_sauce::shopify`. Keep Anker compatibility exports and its
retailer-specific configuration and regression fixtures in `retailers_anker`.
Move the graphql_client dependency with the operations.

Add a shared market-discovery request and parser. Countries supply their own
available languages and currency; do not cross-join global language lists.
Sort and deduplicate contexts and reject empty, invalid, conflicting or errored
responses. These helpers do not enqueue commands or change existing Anker runs.

## Consequences

Retailer crates can share Shopify behavior without depending on the Anker crate.
Durable market fan-out, contextual identities, artifact versioning and retailer
registration remain separate implementation tasks tracked in TODO.md.
