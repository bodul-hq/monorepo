# PR #23 selected review fixes — verification

Date: 2026-10-30.

Implemented comments 1, 2, 3, 4, 6, 9 and 11. Item 33's existing marker,
SVG serving/caching, reporting ownership and published history are unchanged.
No dependency declarations or lockfiles changed, so no audit was run.

## Portable checks

Ran `make fmt`, followed by `make ci`, in all four changed crates.
Formatting and clippy passed for each.

| Crate | Test outcome |
| --- | --- |
| Customer Facing | CI passed: 41 passed, 39 PostgreSQL tests ignored |
| Offers | 86 passed, 37 ignored; four existing publisher HTTP tests failed while binding sockets with `Operation not permitted` |
| Retailer Sourcing | CI stopped in `export_offer_sources`: six passed and one existing HTTP fixture test failed while binding a socket |
| Development | Library and preceding binary tests passed; main binary: 21 passed, seven HTTP client tests failed while binding sockets |

Development's passing tests include all 14 dashboard rendering tests, the
explicitly supplied icon after a display-label change, scoped compact queues,
accessible zero counts, unknown-only populated queues, missing arrays,
unavailable reports, and existing escaping coverage.

A separate sourcing `cargo test --bin retailer_sourcing reporting` passed
two portable route tests and ignored six PostgreSQL tests. All test targets,
including the new migration tests, compiled under clippy.

## PostgreSQL checks and measurements

Ran each service's `make test-postgres`; all stopped because
`DATABASE_URL_TEST` is unset. A second attempt with a local test URL ran each
service's fixed-window and migration measurement tests; each failed at connection
establishment (`BadConnection`), before executing fixture or migration SQL.
A direct psql attempt reports `Operation not permitted` opening the local
PostgreSQL socket. PostgreSQL client binaries exist, but `postgres` and
`initdb` server binaries are absent, so a private single-user fallback was
not available.

Consequently no baseline or indexed execution times, buffers, index sizes or
server version could be measured. Migration up/down/reapply and populated route
execution remain unverified here; this record does not claim they passed.

The existing fixed-window tests cover inclusive/exclusive UTC boundaries,
December 31 rollover, non-UTC database sessions, unknown statuses, status changes
and rescheduling. Counting fixtures and populated reporting routes now use
explicit December 31 schedules and instance clocks, with exact window assertions.

Each service adds
`reporting::store::postgres_tests::postgres_scheduled_indexes_round_trip_and_explain`,
selected by the unchanged `make test-postgres` target. It:

- Creates a unique schema inside a rollback-only transaction and uses the
  service's embedded transactional migrations.
- Inserts 200,000 historical, 200 current-day and 200 next-day records in each
  queue, across six known codes and an unknown code, with 256-byte payloads.
- Runs ANALYZE on fixtures only and checks exactly 200 reported entries per queue.
- Prints PostgreSQL version and baseline/post-index
  `EXPLAIN (ANALYZE, BUFFERS)` output for the four-queue reporting query.
- Prints each new index's byte size, compares reports across up/down/reapply,
  and checks that the complete pre-existing index definitions survive.

Run in each service with a configured test database:

```sh
cargo test --bin <service_name> postgres_scheduled_indexes_round_trip_and_explain -- --ignored --nocapture
make test-postgres
```

Customer Facing still requires its documented trusted `pg_trgm` setup.
Capture the output and record execution time, shared hit/read buffers, index
sizes and server version before drawing performance conclusions. These are
synthetic warm-cache measurements; no planner settings are forced.

The twelve additive B-tree indexes use the existing migration mechanism.
Creation blocks writes; each service README calls for a maintenance window.

## Live and visual checks

Started via `make serve` with ports 3191 (Offers), 3192 (Customer Facing),
3193 (Sourcing, `COMMAND_WORKERS=0`) and 3195 (Development).

Offers and Customer Facing stop because `DATABASE_URL` is required.
Sourcing's initial start failed on an empty object-storage URL; retrying with a
local absolute URL reached pool connection retries and was stopped by the
15-second timeout. Development fails to bind with `Operation not permitted`.
Health/dashboard and authenticated reporting curl probes could not connect.

Headless Firefox exited with status 1 without a screenshot. Desktop/mobile
checks in both themes, scrolling checks, and live authenticated report checks
therefore remain pending. Static CSS inspection confirms the two-class empty
selectors outrank the desktop and mobile one-class rules, explicitly reset
`min-height`, and preserve clipping, positioning, dimensions and hiding margins.

## Commit delivery

The workspace's `.git` is read-only: `git add` and `git commit` fail creating
`.git/index.lock`. One commit per issue is instead preserved in a separate
Git directory under `/tmp/pr23-review-commits`, with a bundle for import.
Source changes remain in the original workspace; its branch is not moved.
No push, review-thread changes, deployment or history rewrite was performed.
