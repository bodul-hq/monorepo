# Product identifiers in the identity projection

Status: implementation and focused acceptance tests are present; database and
live acceptance remain blocked. The 2026-10-27 follow-up started from `dfe0de1`;
[current evidence](24-27-offer-facts-verification.md#completion-follow-up-2026-10-27)
records the results. Planning review HEAD: `d256fbc`.
This is an independent completion plan for TODO 27, not a new implementation of
the comparison-price, status or content plans. The historical
[verification record](24-27-offer-facts-verification.md) distinguishes executed
portable checks from PostgreSQL and live checks that remain unexecuted.

## Confirmed scope and current behavior

The Rust module is `read_offers_identifiers`; its SQL table is
`read_offer_identities`. [ADR 37](../adr/0037-identity-product-identifiers.md)
adds five nullable TEXT columns: `gtin`, `sku`, `shopify_product_gid`,
`shopify_product_variation_gid` and `shopify_handle`. Keep the existing copies
in `read_offers`, exact stored strings and retailer/URL identity rules. There is
no table rename, identifier-based matching, merging, deduplication, identifier
uniqueness constraint or additional export/event version in this task.

| Existing implementation | Actual responsibility |
| --- | --- |
| [Identity entity](../../services/offers/src/read_offers_identifiers/entity.rs) | Holds identity keys and typed `ProductIdentifiers`. |
| [Repository contract and guards](../../services/offers/src/read_offers_identifiers/repository.rs) | Validates identifiers and UUIDs before writes; rejects duplicate insert keys and missing update targets. |
| [PostgreSQL writer](../../services/offers/src/read_offers_identifiers/repository/postgres.rs) | Inserts five nullable values; one UPDATE replaces the complete set, including NULLs, without changing identity keys. |
| [In-memory writer](../../services/offers/src/read_offers_identifiers/repository/in_memory.rs) | Performs the same replacement while holding its mutex. |
| [Lookup repository](../../services/offers/src/infra/identifiers_repository.rs) | Finds by exact retailer/URL and decodes all five columns; its legacy key-only insert leaves them NULL. |
| [Projector](../../services/offers/src/read_offers_identifiers/projector.rs) and [registration](../../services/offers/src/main.rs) | Project existing `OfferCreated` and `OfferIdentifiersChanged` events; malformed payloads and repository failures propagate to Mulac. |
| [Migration up](../../services/offers/migrations/2026-10-27-000001_identity_product_identifiers/up.sql) / [down](../../services/offers/migrations/2026-10-27-000001_identity_product_identifiers/down.sql) | Backfill matching rows transactionally; rollback drops only these five identity columns. |

Import enrichment and projection replacement are distinct. Missing/null individual
identifier values in imports retain known values in the aggregate; an identifiers-change
event carries the complete decided set and can replace fields with NULL. Keep
that contract from [ADR 29](../adr/0029-offer-product-identifiers.md).
The [handler](../../services/offers/src/offers/handler.rs) still uses the identity
projection to find the aggregate. Finish an import's events before importing the
same retailer/URL again; this task does not repair existing publication or
concurrency limitations described in the [offers README](../../services/offers/README.md).

## Coverage and pending execution

Portable tests already cover creation/change mapping, malformed event payloads,
exact in-memory values, complete replacement, clearing, repeated replacement,
invalid inputs leaving state unchanged, missing targets and insert uniqueness.
Their historical pass is recorded in the shared verification document; it is
not evidence that a database migration has run successfully.

The six [dedicated PostgreSQL tests](../../services/offers/src/read_offers_identifiers/postgres_tests.rs)
compile but are ignored without `DATABASE_URL_TEST`. They currently cover:

- Up/down/up with full, partial and NULL identifier sets, Unicode, whitespace,
  leading zeroes, unmatched identities, source-only rows and a retailer/URL match
  under different IDs, with unchanged queue/source rows.
- Transactional rejection of same-ID URL/retailer conflicts and empty values in
  every identifier column, with unchanged schema, source, identity and queue rows.
- PostgreSQL writer/lookup round trips, case-sensitive URL lookup, duplicate
  product identifiers on different identities, complete replacement and clearing.
- Registered inbox/command/event processing for creation and a SKU-only change,
  comparing both projections after each import and retaining one identity row.
- [Actual pending inbox and command imports](../../services/offers/src/read_offers_identifiers/postgres_tests/queued_work.rs)
  crossing migration and restart, retaining legacy v5 history and unspecified
  identifiers while updating only the observed SKU.
- A pending complete v1 identifiers-change event retaining its raw payload,
  replacing fields including NULLs, and redelivering without another identity
  or changed/duplicated outbox entry.

The original dummy `LegacyPending` row remains a byte-preservation fixture;
the new queued-work module exercises valid work after migration. The portable
projector suite also verifies a missing target fails without a partial row.
These close the identified coverage gaps, but compilation and portable results
do not establish a successful PostgreSQL migration or runtime rehearsal.

## Completion checklist

1. **Establish a runnable verification environment.** Use Rust/libpq and the
   sibling Mulac checkout required by the [root README](../../README.md), plus
   PostgreSQL 16+ with a separate UTF8 test database. The test role needs CREATE
   SCHEMA; each test creates and drops its own schema. Enable local listening
   sockets for the ordinary suite and live service. Docker is optional; the
   previous environment had neither Docker access nor a PostgreSQL server.
   Use a separate development database for the live acceptance exercise.

2. **Migration matrix extended in the dedicated test file.** It includes a
   same-ID retailer conflict and empty values in all five columns. Assert migration failure leaves schema,
   identity rows, source rows and queued work unchanged. Keep whitespace valid:
   the identifier contract rejects empty strings/NUL, not padded source values;
   PostgreSQL TEXT cannot contain NUL. Add a source-only row and a same
   retailer/URL pair under different IDs: neither may be copied or reassigned.
   Unmatched identity values must remain NULL, with the reported count matching
   the fixture. Verify down/up retains keys, source identifiers and queue rows.

3. **Actual queued-work coverage added.** The tests use the existing
   isolated-schema helper. Seed legacy creation history and matching projections,
   queue a valid identifier-only import before applying the identity migration,
   then run inbox, commands and events in order after migration. Assert one
   identity, unchanged retailer/URL, preserved unspecified import identifiers,
   exact new SKU and equality of all five columns in both projections. Also
   retain an existing complete `OfferIdentifiersChanged` payload pending across
   migration and verify it applies after startup without rewriting its bytes.
   Use historical fixtures/records rather than inventing a new wire version.

4. **Registered redelivery and missing-target tests added.** Redeliver
   the same complete identifiers-change event and assert unchanged identity keys,
   correct full replacement including NULLs, no extra identity and no duplicate
   customer-facing outbox entry. Exercise a missing target through the projector
   and confirm a visible failure with no partial row. Keep creation replay and
   aggregate-publication idempotency outside this task. Existing invalid-input
   and repeated-update tests already cover the ordinary atomic update behavior;
   the PostgreSQL writer uses one statement. Add injected SQL failure coverage
   only if review identifies a real application error-propagation boundary that
   the existing tests miss, not to retest PostgreSQL's atomicity guarantee.

5. **Run the existing and extended suites.** Start with identity tests, then the
   entire offers PostgreSQL suite, which also exercises legacy key-only lookup,
   duplicate identities, full migration rollback/rebuild and import enrichment.
   Fix failures within this contract and rerun affected checks. Run ordinary
   offers CI without filters and preserve exact failure output if infrastructure
   still prevents it. Record the tested revision and commands after completion;
   do not relabel ignored tests or environment failures as passes.

6. **Rehearse deployment and accept the runtime behavior.** Use the preflight
   and live procedure below on disposable/restored data. Capture before/after
   row counts, the unmatched report, identifier equality and queue results.
   Update this plan's status and the shared verification record only after
   evidence exists, then mark the specific TODO complete.

## Verification commands and acceptance

Configure `DATABASE_URL_TEST` for a disposable database and `DATABASE_URL` for
the separate development database before running these commands. Use the real
connection configuration described in the offers README; do not copy placeholder
credentials into a result. Run from `services/offers`:

```sh
make fmt
make ci
cargo test --bin offers read_offers_identifiers -- --nocapture
cargo test --bin offers read_offers_identifiers::postgres_tests -- --ignored --nocapture
make test-postgres
make migrate
PORT=3001 make serve
```

`make migrate` requires Diesel CLI with PostgreSQL support; normal startup also
applies embedded migrations through [bootstrap](../../services/offers/src/bootstrap.rs).
In another terminal, `curl --fail http://localhost:3001/api/health` must return
`ok`. Submit a populated import through `/api/tmp-import`, run `/api/inbox/run`,
`/api/commands/run` and `/api/events/run`, then repeat with only the SKU changed.
Each runner takes `{"limit":100}`; inspect its `errors` array as well as HTTP
status. Compare SQL rows in `read_offer_identities` and `read_offers`: the UUID
and exact retailer/URL remain the same, all identifiers agree, and other values
remain intact. No identity read HTTP route is required. The full root CI remains
a repository completion check; its last attempt stopped in existing
`inbox_client` socket tests. No dependency audit is needed unless dependencies change.

## Migration preflight, rollout and rollback

Stop event runners and wait for active projection writes to finish before the
final preflight. On a restored copy first, use read-only versions of the SELECTs
in the migration to count exact ID/retailer/URL matches, same-ID key conflicts,
empty identifiers in matched source rows and identities with no source ID.
Record unmatched rows separately; do not infer values from GTIN, SKU or URL.
Resolve source contradictions explicitly before retrying; retain an unmatched
row as NULL when no authoritative source row exists. Capture source identifiers,
identity keys and queue state for before/after comparison.

The migration locks `read_offers` in SHARE mode and `read_offer_identities` in
ACCESS EXCLUSIVE mode. Diesel runs validation, column additions, backfill and
NULL-safe equality verification in one transaction. Confirm those locks can be
obtained in the deployment window. Apply schema before running the new lookup
code, verify zero conflicts/mismatches and the expected unmatched count, then
restart runners. The migration emits a PostgreSQL NOTICE, but the pinned Diesel
connection installs a no-op notice processor, so startup logs are not evidence
that the count was observed. Capture this read-only query explicitly (the tests
assert its result too):

```sql
SELECT count(*) AS unmatched_identities
FROM read_offer_identities i
WHERE NOT EXISTS (SELECT 1 FROM read_offers o WHERE o.id = i.id);
```

Existing history and queued payloads keep their formats.

Rehearse rollback through the isolated-schema tests: the down migration removes
only the new columns and leaves `read_offers`, identity keys and history intact;
reapplying reconstructs values from `read_offers`. Later migrations may need to
be unwound first in the disposable test schema. Coordinate any deployed rollback
with a binary that does not query removed columns. Do not use the destructive
`make migrate-redo` reset on retained data. Production data corrections, if any,
must preserve authoritative values rather than silently choosing a conflicting row.

Done means the dedicated and broader offers PostgreSQL tests actually pass,
ordinary CI and live health pass, queued imports and change redelivery preserve
one exact identity, and the migration rehearsal proves verified copying,
documented unmatched rows and reversible schema changes. Record any unresolved
environment blocker and leave TODO 27 unfinished until these criteria are met.
