# Record completed offer exports in the sourcing outbox

Status: Completed. Covers TODO item 4. See the sourcing README's
“Recording export messages” for the implemented behavior.

## Goal and completion boundary

Processing an `OfferExportWasPrepared` event durably records one outgoing
import message for every importable entry in that immutable export. Completion
means the messages are available to the existing sourcing publisher, including
after failures and replay. Offers need not be running.

Items 1, 3, and 5 are completed. This task does not depend on items 6 or 7.
It does not include HTTP publication, downstream queue execution, export
reconversion, or automatic historical backfill. Demonstrate completion using
new or explicitly replayed export events.

This handoff specifies observable requirements and existing contracts.
Implementation design and code organization belong to the implementing tool.

## Inspected baseline and evidence

Inspected monorepo revision: `5315da8f36e50be3d70d0da7f873f34262992a6c`.
Sibling Mulac revision: `9474319fff2237d81828a23d97df5d2f97741858`.
Cargo path dependencies use the actual sibling checkout, not a pinned revision.
Recheck relevant contracts if either checkout changes.

References describe existing behavior, not prescribed edit locations:

- [Sourcing README](../../services/retailer_sourcing/README.md), particularly
  “Processing for export” and “HTTP outbox delivery.”
- [Export contract](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/entity.rs),
  [event](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/event.rs),
  and [completed export storage](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/store.rs).
- [Existing HTTP publisher](../../services/retailer_sourcing/src/platform/offers_publisher.rs)
  and [import fixture](../../services/retailer_sourcing/tests/fixtures/offer_export/import_payload.json).
- [ADR 19](../adr/0019-immutable-offer-export-preparation.md) and
  [ADR 20](../adr/0020-temporary-http-outbox-delivery.md).
- Local Mulac `libs/outbox/src/assembly/domain.rs` defines the outbox metadata.

Historical plans 3 and 5 contain pre-implementation assumptions. Current code
and accepted ADRs establish the implemented boundaries.

## Existing input contract

`OfferExportWasPrepared` contains:

| Field | Meaning |
| --- | --- |
| `export_id` | Completed immutable export UUID |
| `schema_version` | Export contract version; currently `1` |
| `source` | Sourcing offer UUID, consolidated sitemap UUID, retailer code, source URL, and immutable structuralized-object reference |

The event envelope separately supplies event identity and optional correlation,
causation, and source metadata.

A completed `OfferExport` contains `id`, `schema_version`, `source`,
`entries`, and `exclusions`. Each entry has a stable UUID, variation index,
and import payload. Exclusions record a variation index, URL, and reason;
the current reason is `missing_price`.

Completed export bodies reside in the `offer_export` object-storage bucket.
The existing export loader reads completed snapshots by UUID and validates
their stored object reference. Pending preparations and the latest
structuralized document are not the completed snapshot named by an event.
The source's storage reference identifies the structuralized input; it is
distinct from the completed export's storage reference.

An empty or wholly excluded export is valid and produces zero messages.
Export preparation already determines importable entries. Priced unavailable
stock is included; missing prices are excluded. Entry IDs are delivery IDs,
not the receiving offers service's business IDs.

## Required output contract

| Outbox value | Required meaning |
| --- | --- |
| Row ID | Stable export-entry UUID |
| `message_id` | Same stable export-entry UUID |
| Payload | That entry's existing import payload |
| `routing_key` | `offers` |
| `event_type` | `OfferExportWasPrepared` |
| `event_id` | Original event-envelope UUID |
| `correlation_id`, `causation_id`, `source` | Original event-envelope values |
| `content_type` | `application/json` |

The source event identifies a notification about an export; one export can
contain many entries. Its event UUID cannot substitute for each entry's
delivery UUID. Required event-envelope metadata must be available; invalid
or missing required identity is a visible processing failure.

Example payload:

```json
{
  "retailer_code": "AnkerAu",
  "title": "USB charger",
  "price": {"amount": 1999, "currency": "AUD"},
  "url": "https://shop.example/Charger?variant=123#details"
}
```

Preserve retailer enum serialization, title text, exact URL, integer minor-unit
amount, and currency. Do not add an export wrapper or send an array of imports.
The existing publisher translates destination `offers` to HTTP inbox command
key `ImportOfferDetails`. It preserves message ID, payload, correlation, and
source, defaulting an absent source to `retailer_sourcing`. Source event ID,
type, and causation remain local outbox metadata.

## Required processing and failure behavior

- The event and completed snapshot agree on export UUID, source, and supported
  schema version. A delayed event still refers to its original immutable
  snapshot after newer source data exists.
- Every eligible entry has exactly one logical outbox message after successful
  processing. Exclusions create no messages.
- Duplicate and concurrent notifications, including fresh envelope UUIDs
  referring to the same export, do not create additional delivery identities.
- Replay preserves existing payloads, original metadata, statuses, attempt
  counters, reservations, and completed delivery. Envelope differences on a
  repeated notification do not replace the first recorded metadata.
- A failure followed by retry produces the complete set without duplicates,
  including when some entries were already recorded or delivered.
- Reusing an entry UUID for conflicting delivery content fails visibly.
- Missing, incomplete, inaccessible, malformed, or inconsistent snapshots
  fail visibly and follow existing event-processing retries. A failed attempt
  must not report successful completion of the whole export.
- Recording works while offers is offline or `OFFERS_URL` is absent. Reading
  object storage is part of consuming the existing immutable export; sending
  requests to offers is the separate publisher's responsibility.
- No new entry IDs, automatic downstream queue runs, or background workers
  are introduced by this task.

## Acceptance scenarios

| Scenario | Required evidence |
| --- | --- |
| Zero, one, and multiple entries | Exactly the eligible message set, including zero-message success |
| Mixed entries and exclusions | No message for excluded variations |
| Duplicate/concurrent notification | One delivery identity per entry |
| New envelope UUID, same export | Existing messages and first metadata remain unchanged |
| Partial recording failure then retry | Complete final message set without duplicates |
| Replay after publication | Completed rows are not reset or resent by recording |
| Conflicting entry identity | Visible error; existing content preserved |
| Newer source exists | Delayed event uses its own completed snapshot |
| Missing/pending export, object-storage failure | Visible retryable processing failure |
| Malformed object, unsupported schema, source mismatch | No false success or incorrect import |
| Unrelated event | No export outbox effects |
| Publisher compatibility | Recorded payload deserializes as the actual offers import command |

Database tests must establish persistence, concurrency, and recovery behavior.
Verify the existing publisher can consume representative recorded messages.
A fake or existing inbox can establish delivery compatibility; customer-facing
is not required. Explicitly distinguish recording, HTTP acceptance, and
subsequent offers processing in the evidence.

## Verification and operational handoff

Follow root and applicable service `AGENTS.md` requirements. Run
`make -C services fmt` before `make -C services ci`, and the relevant
`make -C services/retailer_sourcing test-postgres` checks with a separate
`DATABASE_URL_TEST` and isolated schemas. Run `make -C services audit` only
if Cargo dependencies or a lockfile change.

Start sourcing from its service directory with `PORT=3102 make serve` and
valid database/object-storage configuration, then check `/api/health`.
Keep existing smoke tests passing and extend coverage for any added routes.
Report any unavailable integration prerequisites rather than claiming those
checks passed.

Document the manual queue progression and how to observe recorded messages,
along with retry exhaustion and stale-reservation recovery. Do not reset
operational databases. No automatic historical backfill is required; events
already marked completed before this listener existed are not automatically
replayed. Existing accepted ADRs remain historical records; architecture
changes require new decisions under repository policy.
