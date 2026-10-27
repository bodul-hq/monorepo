# ADR 33: European Union locale market

Status: Accepted; supersedes ADR 31's exclusion of EU locale regions only.

Date: 2026-10-26

## Context

Anker's saved `/eu-en/` product pages declare `html[lang="en-EU"]`. The user
confirmed that these observations should carry the European Union market and
English language. EU is a regional market, not a country accepted by CountryCode.

## Decision

When no applicable explicit market is declared, extraction maps the selected
locale's parsed `EU` region to `{"kind":"region","label":"European Union"}`.
Its primary language is retained (`en-EU` produces `en`). Apply this to the
existing eligible document and standalone Shopify locale declarations, using
canonical region parsing and preserving the complete raw locale and source path
with a `selected locale-derived regional market` diagnostic.

Explicit valid markets still win. Invalid, dynamic or conflicting explicit
markets block fallback. Do not reinterpret `Shopify.country="EU"` as a country,
infer anything from `/eu-en/` URLs, or derive a market from Unicode extensions.
Other unsupported and numeric regions remain unknown. Country fallback and
locale precedence remain as specified in ADR 31.

## Consequences

The existing regional-market contract carries the result through export,
events, projections and REST/GraphQL without a migration or version change.
This applies only to new extractions. Old sidecars, exports and historical
events retain their original facts. Updating existing offers requires a new
source observation through the normal pipeline; no historical data is rewritten.
