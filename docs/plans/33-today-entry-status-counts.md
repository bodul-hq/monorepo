# Today's command and event status counts

Status: planned; implementation has not started.

## Scope and current behavior

The user confirmed all three dashboard services: `retailer_sourcing`, `offers`,
and `customer_facing`. Today means the current UTC calendar day, using each
entry's current `scheduled_at`.

Each service's `src/reporting/store.rs::load_entry_statuses` currently groups
every command and event by status without a date predicate. The development
app consumes these reports through `/api/reporting/entry-statuses` and renders
them in `src/dashboard.rs::render_entries`.

Change these existing reports and the existing dashboard. This plan supersedes
plan 32's instruction to preserve all-time command/event counts. Its proposed
separate four-queue sourcing page is not a prerequisite or part of this fix.
Inbox/outbox reports, table row estimates and queue execution are out of scope.

## Implementation

1. In each service, capture one UTC reference instant per report after acquiring
   the connection. Derive midnight and the next UTC midnight once. Provide an
   internal helper accepting the reference instant for deterministic tests;
   expose no request clock or date override.
2. Bind the resulting timestamp bounds into both branches of the existing
   single `UNION ALL` query. Apply `scheduled_at >= start AND scheduled_at < end`
   before grouping. Preserve one PostgreSQL snapshot for both queues, active
   schema behavior, integer counts, ordering, status labels and unknown codes.
   Set `generated_at` from the same reference instant. Avoid session-local
   `CURRENT_DATE` and casting the indexed column to a date.
3. Keep the existing response fields and routes. Update API descriptions to
   explain scheduled-for-today UTC semantics. Preserve bearer authentication,
   missing-database/error handling and checked count conversion. Follow the
   offers crate's named error helper conventions if new errors are needed.
4. Update the shared dashboard renderer to say “Scheduled for today (UTC), by
   current status”; derive the displayed report date from its `generated_at`,
   so independent service fetches around midnight remain understandable.
   Change empty copy to “No entries scheduled for this UTC day”. Keep unavailable
   reports visibly distinct from empty reports. Retain manual refresh and
   server-side credentials. Deploy all three service fixes before the app copy.
5. Update the three service READMEs and development README with the changed
   report semantics. No dependency, schema, worker or architectural change is
   expected. Inspect representative query plans before proposing indexes;
   preserve existing consumer indexes and require measured justification for
   any follow-up migration.

Include entries later today and all statuses, even completed or unknown ones.
Exclude overdue entries from earlier days and entries scheduled tomorrow.
Creation, processing and update timestamps do not determine membership. Retries
that reschedule entries can change the day's totals; these are current snapshots,
not historical throughput. Different services need not share a database snapshot.

## Acceptance and verification

- For each service, extend isolated PostgreSQL reporting tests with entries just
  before UTC midnight, exactly at midnight, later today, just before tomorrow,
  and exactly at tomorrow. Verify both queues, mixed/unknown statuses, sorting,
  exact counts and empty results using an injected reference time.
- Make received/processed dates differ from scheduled dates. Check status
  changes and rescheduling into and out of the window. Exercise month/year
  rollover and a non-UTC database session on a DST transition date.
- Retain route authentication and missing-database tests and verify the existing
  JSON contract. Update dashboard fixtures to cover all three service sections,
  the reported UTC date, empty copy and unchanged error handling.
- During implementation, run `make fmt` then `make ci` in all four changed
  crates, plus each service's PostgreSQL test target. Audit only if dependencies
  change. Start each service and the app on distinct `PORT` values with
  `make serve`; verify health, authenticated reporting and dashboard output
  against fixtures, disabling sourcing command workers for stable counts.
  Record actual results and environmental blockers; do not mark skipped tests
  as passing.

Mark the TODO `[~]` when implementation starts and `[x]` only when the fix and
its verification are complete. This planning task stops at `[>]`.
