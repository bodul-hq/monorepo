# ADR 31: Source locale market fallback and primary language

Status: Accepted; supersedes ADR 28's full-tag selection policy and item 19's
no-derivation rule only. Other source exclusions, wire contracts and replay rules remain.

Date: 2026-10-26

## Context

A current page locale such as `it-IT` provides an explicit country region when
no direct market declaration exists. New observations should use its primary
language while retaining the complete source declaration for review.

## Decision

Only extraction interprets a source locale. Parse the complete supported ICU
Unicode BCP47 locale and compare canonical full identities before reducing the
language. Document language retains precedence over standalone Shopify locale
literals. Emit the lowercase primary language for every valid new observation.

Distinguish absent, accepted, invalid and conflicting market selection. Only
absence permits fallback to the selected locale's language-identifier region,
validated with CountryCode (including UK to GB). Explicit countries/regions win;
invalid, unsupported, blank, dynamic or conflicting market declarations block
fallback. A locale conflict keeps language unknown and blocks derivation.

Do not maximize likely subtags or derive countries from Unicode extensions,
URLs, retailer codes, currencies, payment settings or widgets. Numeric regions,
EU, ZZ and unsupported countries stay unknown. `und` remains undetermined;
`und-IT` can provide country IT. Script, variants and private-use subtags do not
change primary-language extraction. Retain raw full tags, paths and explicit
derivation/blocking reasons in the sidecar.

## Consequences

New extractions flow through the existing export/event/projection contracts.
No schema migration or version bump is needed. Generic deserialization, export
conversion and replay preserve old full-tag observations and immutable bytes.
New observations use the existing localization-change event; identical reimports
emit none. No corpus, projection, queued payload or history is rewritten.
Rollback changes future extraction only. Historical correction requires a
separate source-backed workflow with fresh artifact and delivery identities.
