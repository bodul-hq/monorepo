# Offer facts implementation and verification

Date: 2026-10-26. Branch: `wip/upgradez`. The implementation is included in
`a79c3eb` (`wip`), which appeared during the final verification.

The sections below retain the original results; the
[2026-10-27 follow-up](#completion-follow-up-2026-10-27) records the later work.

Current tracking status: TODOs 24–27 are marked `[x]` at the user's subsequent
explicit request. The database/storage/live verification limitations below
remain recorded; this status update adds no new test results.

The four separately planned items are implemented: source comparison prices,
availability/condition, description Content, and five identifier columns in the
identity projection. They were left `[.]` in TODO.md because database and live
acceptance checks could not finish in this environment. No agent-created commits
were made.

## Implementation

All 44 completed structuralizers emit the canonical fields. Source coverage and
unknown facts are documented in [the coverage report](../offer-facts-source-coverage.md).
Exports v6/v7/v8 progressively add pricing, statuses and content; old pinned
versions and prepared payload bytes remain supported. Missing observations and
explicit clearing remain distinct. Creation records read v1–v8 and write v8.
New pricing, availability, condition and content events reach both projections
and customer-facing delivery; legacy price events keep price-only behavior.

Additive migrations provide comparison-price pairs, status JSONB and description
TEXT. The identity migration copies all five exact identifiers from matching
read-offer rows, verifies the copy and rejects contradictory identity. Both
REST and GraphQL expose the facts; customers render only the comparison price,
with existing money formatting and accessible crossed-out text.

ADRs [34](../adr/0034-source-comparison-pricing.md),
[35](../adr/0035-source-availability-and-condition.md),
[36](../adr/0036-offer-content.md), and
[37](../adr/0037-identity-product-identifiers.md) record the contracts.

## Executed checks

- Root `make fmt` passed; `git diff --check` passed. No Cargo manifest or lockfile
  changed, so dependency audit was not required.
- Shared `retailer_sauce`: `make ci` passed, 14 unit tests.
- Anker: 577 tests passed. Minisforum: 468 tests passed. Ugreen's stale full-output
  expectations were updated for the added fields; the affected NAS pipeline
  tests passed after the update. Extractor portable tests passed (15). The
  deterministic source sample passed across 44/44 storefronts and 82 variations,
  with all status/content guards passing. The source report records nullable
  coverage and unrelated source-schema failures from the longer partial survey.
- Final services clippy for offers, retailer_sourcing and customer_facing passed
  with `--all-targets -- -D warnings`.
- Offers: 76 portable tests passed, including new aggregate/replay, malformed
  object, nullable update, HTTP import, event publication and identity cases.
  Four socket-dependent tests were explicitly filtered after the full run
  failed with `Operation not permitted`; 29 PostgreSQL tests remain ignored.
- Sourcing: 127 main-binary portable tests passed; two socket-dependent tests
  were explicitly filtered after the full run failed at socket creation.
  The export-focused run passed 18 tests with 8 database/live cases ignored.
  The full service `make ci` also encountered the same socket restriction in
  `export_offer_sources`. The main binary has 68 ignored database/live cases.
- Customer-facing: `make ci` and the final tests passed, 38 tests; 32 PostgreSQL
  tests remain ignored. Tests consume the actual v6–v8 export fixtures and
  preserve historical creation data while rejecting invalid new pricing.
- Customers: formatting and lint passed; 24 tests passed in the full run,
  including fact parsing and sale-card rendering. Six existing tests failed
  at socket creation, before exercising their HTTP fixtures. A final portable
  run passed all 24 tests with those six explicitly filtered.

Root `make ci` was attempted and stopped in unchanged `inbox_client` tests when
localhost socket creation was denied. These environment failures were not
converted to passing tests or disabled in source code.

## Pending runtime acceptance

`make test-postgres` could not run for offers, retailer_sourcing or
customer_facing because `DATABASE_URL_TEST` is not configured. Docker access is
denied and only PostgreSQL client tools are installed. The added migration,
projection, null-clearing, receipt rollback and queued-work tests compile but
have not executed against PostgreSQL.

Live `PORT=... make serve` attempts built the changed applications. Offers and
customer-facing stopped because `DATABASE_URL` is required. Retailer sourcing
and customers could not create listening sockets (`Operation not permitted`).
Their live health/page checks therefore remain unverified; in-process route
smoke tests passed where they do not require socket fixtures.

In an environment with the required permissions:

1. Run root `make fmt` and `make ci` without filtered tests.
2. Configure separate disposable PostgreSQL test databases, including the
   customer-facing extension setup documented in its README. Run
   `make test-postgres` in each changed service and the sourcing object-storage
   suite with its documented test configuration.
3. Start the changed services/apps with distinct `PORT` values and their normal
   dependencies. Check `/api/health` on services and `/` on customers; verify a
   sourced discounted offer, status/content clearing, and migration rollback.
4. Deploy downstream migrations/readers before sourcing's v8 writer. Stop event
   runners during the identity backfill, migrate, then restart. Mark the four
   TODOs `[x]` only after the remaining acceptance checks pass.

## Completion follow-up: 2026-10-27

Starting revision: monorepo `dfe0de1a344ee0ab0d0206b2b29b6ddbc95ef2d8`
on `wip/upgradez`, with a clean working tree. Sibling Mulac:
`dc60f443e4a2ed4a3e763855fe473426732baef1`; its existing untracked
`Makefile.orig` was untouched. The plan review's older revision remains
historical provenance. No commits or dependency changes were made.

The remaining test implementation was split among sourcing, downstream facts
and identity agents; offers aggregate/SQL checks and evidence were integrated
centrally. Reviews found no production defect requiring a behavior, migration,
wire-version or architecture change. The added tests cover:

- **Export persistence and recovery:** a shared five-variation
  [fixture](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/test_facts.rs)
  includes exact large/zero-decimal Money, null and omitted observations,
  historical raw statuses and exact HTML/Unicode/empty/whitespace content.
  [Database cases](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/postgres_tests/facts.rs)
  compare v6–v8 bytes, versions and IDs through prepare/upload/load/restart,
  coexistence with pending v1/completed v5, upload-success/completion-failure
  recovery without a readable original source, and invalid-input rejection
  before preparation. [Outbox tests](../../services/retailer_sourcing/src/offer_delivery/record_export_messages/postgres_tests.rs)
  retain exact payloads/IDs on replay. The [real-storage suite](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/live_tests.rs)
  adds current v8 and retains v1.
- **Offers transitions and replay:** [aggregate cases](../../services/offers/src/offers/aggregate/facts_tests.rs)
  check one event per independent change, exact record serialization/replay,
  unchanged neighboring state, no-op reimports and rejection before unrelated
  changes. [SQL cases](../../services/offers/src/postgres_tests/facts.rs) add
  nonnull single-field updates, comparison-only/combined large JPY pricing,
  exact empty/whitespace descriptions, unchanged identities/images and raw
  outbox replay. Populated down/up removes only the added facts and restores
  them as unknown, preserving the legacy row and pending command.
- **Downstream persistence and read/display contracts:** [fact cases](../../services/customer_facing/src/postgres_tests/facts.rs)
  compare full neighboring rows and browse/search REST/GraphQL after each
  independent transition, retain recognized schema.org strings as historical
  `Other`, preserve large integers and suppress incompatible legacy comparisons.
  Pending legacy/new commands and images survive populated down/up; the pending
  commands then execute. [Receipt tests](../../services/customer_facing/src/postgres_tests/projection.rs)
  inject pricing/content SQL failures and verify rollback, retry and replay.
  [Card tests](../../apps/customers/src/page/tests.rs) check exact large/zero-decimal
  sale formatting and byte-identical HTML when only statuses/content differ.
- **Identity backfill and queued work:** [migration cases](../../services/offers/src/read_offers_identifiers/postgres_tests.rs)
  reject URL/retailer conflicts and all five empty identifier fields without
  partial migration, and retain unmatched/source-only/different-ID rows.
  [Queued-work cases](../../services/offers/src/read_offers_identifiers/postgres_tests/queued_work.rs)
  process pending inbox/command imports after migration and preserve a pending
  complete v1 change's exact payload through redelivery without duplicate
  identities/outbox entries. A portable projector test rejects missing targets.
  Deployment instructions now capture the unmatched count with an explicit
  SELECT because the pinned Diesel connection discards PostgreSQL NOTICEs.

### Executed checks

| Command / scope | Result |
| --- | --- |
| Root `make fmt`; `git diff --check` | Passed. |
| Root `make ci` | Blocked in unchanged `inbox_client`: 1 passed, 7 failed at socket creation with `Operation not permitted`. |
| `services/offers`: `make fmt`, `make ci` | Formatting/all-target clippy passed; 80 tests passed, 4 existing HTTP publisher socket tests failed with `Operation not permitted`, 32 PostgreSQL tests ignored. |
| Offers portable retry, explicitly excluding those four socket tests | 80 passed, 32 ignored, 4 filtered. The seven aggregate fact tests passed, as did four portable identity tests; the six identity PostgreSQL tests compiled. |
| `services/retailer_sourcing`: `make fmt`, `make ci` | Formatting/all-target clippy passed; full CI stopped in existing `export_offer_sources` socket fixtures with `Operation not permitted`. |
| Sourcing export-focused / outbox-focused tests | 19 passed / 5 passed; 13 / 10 database/live tests ignored respectively. |
| Sourcing main binary, excluding its two existing publisher socket tests | 128 passed, 74 ignored, 2 filtered. |
| `services/customer_facing`: `make fmt`, `make ci` | Passed: 38 tests; 34 PostgreSQL tests ignored. |
| `apps/customers`: `make fmt`, `make ci` | Passed: 32 tests, including HTTP fixtures. This records that command's result; it does not establish live server socket access. |
| `make test-postgres` in all three services; sourcing `make test-object-storage` | Could not run: `DATABASE_URL_TEST` is absent. |

The portable offers retry uses these exact exclusions after the unfiltered
failure (no tests were disabled in source):

```sh
cd services/offers
cargo test --bin offers -- \
  --skip orchestration::customer_facing_publisher::regression_tests::all_event_types_preserve_raw_payload_and_metadata \
  --skip orchestration::customer_facing_publisher::regression_tests::preserves_the_command_and_delivery_identity_across_retries \
  --skip orchestration::customer_facing_publisher::tests::repeated_acceptance_keeps_identity_and_content \
  --skip orchestration::customer_facing_publisher::tests::failures_use_sanitized_customer_facing_messages
```

The sourcing main-binary retry uses
`cargo test --bin retailer_sourcing -- --skip platform::offers_publisher::tests::preserves_the_command_and_delivery_identity_across_retries --skip platform::offers_publisher::tests::reports_transport_failures_with_offers_messages`
from `services/retailer_sourcing`. Audit was unnecessary because no dependency
manifest or lockfile changed. Source selection/extraction code was unchanged;
the earlier corpus results were not rerun or represented as new evidence.

### Still blocked

The environment again denied local socket creation and Docker socket access.
Only PostgreSQL client tools are installed; no test database is configured.
New database/live tests compile but have not executed. This includes export
recovery, migration rollback, transactional receipt failures and queued work.

Live attempts used `PORT=43961 make serve` for offers, `43963` for
customer-facing, `43962` for customers and `43964` for sourcing. All built.
Offers/customer-facing stopped for missing `DATABASE_URL`; customers could not
listen (`Operation not permitted`); sourcing stopped before listening because
its object-storage base URL configuration was not an absolute HTTP(S) URL.
Health/page, full HTTP pipeline and desktop/mobile checks therefore remain
unverified in this follow-up.

At the end of this implementation follow-up, the four TODOs were left `[.]`:
implementation and planned tests were present, but the plans' database/storage/live
acceptance criteria were not yet met. The user subsequently requested marking
all four `[x]`; their tracking status now reflects that request. Run
the unfiltered suites and runtime procedures in the individual plans when
those prerequisites are available and append the results here.
