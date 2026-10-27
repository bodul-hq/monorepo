# Offer facts implementation and verification

Status on 2026-10-26: items 17–20 are implemented in separate commits, as requested.
Items 18–20 still require final database/live verification. TODO is marked done
at the user’s request. Item 21 was planned concurrently and is outside this round.

## Implemented contracts

- Item 17 carries successful download time with the pinned source artifacts,
  immutable exports and monotonic offer/customer-facing scrape-time updates.
- Item 18 preserves each Testimony revision in Mulac event and offers outbox
  metadata, with optional legacy fields and a positive PostgreSQL BIGINT bound.
- Item 19 carries source-backed market/language plus extraction diagnostics.
  [Coverage](../localization-source-coverage.md) records 133 saved pages across
  45 storefronts; 130 complete pipelines and three Anker Korea pages without a
  structuralized document. Portable fixtures exclude payment/widget settings.
- Item 20 carries exact optional product identifiers, enriches missing values,
  and emits a complete identifier-set event for corrections. Identifier changes
  preserve retailer/URL identity. Incomplete change-event payloads are rejected.
- Export readers support v1–v5; new exports use v5. Creation history supports
  stored versions v1–v5; new creation records use v5. Additive migrations extend
  both read projections. Customers receives the facts without displaying them.

See ADRs [26](../adr/0026-offer-scrape-provenance.md),
[27](../adr/0027-offer-revision-metadata.md),
[28](../adr/0028-source-backed-offer-localization.md), and
[29](../adr/0029-offer-product-identifiers.md), and the
[queue walkthrough](../offer-facts-walkthrough.md).

## Verification completed

- Root `make fmt` and `make lint` pass across lib, services and apps.
- `cargo test --offline -p retailer_sauce -p retailer_data_extractor` passes:
  shared types, portable extraction fixtures, facade tests and saved-corpus
  extraction/normalization/structuralization checks.
- In services, `cargo test --offline -p offers offers::` passes 19 tests;
  `cargo test --offline -p customer_facing` passes 30 with 21 PostgreSQL tests
  ignored; `cargo test --offline -p retailer_sourcing offer_discovery::` passes
  50 with 25 PostgreSQL tests ignored. Ignored tests are not passing evidence.
- Customers' two pure offer-response tests pass. The two HTTP client tests fail
  while binding fixture listeners, with `Operation not permitted`.
- Mulac's two targeted commanding/eventing sequence tests and two outbox
  metadata conversion tests pass, using a writable monorepo target directory
  and the sibling's locked dependencies.
- Item 17 had a successful root CI run, isolated PostgreSQL suites (20
  customer-facing, 21 offers, 60 sourcing cases), and live health/queue checks
  before the later items were implemented. Its live sequence checked creation,
  a newer scrape-only update, and an older replay. These results do not establish
  final integration success for the combined implementation.

## Commits and remaining checks

The user requested separate commits after initially requesting one combined
monorepo commit. The dependency was committed first, then each planned item:

- Mulac prerequisite: `dc60f443e4a2ed4a3e763855fe473426732baef1`.
- Item 18, event revisions: `8cf7d6a`.
- Item 17, scrape time: `6fdc165`.
- Item 19, source-backed localization: `9f1e48d`.
- Item 20: the commit containing this record, “Carry product identifiers through offer projections and APIs”.

Intermediate snapshots were checked separately. Item 18 passed service compilation
and 14 offers tests. Item 17 passed service compilation/lint, 15 offers tests,
49 sourcing tests and 30 customer-facing tests. Item 19 passed service/app
compilation/lint, 17 offers tests, 49 sourcing tests, 30 customer-facing tests
and the customers response-contract test. The final implementation's checks are
recorded above. Unrelated sibling `Makefile.orig` and item-21 planning changes
were preserved outside these commits.

Socket restrictions remain. Root `make ci` was retried and stops at seven
`inbox_client` fixture-listener failures with `Operation not permitted`. Mulac
CI was retried using a writable target directory; it reaches outbox tests but
two broker fixture listeners fail with the same error. The new database tests
compile but have not run; final live services and customer page checks remain
pending. Git operations succeeded through the available Git tooling.

`make audit` could not lock the protected advisory cache. A copy of the cached
advisory database was compared against HEAD and working lockfiles with
`cargo audit --no-fetch --no-yanked --stale`: **no new vulnerability or warning
IDs** in any workspace. Existing lib/apps advisories are RUSTSEC-2026-0382 and
RUSTSEC-2026-0383; services already reports RUSTSEC-2026-0346, 0347, 0348, 0349,
0367, 0368, 0369 and 0382. This is a cached comparison, not a refreshed audit or
a clean bill of health. Refresh the audit when network/cache access is available.

Once socket/database access is available, run full CI in both repositories, the
isolated PostgreSQL suites and live walkthrough, then record their results.
