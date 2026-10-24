# Offers inbox submission API plan

Status: implemented. See [the offers README](../services/offers/README.md#inbox-submission)
for the active contract and walkthrough.

## Summary

Add `POST /api/inbox` to persist one generic message through Mulac’s existing
inbox recorder. Messages remain queued until the existing inbox, command, and
event processing endpoints are called.

## API contract

- Request: required `payload` JSON value and nonblank `routing_key`; optional
  UUID `message_id`, UUID `correlation_id`, and string `source`.
- Serialize `payload` to JSON text for Mulac. Preserve the supplied metadata.
- Generate a message ID when omitted.
- Return `202 Accepted` with `{"message_id":"<uuid>"}` after successful storage.
- Repeated message IDs return success and preserve the original message, even
  when the new content differs.
- Return `400` for malformed envelopes or invalid UUIDs; `500` for storage or
  blocking-task failures, without exposing database diagnostics.
- Accept unknown routing keys and arbitrary JSON payloads; command-specific
  validation happens during processing.

## Implementation

- Construct an `InboxRecorder` using the existing PostgreSQL pool,
  `InboxRecorderRepository`, and `InboxStoreStorage`; expose it through
  application state to the orchestration API.
- Convert requests through Mulac’s `InboundMessageEnvelope` and
  `NewInboxMessageEnvelope`. Perform synchronous persistence with
  `spawn_blocking`.
- Use named error helpers, following the offers-specific repository
  instructions.
- Include the operation and request/response schemas in OpenAPI.
- Add a Bruno submission request and document an example followed by
  inbox → commands → events processing.
- Reuse the pinned Mulac dependencies and existing tables; no migration is needed.
  Offers declares `mulac-inbox` directly to access `InboxStoreStorage`, which the
  kernel does not re-export. This is the same version already in the lockfile.

## Validation

- HTTP tests: request validation, OpenAPI registration, and sanitized storage
  failures.
- PostgreSQL tests: payload and metadata persistence, generated IDs,
  duplicate-ID behavior, and submission leaving command/event queues untouched.
- Exercise a valid offer command through HTTP submission and all three
  processing stages, verifying its projection.
- Run `make fmt`, `make ci`, and `make test-postgres` in offers. Start with
  `PORT=<port> make serve` and verify health and submission.
- Run dependency audit only if dependencies or lockfiles change.

## Defaults and boundaries

- Offers only; single-message REST submission.
- Match existing unauthenticated operational endpoints.
- No batch submission, GraphQL mutation, message lookup, scheduling controls,
  broker consumption, or automatic processing.
