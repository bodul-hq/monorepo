# 85. Remove storefront raw facades

Status: Accepted

## Decision

Remove storefront offer_details/v2/raw.rs wrappers and their fixture/corpus tests,
along with raw prelude aliases and type-sharing tests. Remove v2 module declarations
where the module becomes empty. AU/CA extracted-model facades remain.

Consumers import retailer_sauce::shopify::graphql::raw directly. This supersedes
ADR 84's retention of storefront raw facades. Shared raw-model tests and extraction
tests remain; fixtures used by extraction tests are retained. Stored data and
production extraction behavior do not change.
