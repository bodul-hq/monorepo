# 64. Brand-shared GraphQL integrations

Status: Accepted

Supersedes ADRs 61 and 62 for GraphQL implementation ownership.

Each retailer brand owns one GraphQL engine, operation/schema set and typed raw model in its `graphql` module. Storefront adapters supply immutable configuration and retain fixtures. Known source-field differences use optional typed fields. Unknown fields remain rejected. Anker also owns one legacy Nordic query/response implementation with country and currency configuration.

HTML parsers and extracted representations remain storefront-specific. AU/CA keep their extracted output contracts; existing public paths use aliases and adapters. Commands, events, pagination and authentication policies are unchanged. No database migration is required.
