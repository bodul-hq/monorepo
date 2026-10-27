# Daily sourcing queue statistics and dashboard

Status: planned. Implementation has not started.

Planned against monorepo `f0c4e50` and sibling Mulac `fd29f72` on 2026-10-27.
This is one TODO with a sourcing API and a development-app page. The user
confirmed that “today” means **scheduled for today, in UTC**, rather than
created or processed today.

## Contract and scope

Count rows whose current `scheduled_at` falls in the half-open interval
`[today at 00:00:00 UTC, tomorrow at 00:00:00 UTC)`, grouped by current integer
status, separately for `inbox_entries`, `command_entries`, `event_entries`
and `outbox_entries`.

Include all statuses, including completed, dead, archived and unknown codes,
and entries scheduled later today. Exclude overdue entries scheduled before
today. Count each stored entry once, not its attempts or historical status
transitions. Do not filter on `received_at`, `updated_at` or `processed_at`.

This is a live schedule/status snapshot. Retries can change `scheduled_at`, so
an entry can move into or out of today's report. Status changes move entries
between groups. The report is not historical daily throughput and its totals
need not increase monotonically. No queue actions, historical storage, arbitrary
date/timezone inputs or other-service statistics are included.

## Sourcing API

Add authenticated `GET /api/development/queue-statistics/today`, registered
through the development feature's existing bearer middleware and OpenAPI
assembly. Keep `/api/reporting/entry-statuses` unchanged: it remains the all-time
command/event report. No additional GraphQL query is required.

Return `200` with this additive JSON contract:

```json
{
  "generated_at": "2026-10-27T13:30:00Z",
  "date": "2026-10-27",
  "timezone": "UTC",
  "timestamp_field": "scheduled_at",
  "window_start": "2026-10-27T00:00:00Z",
  "window_end": "2026-10-28T00:00:00Z",
  "inbox_entries": [{"status": 0, "status_text": "received", "count": 12}],
  "command_entries": [{"status": 2, "status_text": "reserved", "count": 4}],
  "event_entries": [],
  "outbox_entries": [{"status": 5, "status_text": "completed", "count": 8}]
}
```

Always include all four arrays; an empty queue/window is `[]`. Emit only
observed status groups and sort each array numerically by status. Counts are
exact nonnegative `u64` values converted from PostgreSQL `COUNT(*)` with checked
conversion. Retain the raw status code alongside its label:

| Code | Text |
| --- | --- |
| 0 | `received` |
| 2 | `reserved` |
| 4 | `failed` |
| 5 | `completed` |
| 7 | `dead` |
| 8 | `archive` |
| Any other integer | `unknown` |

These codes are verified in all four Mulac assembly domain enums. Unknown
codes remain visible rather than failing or being silently dropped.

Follow existing reporting failure behavior: missing/wrong bearer yields `401`
before database work; absent DB or pool/connectivity failure yields `503`;
unexpected query/task failure yields generic `500`. Never expose SQL, connection
strings, payloads or credentials. Add `Cache-Control: no-store` to report
responses, including errors returned by the new handler.

## Query and implementation boundaries

Add a focused `development::queue_statistics` slice with HTTP/DTO and store/query
code, following existing development slices. Follow reporting's
`spawn_blocking`, bounded pool acquisition and generic error conventions.
Reuse status labeling through a narrow internal interface if useful; do not
expose another feature's private modules or reorganize unrelated reporting.

Use one SQL statement containing four grouped queries joined by `UNION ALL`,
or equivalent aggregate subqueries. All queues then share one PostgreSQL MVCC
snapshot while workers run. Query the connection's active schema with static
table identifiers. Do not read payloads or acquire row locks.

Capture one reference instant after pool acquisition; derive the UTC date and
both bounds once. Prefer a SQL clock/bounds CTE using `statement_timestamp()`,
returning metadata even when all groups are empty. Add an internal deterministic
clock/bounds seam for tests, not a request parameter. Compute UTC midnight as a
timestamp without timezone, then convert that midnight and the next calendar
midnight separately `AT TIME ZONE 'UTC'`. Avoid session-dependent
`CURRENT_DATE`, `scheduled_at::date`, or `timestamptz + interval '1 day'`.
The same reference instant determines `generated_at`, `date` and the window.
Equivalent existing Chrono UTC types and bound UTC timestamps are acceptable.

Each predicate is `scheduled_at >= start AND scheduled_at < end` against the raw
column. Existing queue indexes lead with `(status, scheduled_at)`; they do not
guarantee efficient all-status date-range scans. Inspect
`EXPLAIN (ANALYZE, BUFFERS)` on representative test data. Start without a
migration; if evidence shows unacceptable scanning, add an additive service
migration for `(scheduled_at, status)` indexes on the four queues, with down SQL
and migration tests. Preserve consumer indexes and accepted old migrations.
No Cargo dependency or Mulac change is required for this UTC implementation.

## Development dashboard

Add a separate server-rendered `/dashboard/queues` page titled “Today's sourcing
queues”, with a home-page tool card and top-bar navigation entry. Retain the
existing `/dashboard` behavior and all-time status reports. Use a focused
dashboard submodule, existing `Dashboard` state, shared layout, responsive
styles and private client.

Add a client method using existing `RETAILER_SOURCING_URL` and
`RETAILER_SOURCING_TOKEN`. Preserve base-path prefixes, sensitive authorization
headers, five-second timeout and disabled redirects/retries. The new page fetches
only this new sourcing report. Credentials remain server-side. Refresh reloads
`/dashboard/queues`; no browser API calls or automatic polling.

Show reported UTC date, generation time, window, and a plain-language explanation:
entries currently scheduled for this UTC day, grouped by current status; retries
may move entries between days. Render four queue cards, each with a total and a
table of status label, numeric code and exact count. Use wide/checked integer
totals without floating-point rounding. Empty arrays show zero and “No entries
scheduled for this UTC day”.

Use reported dates rather than independently deciding today in the app. Validate
the fixed UTC/scheduled_at contract, timestamps/window and unique status groups.
Preserve unknown codes. Escape upstream text, use accessible table headings and
keyboard-usable scrolling at narrow widths. Reuse both themes and the existing
optional theme toggle; the data page works without JavaScript.

Match the existing dashboard's `200` HTML error-state convention for unavailable,
authentication, malformed-response and configuration failures. Distinguish these
cases, offer refresh, and never show failed requests as zero counts or expose
diagnostics/tokens. Return `Cache-Control: no-store` on successful/error pages.
An older sourcing deployment returning `404` appears unavailable until upgraded.

## Validation and acceptance

1. Sourcing portable route tests: bearer enforcement, absent DB, no-store, exact
   registration, OpenAPI security and response schemas; known labels and unknown
   positive/negative codes.
2. Isolated PostgreSQL fixtures: all four tables, mixed statuses, exact grouping,
   ordering, empty results, unchanged legacy report and active-schema isolation.
   Seed immediately before start, exactly at start, later today, immediately
   before end and exactly at end. Check month/year rollover and a non-UTC session
   timezone on a DST-transition date to prove boundaries remain the UTC day.
3. Vary received/processed dates independently of scheduled dates. Change status
   and reschedule across the boundary, verifying updated counts without counting
   attempts. Cover generic query failures and the single-statement snapshot.
4. App client fixtures: exact path/auth/base-prefix handling, timeout/connection
   failures, redirects rejected, auth errors, invalid JSON/contract, four arrays,
   unknown statuses and large counts.
5. Page tests: route/home/nav links, reported date/window, four cards and totals,
   empty/error states, escaped hostile text, no secrets/browser API calls, and
   unchanged old dashboard.
6. Run `make fmt` then `make ci` in both changed crates, plus sourcing
   `make test-postgres`. Run audit only if dependencies actually change. Start
   sourcing and development on distinct `PORT` values; check sourcing health,
   authenticated statistics and the new page against controlled data. Use
   `COMMAND_WORKERS=0` for deterministic fixtures. Check desktop/mobile and both
   themes, refresh and an unavailable sourcing service. Record runtime blockers
   honestly rather than claiming skipped checks passed.

## Documentation and delivery

Update both crate READMEs and the root app/endpoint overview with routes, UTC
scheduling semantics, auth, examples and manual refresh behavior. Add a Bruno
request under sourcing's existing development collection, reusing its environment
and bearer configuration. This is additive reporting within
[ADR 13](../adr/0013-development-database-reporting.md); no new architecture ADR
is needed unless implementation introduces a new architectural choice.

Implement API/query/tests first, app client/page/tests second, then docs and live
acceptance. Deploy sourcing before the app. No queue rewrite, backfill, new
background job or worker change is required. If performance evidence requires
indexes, coordinate the additive migration with service deployment. Mark the
TODO `[~]` when implementation starts and `[x]` when complete, linking actual
verification results.

