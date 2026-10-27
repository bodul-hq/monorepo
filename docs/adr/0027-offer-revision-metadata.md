# 27. Preserve Testimony revisions through Mulac

## Status

Accepted. Extends ADR 15's typed handler and ADR 21's outbox metadata.

## Date

2026-10-26

## Context

Testimony already assigns and stores per-offer sequence numbers. Returning raw
events to Mulac discarded those revisions at the command-handler boundary.

## Decision

The handler captures pending envelopes before saving, then returns transparent
transport adapters containing each event and its exact revision. Mulac's optional
`ApplicationEvent::sequence()` hook copies that number into event metadata.
Framework-owned IDs, correlation and causation retain their existing behavior.

Preserve sequence through event reservation, subscriber dispatch and outbox JSON.
Legacy and unrelated callers may omit it. Validate positive integers up to
PostgreSQL BIGINT's maximum, and reserve capacity for a complete aggregate update
before mutation. No-op imports consume no revisions. Each offer starts at one.

## Consequences

This needs the sibling Mulac optional-sequence API. Deployment must include that
prerequisite; Cargo path dependencies do not pin a revision. Existing queued
metadata remains readable without backfills. The HTTP inbox envelope still does
not forward arbitrary event metadata. Sequence does not enforce customer update
ordering or close the existing Testimony-save/Mulac-publication failure window.

The implemented Mulac prerequisite is commit `dc60f443e4a2ed4a3e763855fe473426732baef1`. Cargo path
dependencies use the checkout contents; the revision is recorded for reproducibility.
