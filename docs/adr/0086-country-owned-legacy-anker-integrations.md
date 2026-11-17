# 86. Country-owned legacy Anker integrations

Date: 2026-11-16

Status: Accepted

Supersedes ADR 47's retained aggregate processing integration and ADR 62's
shared normalized models and `NordicMarket` compatibility router.

## Context

The five active Nordic countries still depend on shared legacy GraphQL and
sitemap modules. The obsolete `ankernordicscom` HTML integration also remains
callable for historical work, despite the separate country codes.

## Decision

Each of `ankerse`, `ankerno`, `ankerfi`, `ankeris` and `ankerdk` owns its legacy
query, raw and normalized types, parsing, validation and mappings under
`offer_details/v1/graphql`, and its sitemap code under `sitemap/v1`. These are
independent physical copies with no synchronization or shared Nordic parent.
Remove `nordic`, `graphql/nordic` and the `ankernordicscom` integration.

The brand registry exposes `LegacyOfferSource` and `legacy_offer_source_for`.
The extraction facade uses this interface to construct and validate downloads
and run extraction, normalization and structuralization. The registry only
selects the country implementation; it owns no source model or query.

Retain the old `AnkerNordicsCom` enum and slug for historical deserialization.
Do not map them to Sweden or migrate existing data. Architecture and extraction
lookups report unsupported processing; normalization, structuralization and
sitemap lookups return no integration. Download construction and validation
return explicit errors, so queued work cannot fall back to a generic HTML GET.

Preserve the active countries' raw, normalized and structuralized formats,
country/currency contexts, URLs, endpoint and token configuration. The v2
initial-fill pipeline and its shared `retailer_sauce` GraphQL engine are
unchanged.

## Consequences

Legacy country sources can evolve independently, including their normalized
representations and sitemap rules. Identical code is intentionally duplicated.
Registry and facade tests cover country isolation, exact prices, incomplete or
invalid responses, URL guards and the retired code's unsupported processing.
Historical aggregate records remain readable, while their queued processing
fails explicitly until separately resolved.
