# 90. Deliver v2 structuralized variants through the offers outbox

Status: Accepted

## Context

Initial fills produce immutable artifacts identified by run and object. The
offers inbox accepts one import per retailer and variation URL. The sitemap
export path identifies its source by offer and consolidated sitemap, which a
v2 initial fill does not produce.

## Decision

Subscribe to `OfferDetailsExtracted` and queue a structuralization command
keyed by the extracted object. Run the retailer's v2 structuralizer and pin its
output in `offer_details_structuralized`. Publish `OfferDetailsStructuralized`
after the write succeeds.

A subscriber reads that object and maps its variants to the existing offers
import contract. Only variants with a storefront URL and known price qualify.
Record the messages in one outbox transaction. Derive each delivery ID from
the structuralized object ID and the numeric variant ID.

## Consequences

Replaying a notification reuses the same command or delivery IDs. A fresh
observation has its own artifact and can produce a new delivery. The existing
outbox publisher handles HTTP submission, acknowledgment and retry.
