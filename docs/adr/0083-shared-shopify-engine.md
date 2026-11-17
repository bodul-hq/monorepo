# 83. Shared Shopify acquisition engine

Status: Accepted

## Decision

Move identical catalog pagination, detail fetching and typed query operations into
retailer_sauce::shopify::graphql with the shared Engine and Operation types.
Move locale request contracts and inventory context generation from retailers_wip
into retailer_sauce::initial_fill, preventing a reverse dependency on consumers.
Brands retain configurations, raw models, extraction and integration tests.

## Compatibility

Query documents, schema, serialized contexts and artifacts remain unchanged.
Preserve pagination limits, repeated-cursor detection, owner and currency checks,
context evidence validation and token redaction. Use the existing Ugreen/Minisforum
policy of ignoring historical tokens when acquisition is disabled and optional.
Current Anker configurations retain their token behavior. Brand Engine/Operation
exports point to the shared implementation. No queue migration is required.

This supersedes ADR 61's ownership of these shared acquisition implementations;
brand-owned source interpretation remains in effect.
