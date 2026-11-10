# 59. Multi-market Shopify initial fills

Date: 2026-11-10

Status: Accepted

## Context

All ten Minisforum and 21 Ugreen retailers, including NAS stores, need GraphQL
initial fills for every advertised country/language combination. Country and
language affect product visibility, prices and content; product IDs alone cannot
identify the resulting artifacts. Saved data3 endpoint and localization evidence
covers these stores. Live verification is separate from captured evidence.

## Decision

Keep one run per retailer in the existing bulk API. These 31 configurations
start DiscoverStorefrontMarkets, recording the discovered context list in a
StorefrontMarketsDiscovered event. Each advertised country's availableLanguages
creates its own catalog command with country, language and currency. Do not
cross-join unrelated language lists. Discovery errors prevent fan-out.

Carry optional context through catalog/detail commands and events. Contextual
commands include country/language in stable identity keys; old payloads without
context retain historical identities. Each continuation uses the same context.
Validate explicit context evidence and currencies; do not silently accept locale
fallback. Commands retain independent queue retry behavior. Existing Anker flows
remain fixed-context and use their existing authentication policies and pacing.

The new stores use public access backed by captured responses. Authentication
errors are explicit; there is no automatic authentication fallback. Read-only live
verification must be repeated in an environment with working external DNS.

Contextual artifacts use schema_version 2 and include context alongside product
and original pages. Context-free artifacts remain schema_version 1. Raw readers
accept only these version/context combinations. Extracted types are unchanged.

Development listings expose context. The dumper uses
`<retailer>/offer_details_v2/sources/<COUNTRY>-<LANGUAGE>/<product-id>.json` for
contextual artifacts and retains the flat layout for historical artifacts.
Newest-copy selection therefore operates separately in every market.

## Consequences

Multi-market fills may create substantially more work than fixed-context fills.
Discovery snapshots are persisted and fan-out IDs are deterministic for replay.
No database schema migration is needed: context resides in existing JSON payloads.
Consumers/readers must be upgraded before starting contextual fills. Existing
Anker runs, queue concurrency, scheduling and HTML extraction remain unchanged.

The development listing still depends on terminal-event retention and stable
source sets for offset pagination, as documented in ADR 57. Product fixtures and
localization responses from all 31 stores are checked in; full external corpus
checks are opt-in and do not treat uncaptured contexts as unsupported.
