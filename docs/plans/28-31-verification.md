# Items 28–31 implementation and verification

Date: 2026-10-27. Implemented from monorepo `9fb43bc` on `wip/upgradez`, with
sibling Mulac `fd29f72`. No Mulac changes or commits were made in this work.
TODO markers mean implementation is complete; they do not claim blocked
database or live acceptance checks passed.

## Implemented behavior

- **28:** Shared localized content with exact language keys, merge-by-language
  imports, selected scalar compatibility fields, `OfferCreated.v9` and
  `OfferContentsChanged.v1`. Historical v1–v8 creation and legacy scalar changes
  remain readable. Offers and customer-facing backfill/store nonempty JSONB
  lists, atomically update selection fields and lock legacy read/modify/write.
  Customer-facing REST/GraphQL return arrays; customers accepts old and new
  shapes. Source exports/import observations remain unchanged.
- **29:** `COMMAND_WORKERS` defaults to four, zero disables background commands,
  invalid values fail startup. The executable starts command workers only after
  fallible setup, cancels them on signal/server failure and joins all handles
  after HTTP drains. Events/outbox remain manual. Reserved batches finish.
- **30:** Every offer download waits the six-second constant before fetching,
  including retries. Tests inject sleep and verify order and failure paths.
- **31:** Sitemap crawls pause a random inclusive whole 1–5 seconds between
  actual fetch attempts. Skip paths and the first/final boundaries do not add
  pauses. This replaces the previous batch pause policy.

The separate plans and [ADR 38](../adr/0038-localized-offer-content.md) /
[ADR 39](../adr/0039-sourcing-command-workers.md) record scope and rollout.
Cross-reviews covered worker lifecycle/pacing and downstream reconciliation.
One missing success/storage-failure pacing assertion was added during review.

## Executed checks

`make fmt` ran before CI in affected folders and from the root. Formatting and
strict all-target Clippy checks pass in each affected Rust crate.

| Scope | Command/check | Actual result |
| --- | --- | --- |
| Root | `make fmt` then `make ci` | Format passes. CI stops at unchanged `lib/inbox_client`: 1 passed, 7 failed at local socket creation with `EPERM`. |
| `lib/retailer_sauce` | `make fmt`, `make ci` | Pass; 17 tests. |
| `services/offers` | `make fmt`, `make ci` | Format/Clippy pass; 86 tests pass, 35 PostgreSQL tests ignored, 4 existing HTTP publisher tests fail at socket bind with `EPERM`. |
| `services/offers` | Repeat tests excluding those exact 4 socket tests | Pass; 86 passed, 35 ignored, 4 filtered. Final PostgreSQL test edit also passes all-target Clippy. |
| `services/customer_facing` | `make fmt`, `make ci` | Pass; 41 tests, 37 PostgreSQL tests ignored. |
| `apps/customers` | `make fmt`, `make ci` | Format/Clippy pass; 28 tests pass, 6 existing HTTP-fixture tests fail at listener bind with `EPERM`. No filtered rerun was claimed. |
| `services/retailer_sourcing` | `make fmt`, `make ci`, `cargo test --no-fail-fast` | Format/Clippy pass. Full tests fail only in 8 existing socket fixtures across binaries/integration; main binary has 135 passed and 77 ignored before the final additional test. |
| `services/retailer_sourcing` | Repeat tests excluding those exact 8 socket tests | Pass; 147 total tests, 77 ignored, 8 filtered. |
| `services/retailer_sourcing` | Final ordering test, scoped fmt/lint and download-handler tests | Pass; 7 handler tests, including the newly added success/storage-failure case. This adds one distinct passing test to the earlier 147; the full suite was not unnecessarily repeated. |

New portable coverage includes content guards, exact text preservation,
omission/null semantics, selection-only changes, real legacy record adapters,
mixed replay, sequence-capacity atomicity, compatible publication, API schemas,
read mapping and unchanged card rendering. Sourcing tests cover configuration,
joining every handle even after a task failure, delay order and crawl skip/failure
paths without real sleeps. Makefile fixture checks covered `.env` values and
explicit shell/command-line overrides, including zero and empty values.

Isolated PostgreSQL cases compile for populated migration down/up, pending work,
full-list/legacy projections, rollback, receipts, browse/search/GraphQL parity,
row locking and worker concurrency/cancellation. They remain unexecuted because
`DATABASE_URL_TEST` is absent; local server executables are unavailable.
`make test-postgres` was attempted in offers, customer-facing and sourcing and
reported the missing test URL.

## Live attempts and remaining verification

- Offers: `PORT=3417 make serve` built and failed because `DATABASE_URL` is
  required. Its health request could not run successfully.
- Customer-facing: `PORT=3313 make serve` built and failed because `DATABASE_URL`
  is required; `curl --max-time 2 --fail http://127.0.0.1:3313/api/health` could
  not connect.
- Customers: `PORT=3314 make serve` built and failed binding the listener with
  `Operation not permitted`; the homepage curl could not connect.
- Sourcing: `PORT=3109 DATABASE_URL= COMMAND_WORKERS=0 make serve` built and
  failed binding the listener with `Operation not permitted`; health could not
  connect. This attempt does not verify background command execution.

No live health, database migration, worker progression or complete
sourcing-to-customers delivery is claimed. In a runtime with sockets and the
documented databases/extensions, rerun full CI and each service's
`make test-postgres`, then perform startup/health and controlled import/delivery
checks. Verify automatic sourcing command progression without calling
`/commands/run`; events and outbox still need their explicit runners.

Deploy the compatible customers reader, then customer-facing readers/migration,
then offers migration/writer. Pause projection writers during backfill. A down
migration retains selected scalar values but drops additional language entries;
recover them from backup/history if needed. Pacing is per handler/crawl, and
shutdown can wait for reserved batches to finish their sleeps and network calls.

## Dependency audit

Sourcing declares `fastrand = "2.5"`, already locked transitively. The only lock
change adds that dependency edge; no package version changed. `make audit`
was attempted and could not lock the read-only advisory cache. An offline audit
using a copied cache and the services audit configuration was compared with the
baseline services lockfile: both have the same 8 vulnerabilities and 5 warnings,
with **no new advisories**. The cached audit is not a freshly fetched database.

Identical advisory ID/package/version sets:

| Package | Version | Advisory IDs |
| --- | --- | --- |
| rustls | 0.23.45 | RUSTSEC-2026-0346, 0347, 0348, 0349 |
| rustls-webpki | 0.103.15 | RUSTSEC-2026-0382 |
| zip | 8.6.0 | RUSTSEC-2026-0367, 0368, 0369 |
| fxhash | 0.2.1 | RUSTSEC-2025-0057 |
| kuchiki | 0.8.1 | RUSTSEC-2023-0019 |
| unsafe-libyaml | 0.2.11 | RUSTSEC-2024-0449 |
| rand | 0.7.3 | RUSTSEC-2026-0097 |
| tokio | 1.53.2 | RUSTSEC-2026-0386 |

## Local evidence

Logs available in the implementation environment:

- `/tmp/todo28-31-root-fmt.log`, `/tmp/todo28-31-root-ci.log`.
- `/tmp/offers-ci.log`, `/tmp/offers-tests-final.log`,
  `/tmp/offers-lint-final.log`, `/tmp/offers-postgres.log`, `/tmp/offers-serve.log`.
- `/tmp/sourcing-ci.log`, `/tmp/sourcing-tests.log`,
  `/tmp/sourcing-portable-tests.log`, `/tmp/sourcing-order-tests.log`,
  `/tmp/sourcing-order-lint.log`, `/tmp/sourcing-serve.log`,
  `/tmp/sourcing-health.log`, `/tmp/sourcing-postgres.log`.
- `/tmp/sourcing-audit.log`, `/tmp/sourcing-audit-offline.log`,
  `/tmp/sourcing-audit-baseline-services.log`, `/tmp/sourcing-audit-comparison.log`.

Customer-facing and customers results above were captured in executed tool
output; no persistent logs were saved for those runs. Older similarly named
logs from prior tasks are not evidence for this implementation.
