# ADR 36: Offer description content

Status: Accepted; extends ADRs 19, 21, 22 and 23.

Date: 2026-10-26

## Context

Normalized products already retain some primary descriptions, but offers do
not carry them beyond structuralization. The shared Content scaffold supplies
a home for that description without moving titles or images.

## Decision

Use `retailer_sauce::structuralized::content::Content` with exactly one optional
`description: String`. Carry the existing primary product description, retaining
its text, HTML and whitespace. Do not add an SEO fallback or infer descriptions.
Titles and images remain flat and preserve their current behavior.

Export v8 and `OfferCreated.v8` add `content`. A legacy omitted object means no
observation. Present content requires `description`; `{description:null}` clears
it. Reject `content:null`, missing description in a present object, wrong types
and NUL text. `OfferContentChanged` carries the full object and is stored as
`OfferContentChanged.v1`.

Each projection stores description as nullable TEXT. REST and GraphQL expose a
nonnull content object with nullable description. Customers default absent
legacy content to an empty object in their model and do not render descriptions.

## Consequences

The description remains source data rather than trusted markup. Replay and
immutable exports preserve exact values. Existing offers remain unknown until
a source observation supplies or explicitly clears the description.
