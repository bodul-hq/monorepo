# Complete offer availability and item condition

Status: implementation and focused acceptance tests are present; database and
live acceptance pending. The 2026-10-27 follow-up started from `dfe0de1`;
[results and blockers](24-27-offer-facts-verification.md#completion-follow-up-2026-10-27)
are recorded separately from the planning review. Reviewed HEAD:
`d256fbc`; implementation baseline: `a79c3eb`. Recheck the branch/diff and
[shared verification](24-27-offer-facts-verification.md) before execution.

## Confirmed scope and semantics

[ADR 35](../adr/0035-source-availability-and-condition.md) fixes the scope:
source-backed optional facts through customers' response model: no status badges,
filtering, or purchase rules. No product decision remains open.

- Purchase flags mean `available`/`unavailable`, not physical inventory.
  Explicit stock evidence means `in_stock`/`out_of_stock`/`sold_out`.
- Condition is explicit `new` or `{"other":"exact source text"}`. Missing
  evidence is unknown; never infer new/refurbished from a title.
- Match evidence to its variant. An unscoped single offer applies only to a
  single-variant product; ambiguous or unmatched sibling evidence stays unknown.
- Interpret exact HTTP/HTTPS schema.org URLs only on new source processing.
  Historical `Other` values retain their enum identity and exact text, including
  significant whitespace and Unicode. New guarded observations reject blank/NUL
  raw statuses; replay must not call the source constructor again.

| Import/export field | Existing offer | New offer |
| --- | --- | --- |
| Omitted | Preserve that fact | Unknown |
| Explicit null | Clear that fact | Unknown |
| Valid value | Replace that fact | Store that value |

Fresh structuralizers emit both keys, including nulls. Change events require their
key even when null; a missing key is invalid. Each status
changes independently. SQL NULL represents unknown, not JSONB `null`. REST uses
nullable `{kind, raw}` objects with lowercase kinds; GraphQL uses uppercase enum
kinds and `itemCondition`. Only `other` has a nonnull `raw`.

## Current implementation and evidence

| Stage | Existing files / behavior |
| --- | --- |
| Shared types | [Availability](../../lib/retailer_sauce/src/availability.rs), [ItemCondition](../../lib/retailer_sauce/src/item_condition.rs), [Observation and required-nullable decoding](../../lib/retailer_sauce/src/observation.rs). |
| Source mapping | All 44 completed structuralizers; [coverage report](../offer-facts-source-coverage.md) records sources and exclusions. Scope fixtures include [Anker AU](../../lib/retailers_anker/src/ankerau/offer_details/v1/structuralize.rs), [Anker EU](../../lib/retailers_anker/src/ankereu/offer_details/v1/structuralize.rs), [Ugreen CA](../../lib/retailers_ugreen/src/ugreenca/offer_details/v1/normalized.rs), [Ugreen DE](../../lib/retailers_ugreen/src/ugreende/offer_details/v1/structuralize.rs), and [Minisforum US](../../lib/retailers_minisforum/src/minisforumus/offer_details/v1/normalized.rs). |
| Export | [Conversion](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/conversion.rs) and [entities](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/entity.rs) preserve omission/null and pinned versions; statuses begin at v7, current writer is v8. |
| Offers | [HTTP import](../../services/offers/src/offers/http/rest.rs), [command](../../services/offers/src/offers/command.rs), [aggregate](../../services/offers/src/offers/aggregate.rs), [events](../../services/offers/src/offers/event.rs), and [records](../../services/offers/src/infra/records.rs): creation reads v1–v8/writes v8; separate status changes use v1 records. |
| Projection/delivery | [Offers projector](../../services/offers/src/read_offer/projector.rs), [outbox subscriber](../../services/offers/src/subscribers/customer_facing.rs), [publisher](../../services/offers/src/orchestration/customer_facing_publisher.rs), [customer-facing projection and receipts](../../services/customer_facing/src/offer_projection.rs). |
| Storage | [Offers migration](../../services/offers/migrations/2026-10-27-000003_offer_status_facts/up.sql) adds nullable JSONB; [customer-facing migration](../../services/customer_facing/migrations/2026-10-27-000007_offer_availability_condition/up.sql) also checks JSON shape. Application guards and database constraints are distinct; do not assume constraint parity. |
| Read/model | [API adapters](../../services/customer_facing/src/offer_read/facts.rs), [read repository](../../services/customer_facing/src/offer_read.rs), [customers model](../../apps/customers/src/offers.rs), and [strict status decoding](../../apps/customers/src/offers/facts.rs). Statuses remain data only. |

The deterministic sample covers 44 pages/82 variations: availability is nonnull
for 81 and condition for 10 (1 and 72 nulls). This does not establish full-storefront
completeness. Ugreen COM has no accepted
availability source; selected-only facts can leave siblings unknown. Anker KR
and Mi/WIP stubs are outside the completed 44.

## Completion checklist

**1. Transition/replay coverage extended.** No production defect was found.
Independent transitions now compare full neighboring state and exact recorded
events; historical recognized schema.org URLs remain `Other`. Upstream SQL
update/outbox replay, downstream browse/search REST/GraphQL and customers
decoding/non-rendering have focused assertions in
[aggregate fact tests](../../services/offers/src/offers/aggregate/facts_tests.rs),
[publisher regression tests](../../services/offers/src/orchestration/customer_facing_publisher/regression_tests.rs),
[offers DB facts](../../services/offers/src/postgres_tests/facts.rs), and
[customer-facing DB facts](../../services/customer_facing/src/postgres_tests/facts.rs).
Database cases compile but await execution. The required matrix remains:

| Fixture / transition | Required assertion |
| --- | --- |
| Purchasable with `currently_not_in_stock=true` when the island omits explicit stock evidence; duplicate SKU; selected-only offer; single-variant aggregate | Purchase and inventory remain distinct; no facts leak to siblings. |
| `available → unavailable`, `in_stock → out_of_stock → sold_out` | One correct availability change per changed value; condition and unrelated fields unchanged after each step. |
| `new → Other → new`; each fact → null → known | Independent condition/availability changes; identical observations emit no event. |
| Import omits one/both keys; explicitly clears only one | Omission preserves state and null clears only the supplied field. |
| Historical `{"other":"http://schema.org/OutOfStock"}` and `{"other":" Refurbished ž "}` | Exact Other identity/text survives record replay, publication, DB, REST/GraphQL, and customers decoding. |
| Missing change-event key, boolean, unknown enum spelling, blank/NUL Other | Invalid input does not mutate projections or leave an applied receipt. |
| Duplicate delivery, retry, update before creation | Stable delivery identity, receipt protection, visible retry until creation; no new ordering guarantees. |

Use shared [v6](../../services/retailer_sourcing/tests/fixtures/offer_export/import_payload_v6.json),
[v7](../../services/retailer_sourcing/tests/fixtures/offer_export/import_payload_v7.json),
and [v8](../../services/retailer_sourcing/tests/fixtures/offer_export/import_payload_v8.json)
fixtures: v6 omits statuses, v7 supplies them, v8 adds content. The shared
[v6–v8 persistence/recovery cases](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/postgres_tests/facts.rs)
now cover status values, nulls, omission and raw historical strings alongside
pricing/content, with unchanged pinned bytes and export/entry/message identities.
The real-storage suite retains v1 and adds current v8; these cases still need
PostgreSQL and object-storage execution.

**2. Re-run required checks with usable sockets.** Previous root CI stopped at
socket creation; DB suites lacked `DATABASE_URL_TEST`. Those are unverified
acceptance gates, not passing checks. Run root `make fmt`, then unfiltered
`make ci`; use the shared verification record to target failures. Audit only
if a subsequent fix changes Cargo dependencies. The optional read-only sample is:

```sh
OFFER_FACTS_MAX_PAGES_PER_STOREFRONT=1 cargo test --manifest-path lib/Cargo.toml -p retailer_data_extractor --test offer_facts_corpus -- --ignored --nocapture
```

Record absent corpus inputs and parser failures. The full multi-GB scan is
optional; do not rewrite saved corpus artifacts for tests.
Known unrelated limitations are Anker COM `products-a2628.html` (`filterPrice`)
and NAS AU/CA `products-worry-free-purchase.html` float parsing; retain them in
the coverage report without expanding this item into source-schema repair.

**3. Execute persistence and migration acceptance.** Use the toolchain, libpq,
PostgreSQL 16+, registry, and sibling Mulac prerequisites in the [root README](../../README.md).
Configure distinct disposable UTF8 test databases with CREATE SCHEMA permission;
provision trusted `customer_facing_extensions.pg_trgm` per the [customer-facing README](../../services/customer_facing/README.md).
Set the following test URL variables, then run:

```sh
DATABASE_URL_TEST="$OFFERS_TEST_DATABASE_URL" RUST_TEST_THREADS=4 make -C services/offers test-postgres
DATABASE_URL_TEST="$CUSTOMER_FACING_TEST_DATABASE_URL" RUST_TEST_THREADS=4 make -C services/customer_facing test-postgres
DATABASE_URL_TEST="$SOURCING_TEST_DATABASE_URL" RUST_TEST_THREADS=4 make -C services/retailer_sourcing test-postgres
DATABASE_URL_TEST="$SOURCING_TEST_DATABASE_URL" OBJECT_STORAGE_URL="$TEST_OBJECT_STORAGE_URL" OBJECT_STORAGE_TOKEN="$TEST_OBJECT_STORAGE_TOKEN" make -C services/retailer_sourcing test-object-storage
```

The last command requires a running test-backed object-storage instance and
creates immutable test artifacts. Require the linked DB fact suites to prove
legacy rows gain SQL NULL, exact JSONB survives changes/replay, invalid commands
leave no receipt, updates retry after creation, and browse/search transports
agree. Require the existing populated migration round-trip tests to preserve
old rows, images, queues, identities, and receipts. Verify status-column
rollback only on a disposable clone, respecting later content migrations;
dropping columns loses their new values. Do not use `migrate-redo --all`.

**4. Run live acceptance.** Configure separate development databases and the
same extension prerequisite. In separate terminals, with test object storage
already running, start downstream before upstream:

```sh
DATABASE_URL="$CUSTOMER_FACING_DATABASE_URL" PORT=3003 make -C services/customer_facing serve
DATABASE_URL="$OFFERS_DATABASE_URL" CUSTOMER_FACING_URL=http://localhost:3003 PORT=3001 make -C services/offers serve
DATABASE_URL="$SOURCING_DATABASE_URL" OBJECT_STORAGE_URL="$TEST_OBJECT_STORAGE_URL" OBJECT_STORAGE_TOKEN="$TEST_OBJECT_STORAGE_TOKEN" OFFERS_URL=http://localhost:3001 PORT=3002 make -C services/retailer_sourcing serve
CUSTOMER_FACING_URL=http://localhost:3003 PORT=3004 make -C apps/customers serve
```

Check all service `/api/health` endpoints, customer-facing `/api/openapi.json`,
and customers `/` and `/search?q=charger`. Start a reproducible acceptance offer:

```sh
curl --fail-with-body -sS http://localhost:3001/api/tmp-import -H 'Content-Type: application/json' --data '{"retailer_code":"ankerau","title":"Status acceptance charger","price_amount":1999,"currency":"AUD","url":"https://example.com/plan-25-status","availability":"available","item_condition":{"other":" Refurbished ž "}}'
```

After each submission, POST `{"limit":100}` to offers `/api/inbox/run`,
`/api/commands/run`, `/api/events/run`, `/api/outbox/run`, then customer-facing
`/api/inbox/run` and `/api/commands/run`. Inspect batch errors/state; `202` receipt
alone is not success. Finish offers events before another import of the same
URL because identity lookup depends on projection. Repeat the same required
form fields with the transition matrix above. Also submit v7/v8 fixtures via
`/api/inbox` using routing key `ImportOfferDetails`; these use nested `price`,
not the flat `/api/tmp-import` DTO. New observations use new message UUIDs;
delivery retries keep their original identity.

Compare the acceptance URL's statuses in offers `read_offers`, customer-facing
`offers`, Testimony history, and outbox payloads. Query REST browse/search and
GraphQL browse/search, selecting `availability { kind raw }` and
`itemCondition { kind raw }`; for example:

```sh
curl --fail-with-body -sS 'http://localhost:3003/api/offers/search?q=Status%20acceptance%20charger'
curl --fail-with-body -sS http://localhost:3003/graphql -H 'Content-Type: application/json' --data '{"query":"{ searchOffers(q: \"Status acceptance charger\") { items { id url availability { kind raw } itemCondition { kind raw } } } }"}'
```

Assert REST/GraphQL kind casing, exact raw strings, and null parity. Customers
must decode new/legacy responses while cards add no status labels/raw text.
Finally drive one freshly sourced offer through the documented sourcing queues
and this downstream flow; fixture injection alone does not prove the full path.

## Dependencies, rollout, and done criteria

Statuses follow [pricing plan 24](24-offer-comparison-pricing.md) at v7;
[content plan 26](26-offer-content.md) adds v8. Keep the current v8 writer; no new
version is needed. Identity identifiers are independent; a combined release
respects their migration window. Deploy customer-facing migrations/readers,
offers migrations/readers and publisher, then sourcing. Preserve historical
artifacts and queued versions; populate unknown statuses through new observations.
Stop newer writers before rollback and retain recoverable removed-column data.
The [older walkthrough](../offer-facts-walkthrough.md) describes v5; use current fixtures and record its refresh separately.

Done means the status matrix, unfiltered CI, DB/migration/object-storage checks,
and live source-to-customers flow pass with recorded evidence and source
limitations. Then update shared verification and mark this TODO complete.
Non-goals: status UI, stock quantities, guessed condition categories, new
retailers, unrelated parser fixes, price/content policy, historical backfills,
queue resets, or new event-order guarantees.
