# Inbox and outbox status statistics

Status: implemented; PostgreSQL and live verification are blocked by the environment.

## Scope and grounding

Extend the existing development dashboard's status reports for
`retailer_sourcing`, `offers`, and `customer_facing` with inbox and outbox
counts, matching their command and event statistics. Object Storage has no
Mulac status panel and remains unchanged.

Grounded against monorepo `1fd8f40bbfdbefdd0a6cc234c6eae3fb444fee91`.
All three services already have `inbox_entries` and `outbox_entries` tables
with integer statuses and non-null `scheduled_at` timestamps. Their reporting
stores currently return only command/event arrays. The development client and
shared `render_entry_boxes` renderer already support the requested display
pattern. Local Mulac's inbox/outbox status definitions use the same codes as
commands/events; recheck these contracts against the checkout at implementation.

Build on [plan 33](33-today-entry-status-counts.md)'s UTC-day behavior already
present in the stores. This plan replaces the separate-page approach proposed
for inbox/outbox in [plan 32](32-daily-queue-statistics-dashboard.md): extend
`/dashboard` and `/api/reporting/entry-statuses`, without a new endpoint or page.
It does not require completing plan 32 or changing queue execution.

## Report contract

Add `inbox_entries` and `outbox_entries` arrays alongside `command_entries`,
`event_entries`, and `generated_at` in all three service responses. Each array
contains observed status groups as `{status, status_text, count}`, sorted by
numeric status. Counts remain exact unsigned integers, converted with checks
from PostgreSQL counts. Empty queues return empty arrays.

Use each row's current `scheduled_at` in the half-open UTC interval from today's
midnight through, but excluding, tomorrow's midnight. Capture one reference
instant per report after acquiring the connection and use it for all four
queues and `generated_at`. Include later today and every current status;
exclude overdue rows from yesterday and tomorrow's rows. Received, processed,
and creation timestamps do not select membership. Retries can move rows into
or out of the window; these snapshots are not throughput history.

| Code | API label | Dashboard label | Background |
| --- | --- | --- | --- |
| 0 | received | New | Light green `#BBEFC9` |
| 2 | reserved | Processing | Medium green `#66BB6A` |
| 4 | failed | Failed | Orange `#FFAD4D` |
| 5 | completed | Completed | Green `#237A43` |
| 7 | dead | Dead | Black `#000000` |
| 8 | archive | Archive | Grey `#D1D5DB` |

Retain unknown codes with API label `unknown` and neutral additional boxes.
Preserve existing authentication, missing-database responses, sanitized errors,
manual refresh, active database schema and independent service failures.

## Implementation steps

1. Extend each service's reporting DTO/OpenAPI schema and the existing SQL
   `UNION ALL` with inbox and outbox branches. Bind the same UTC boundaries to
   all four branches in one statement for a consistent PostgreSQL snapshot.
   Dispatch results explicitly by queue kind; the current command/else-event
   branch must not misclassify new groups. Preserve checked conversion and
   follow Offers' named error helper conventions.
2. Extend the development client's response structs and validation to cover
   both new arrays. Represent absent fields from older servers distinctly from
   empty arrays (for example optional vectors); show a report-upgrade message
   for missing queues rather than falsely reporting zero. Malformed present
   fields retain existing invalid-response handling. Deploy service updates
   first, then the app; older apps ignore the additive response fields.
3. Render Inbox, Command, Event, and Outbox entries in pipeline order within
   each service panel. Reuse the shared boxes, labels, colors, contrast and
   responsive layout. Every supported queue displays all six known statuses,
   with zero for absent groups, and retains the empty-day message where useful.
   Change panel headings/accessibility labels to cover all four queues. Retain
   report date/time and distinguish unavailable reports from genuine zeros.
4. Update the three service READMEs, development README, and root dashboard
   description. Update reporting examples/fixtures and OpenAPI assertions.
   No queue runner, payload, migration, dependency or architecture change is
   expected. Inspect representative query plans before proposing new indexes;
   preserve consumer indexes and require measured justification for changes.

## Acceptance and verification

- Extend isolated PostgreSQL reporting tests for each service to populate all
  four queues with distinct counts and mixed known/unknown statuses. Ensure no
  result is assigned to the wrong array, groups are ordered, and empty inboxes
  or outboxes do not suppress other queues.
- Extend deterministic daily-window fixtures, including temporary tables, to
  include inbox/outbox. Cover just before midnight, midnight, later today,
  just before tomorrow and exactly tomorrow; non-UTC database sessions,
  year rollover, differing received/processed dates, and status/reschedule
  changes. Run against migrated isolated schemas as well as focused fixtures.
- Retain authenticated-route, missing-database and OpenAPI smoke coverage for
  every service. Verify additive fields without changing existing field types.
- Update private-client and dashboard fixtures for all three services. Cover
  populated and empty queues, older responses missing the new fields,
  malformed fields, unknown status escaping, large counts and partial service
  failures. Verify twelve queue groups, their own counts and zero-state boxes.
- Run `make fmt` before `make ci` in each changed crate, then each service's
  `make test-postgres` with `DATABASE_URL_TEST` pointing to isolated test data.
  Run `make audit` only if Cargo dependencies or lockfiles change.
- Run the three services and development app with distinct `PORT` values and
  `make serve`. Disable sourcing command workers for stable fixtures. Check
  service health, authenticated reports and `/dashboard`; inspect desktop and
  mobile layouts in both themes, and verify one unavailable service does not
  hide other reports. Record environmental blockers instead of claiming checks
  passed. Do not reset operational databases or run queues to obtain fixtures.

Completion requires the additive reports and all twelve dashboard queue groups,
updated documentation and recorded validation results. Move the TODO to `[~]`
when implementation begins and `[x]` when completed. This task stops at `[>]`.

## Implementation and verification record

Implemented additive inbox/outbox fields in all three reporting services,
using the existing UTC bounds and a single four-queue query. The dashboard now
renders twelve queue groups, preserves known zero states and unknown codes,
and identifies older service responses without inventing zero counts. Present
null/non-array queue fields are rejected. Documentation and API schemas were
updated without dependency or migration changes.

Verification on 2026-10-29:

- Formatting and lint pass in all four changed crates.
- Customer Facing `make ci` passes: 41 tests passed, 38 database tests ignored,
  including the expanded report JSON and OpenAPI checks.
- Development's rendering and new compatibility tests pass (15 main-binary
  tests passed); seven existing HTTP-client tests fail because the sandbox
  denies opening local sockets.
- Retailer Sourcing reporting smoke tests pass (two passed, five PostgreSQL
  tests ignored). Full CI stops at an existing export tool test requiring a
  local listener. Offers CI reaches 86 passed, four existing publisher tests
  fail on local socket permissions, and 36 database tests are ignored.
- Daily PostgreSQL fixtures now include all four queues and independently
  verify inbox rescheduling and outbox status changes. The migrated sourcing
  fixture verifies distinct inbox/outbox groups. `make test-postgres` cannot
  run without `DATABASE_URL_TEST`. Follow-up: all three daily-window test
  modules are now named `postgres_tests`, so `make test-postgres` includes
  their midnight, DST, and rescheduling checks without a separate command.
- `PORT=<port> make serve` was attempted for every changed server. Offers and
  Customer Facing require missing database configuration; Retailer Sourcing
  rejects its configured object-storage URL; development cannot bind a socket.
  Live page/health and browser checks remain unverified.
- `git diff --check` passes. No dependency audit is needed.
