# Item 10: Customer-facing reports in the development dashboard

Status: completed and verified on 2026-10-25.

## Goal and agreed scope

Add customer-facing as the fourth service on `/dashboard`, matching the existing
reports: database table sizes and estimated rows, plus exact command and event
counts by status. Save the plan alongside the existing nine plans in
`docs/plans`, as agreed. Planning uses `[.]` in TODO; mark `[x]` only after
implementation and verification.

Grounded at monorepo `1cc1b77` and local Mulac `9474319`. Follow ADR 13's existing
reporting architecture. No new architecture decision, dependency, migration,
queue execution, deployment, or operational database changes are included.
Inbox/outbox status breakdowns are outside this item.

## Customer-facing reporting API

Add a service-owned reporting module, using the existing database pool and an
injectable repository. Blocking database work runs through `spawn_blocking`.
Document bearer authentication and both routes in OpenAPI:

- `GET /api/reporting/database-tables`: `schema_name`, UTC `generated_at`,
  `row_count_kind: "estimated"`, and `tables`. Each table contains `table_name`,
  `row_count`, `table_bytes`, `index_bytes`, and `total_bytes`.
- `GET /api/reporting/entry-statuses`: UTC `generated_at`, `command_entries`,
  and `event_entries`. Each status contains numeric `status`, `status_text`,
  and exact `count`.

Use PostgreSQL statistics and size functions for ordinary tables in the active
schema, including offer, queue and migration tables. Exclude system schemas
and views; sort tables alphabetically. Do not scan tables for row totals or run
ANALYZE. Sum table and index bytes for total storage.

Count both command and event queues in one statement, sorted by status. Labels
are `0 received`, `2 reserved`, `4 failed`, `5 completed`, `7 dead`, `8 archive`;
other numeric codes remain visible as `unknown`. Empty queues return arrays
with no entries.

Use `CUSTOMER_FACING_REPORTING_TOKEN`, defaulting to
`customer-facing-development-token`. An explicit empty token authorizes no
requests. Preserve shell and Make overrides when loading `.env`. Reject
missing/invalid credentials with 401, database unavailability with 503, and
query/task failures with sanitized 500 responses. Report responses, including
authentication failures, use `Cache-Control: no-store`.

## Dashboard and documentation

Extend existing private HTTP clients and rendering. Add `CUSTOMER_FACING_URL`
(default `http://172.28.100.42:3000`) and `CUSTOMER_FACING_TOKEN` (default
`customer-facing-development-token`). Reuse prefix handling, bearer headers,
five-second timeout, disabled redirects/retries, validation, and error categories.
Fetch all reports concurrently; keep credentials and diagnostics off the page.

Append the Customer Facing database card and command/event status section.
Count successful database reports out of four; combined table/storage totals
include successful reports only. Every report fails independently. Keep
manual refresh, HTML escaping, accessible tables and the responsive light/dark
design. Do not add browser API requests or background polling.

Update service/development READMEs, the root dashboard description, the service
environment example, and two authenticated Bruno requests. Preserve unrelated
working-tree changes. Rebuild/restart customer-facing before development when
rolling out; recreate containers to apply changed environment values.

## Acceptance and verification

- API tests: authentication before database access, empty reports, OpenAPI
  security/schemas, no-store responses, and sanitized database/task failures.
- Isolated PostgreSQL tests: active-schema isolation; offer/internal tables;
  view exclusion; sorted output; storage arithmetic; empty queues; exact status
  counts and unknown codes; unchanged queue rows after reporting.
- Dashboard tests: upstream URL/token/prefix handling, authentication/timeouts/
  malformed responses, four-service totals, independent failures, empty queues,
  escaped content, and no credential leakage.
- Run `make fmt` then `make ci` in customer-facing and development. Run
  customer-facing `make test-postgres`. Audit only if Cargo dependencies change.
- Run customer-facing with `PORT=3103 make serve` against an isolated test schema
  and development with `PORT=3105 make serve`, overriding its upstream URL.
  Verify health, authenticated reports, and `/dashboard`. Check desktop/mobile
  layouts in both themes. Stop the test upstream and verify other reports remain.
- Record actual results before marking this plan, the plan index and TODO complete.

## Completion evidence

- Added customer-facing's authenticated reporting API with an injectable repository,
  existing-pool PostgreSQL reads, no-store responses, and sanitized failures.
  The dashboard shows four services and independent customer-facing table/status
  results. Added OpenAPI, environment, README and Bruno documentation.
- `make fmt` and `make ci` passed in both changed crates: customer-facing ran
  30 database-independent tests; development ran 35 tests across its targets.
  All 16 customer-facing PostgreSQL tests passed, including three new reporting
  cases, each using its own generated schema. Cargo manifests and lockfiles did
  not change, so no audit was required.
- Started both binaries using `PORT=3103/3105 make serve`, with customer-facing
  connected to an isolated schema in `customer_facing_test`. Verified health,
  authenticated reports, exact queue counts and the four-service dashboard.
  Other services used local HTTP report fixtures; no operational queues ran.
  After stopping customer-facing, the dashboard retained the three other reports
  and changed its availability count to 3/4. All verification processes were
  stopped and only the generated test schema was removed.
- Firefox previews passed in light and dark themes at desktop (1440 CSS pixels)
  and narrow (500 CSS pixels) widths, with no page-level horizontal overflow.
  The requested 390-pixel window was clamped to 500 by Firefox; the recorded
  width reflects the measured viewport. Theme buttons, Customer Facing sections,
  and Refresh were present. Desktop light and narrow dark screenshots were
  visually reviewed; tables scroll within their cards.
- Local evidence: `/tmp/bodul-customer-reporting-ci.log`,
  `/tmp/bodul-dashboard-reporting-ci.log`,
  `/tmp/bodul-customer-reporting-postgres.log`, `/tmp/bodul-dashboard-live.log`,
  and `/tmp/bodul-dashboard-{desktop,mobile}-{light,dark}.png`.
- Deployment is not part of this change. Rebuild/restart customer-facing and
  development to expose the new reports in the running environment.
