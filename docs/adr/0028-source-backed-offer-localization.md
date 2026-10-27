# 28. Source-backed target market and language

## Status

Accepted. Extends ADR 19's exports and ADRs 21/22/25's event contracts.

## Date

2026-10-26

## Context

A retailer's code, currency, payment settings and shipping destinations do not
establish the sales market or current page language. The downloaded source must
supply the facts independently of later processing.

## Decision

The shared facade captures document `html[lang]` and standalone top-level Shopify
country/locale literal assignments without executing scripts. Document language
outranks the Shopify language fallback. Invalid or conflicting applicable values
remain unknown. Payment configuration, widgets, defaults and alternate pages are
excluded. The [coverage survey](../localization-source-coverage.md) defines the
implemented source families and known gaps.

Keep raw declarations, paths and diagnostics in an extraction sidecar. Detach it
before strict retailer-schema decoding and reattach it after each stage. Copy
selected facts into the variations represented by that page. Immutable source
references retain evidence alongside the selected facts.

`localization` has independently nullable target market and language. Countries
use the shared ISO country type. Regions have bounded explicit labels. Language
uses ICU's supported Unicode BCP47 syntax, preserving source spelling; unsupported
forms are unknown during extraction and errors at typed boundaries.

Export v4 adds a required observation; v1–v3 retain their original shapes.
Creation history advances to v4, retaining earlier readers. A missing import
object means no observation; an explicit object replaces both facts, including
known-to-unknown transitions. A null object is invalid. Emit one
`OfferLocalizationChanged.v1` after existing changes when selected facts change.
Projections apply the decided observation. Customers accepts the data without
changing its cards, document language or money formatting.

## Consequences

Deploy compatible clients and customer-facing, then offers and sourcing. No
historical backfill, market guessing or shipping-eligibility inference is added.
Distinct observations still apply in processing order. Metadata sequences alone
do not establish source-order protection.
