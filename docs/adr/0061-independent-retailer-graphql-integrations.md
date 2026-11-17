# 61. Independent retailer GraphQL integrations

Date: 2026-11-10

Status: Accepted

Supersedes the shared GraphQL implementation ownership in ADR 58 and the shared
raw representation ownership in ADR 57. Their wire formats remain unchanged.

## Context

Storefronts can diverge even within one brand or Shopify origin. A shared query,
raw product type or pagination implementation couples their changes and prevents
independent field selection, token policies and source interpretation.

## Decision

Each of the 48 supported retailer codes owns its integration inside its existing
brand crate. `<retailer>/initial_fill/v1` contains its configuration, token
scanner and asset policy, market parsing, query documents and schema snapshot,
request builder, validation, catalog traversal, and detail pagination/merge.
`<retailer>/offer_details/v2/raw.rs` defines concrete source types, including all
nested types, with fixtures captured for that retailer. Nordic fixtures come
from the corresponding country context of the shared Nordic storefront.

These are independent source files. Initial copies may be identical; they are
not maintained through shared implementation macros, symlinks or generation
from a common query/schema. Changes must be made and tested for each affected
retailer. HTML v1 and extracted representations are outside this decision.

The extractor facade selects an object-safe synchronous `InitialFillIntegration`
by retailer code. Its shared contracts are configuration values, credentials,
HTTP request envelopes, market contexts, catalog identifiers/cursors and opaque
serialized artifacts. Retailer-local operation enums do not cross that boundary.
The HTTP adapter handles routing and pacing; the service retains commands,
events, replay identities, logging and object storage.

The migration preserves required/optional authentication policy, token-free
fallback on successful discovery without a token, fixed Anker contexts and
advertised Minisforum/Ugreen contexts, query page sizes, command/event payloads,
artifact versions 1/2 and development dump paths. No database migration is needed.

## Consequences

Each retailer can change independently, including stores sharing an origin.
Source duplication and compilation cost increase deliberately. Isolation tests
check distinct raw type identities and reject cross-retailer implementation
imports. Each retailer has local query/validation and artifact tests plus real
raw fixtures. External saved-corpus tests remain opt-in. Shared infrastructure
must not grow product fields or retailer-specific parsing back into its boundary.
