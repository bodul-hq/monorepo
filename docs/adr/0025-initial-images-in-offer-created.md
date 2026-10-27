# ADR 25: Initial images in OfferCreated

## Status

Accepted. Supersedes ADR 24 only for image events emitted during creation and
for the corresponding creation payload and projection behavior.

## Date

2026-10-25

## Context

Mulac batches do not guarantee event processing order. Emitting initial images
as separate events allowed them to reach the offers projection before their
parent, causing foreign-key failures and retries during ordinary imports. The
user requested that initial images travel in the creation payload instead.

## Decision

Creating an offer emits exactly one `OfferCreated` with an ordered `images`
string array, including `[]`. Imports still filter invalid URLs and preserve
the first occurrence of each exact URL. Aggregate state initializes with those
images. Only updates emit `OfferImageAdded` for previously unseen URLs, using
positions after the images already saved. Title and price event order stays
unchanged; omitted, reordered or repeated images do not clear or add images.

The public routing key remains `OfferCreated`. New Testimony records use
`OfferCreated.v2`. Readers also accept `OfferCreated.v1` and interpret a missing
image field as an empty array. Historical creation-plus-image-event streams
continue to replay. Existing stored records and queued messages are not rewritten.

Offers saves the parent and its initial images in one database transaction.
Customer-facing saves the parent, initial images and original-message receipt
in its existing transaction. Event consumers validate the complete ordered
unique image array; malformed event data fails instead of being filtered.
Update image events retain their existing validation, identity and retry rules.

## Consequences

New creations no longer depend on separate initial image events reaching the
projection after a parent creation event. Later updates may still arrive before
creation and use the existing retry policy. Previously queued image events also
retain that policy; this change does not reset or drain operational queues.

Deploy customer-facing before offers. Older consumers silently ignore unknown
creation fields, so producing the new payload before upgrading them would lose
initial images in their projection. Older offers binaries cannot replay the new
stored `OfferCreated.v2` tag. No migration or dependency change is required.

The current typed offers import handler is retained. Its identity-projection and
history/publication limitations remain outside this fix.
