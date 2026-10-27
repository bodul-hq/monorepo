# Offer comparison pricing

Status: Implementation and focused acceptance tests are present; database,
object-storage and live acceptance remain blocked. Follow-up on 2026-10-27
started from `dfe0de1`; see the [current evidence](24-27-offer-facts-verification.md#completion-follow-up-2026-10-27).

Reviewed against `d256fbc` on `wip/upgradez`. This plan completes the existing
implementation; it does not restart it. The previous execution evidence and
environmental limitations are recorded in
[the verification report](24-27-offer-facts-verification.md).

## Confirmed behavior

Carry a retailer's explicit comparison price for the same variant, in the
payable price's currency and strictly above its amount. Preserve the existing
payable-price selection. Display a valid comparison in smaller crossed-out text
on customer cards, using the existing currency formatter and accessible wording.
[ADR 34](../adr/0034-source-comparison-pricing.md) is the accepted contract.

| Input or event | Required result |
| --- | --- |
| Source comparison greater than current price in the same currency | Carry exact integer Money; show both amounts. |
| Missing source comparison, no payable price, equal/lower amount, wrong currency, or conflicting valid comparisons | Structuralizer emits unknown comparison; do not infer a discount. |
| Import omits `was_price` | Preserve a stored comparison if compatible with the incoming current price. |
| Import supplies `was_price: null` | Clear the comparison. |
| Legacy import changes current price so the retained comparison is incompatible | Emit one atomic pricing event with the new current price and null comparison. |
| New import supplies an invalid comparison | Reject before aggregate mutation or persistence. |
| Historical `OfferPriceChangedEvent` | Replay only current price; retain stored comparison even if it becomes incompatible. Suppress that comparison in read responses and cards. |

Export v6 introduced the field; current v8 retains it. Creation readers accept
v1–v8 and the current writer uses `OfferCreated.v8`. New changes route as
`OfferPricingChanged` and store as `OfferPricingChanged.v1`, carrying required
`pricing: {price, was_price}`. `was_price` may be null but must not be omitted
from that event. The existing `OfferPriceChanged.v1` record and
`OfferPriceChangedEvent` routing key remain price-only.

## Existing implementation to retain

| Boundary | Current implementation |
| --- | --- |
| Retailer evidence | Family helpers in [Anker](../../lib/retailers_anker/src/offer_facts.rs), [Minisforum](../../lib/retailers_minisforum/src/offer_facts.rs), and [Ugreen](../../lib/retailers_ugreen/src/offer_facts.rs). Ugreen accepts scoped explicit ListPrice/StrikethroughPrice specifications only when valid comparisons agree. See [storefront coverage](../offer-facts-source-coverage.md). |
| Immutable export | [Conversion](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/conversion.rs), [schema validation](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/entity.rs), and [store](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/store.rs) preserve versioned omission/null semantics. |
| Offers import and history | [Observation type](../../lib/retailer_sauce/src/observation.rs), [import command](../../services/offers/src/offers/command.rs), [aggregate](../../services/offers/src/offers/aggregate.rs), [events](../../services/offers/src/offers/event.rs), and [record versions](../../services/offers/src/infra/records.rs). JSON money objects are validated before use. |
| Persistence and delivery | [Offers projection](../../services/offers/src/read_offer/repository/postgres.rs), [outbox publisher](../../services/offers/src/orchestration/customer_facing_publisher.rs), and [customer-facing projection](../../services/customer_facing/src/offer_projection.rs). Pricing updates replace the whole pair atomically. |
| Read and display | [API mapping](../../services/customer_facing/src/offer_read/facts.rs), [customers model](../../apps/customers/src/offers.rs), and [card renderer](../../apps/customers/src/page.rs). Incompatible stored comparison prices are hidden without rewriting history. |

Both projections have additive nullable `was_price_amount` / `was_price_currency`
columns and paired-null constraints: [offers migration](../../services/offers/migrations/2026-10-27-000002_offer_was_price/up.sql)
and [customer-facing migration](../../services/customer_facing/migrations/2026-10-27-000006_offer_was_price/up.sql).
Existing rows become unknown, and event history and queues are preserved.

## Completion checklist

1. **Confirm the checkout and executable test environment.** Record the current
   monorepo and sibling Mulac revisions, and inspect the existing diff before
   changes. Use the pinned toolchain, libpq, localhost socket access, PostgreSQL
   16+ test databases and roles able to create/drop isolated schemas. Provision
   customer-facing's `pg_trgm` extension in its trusted schema as described in
   [its README](../../services/customer_facing/README.md). Real export checks also
   need a disposable object-storage instance and its URL/token. Keep each
   service's `DATABASE_URL_TEST` separate from operational databases. Missing
   permissions or services remain an explicit verification block.

2. **Persisted-export coverage added.** The shared
   [fact fixture](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/test_facts.rs)
   and [database cases](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/postgres_tests/facts.rs)
   cover exact large Money, explicit null and omission in v6–v8, repeated
   preparation/load, version coexistence with legacy pending/completed exports,
   and upload-success/completion-failure recovery after restart. Assertions
   compare raw bytes, export/entry/message IDs and source/version association.
   The [live suite](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/live_tests.rs)
   now has separate v1 and current-v8 cases. Execution against PostgreSQL and
   real object storage is still required; no production version was changed.

3. **Focused pricing matrix extended; database execution pending.** The
   [aggregate tests](../../services/offers/src/offers/aggregate/facts_tests.rs),
   [offers database cases](../../services/offers/src/postgres_tests/facts.rs),
   [customer-facing database cases](../../services/customer_facing/src/postgres_tests/facts.rs),
   and [customer rendering tests](../../apps/customers/src/page/tests.rs)
   now cover comparison-only and combined changes with full neighboring-state
   snapshots, exact record replay, independent SQL updates/outbox replay, API
   parity and exact large/zero-decimal display. Retain these acceptance cases:
   sale creation; comparison-only change; simultaneous current/comparison
   change; identical observation producing no event; explicit clearing; legacy
   omission retaining a compatible value; legacy omission clearing an incompatible
   value; invalid/missing event fields rejected without partial state or receipt;
   repeated delivery retaining the latest projection; and historical price-only
   replay retaining stored comparison data while both REST and GraphQL suppress
   an incompatible pair. Include zero-decimal currency and exact large-integer
   values when exercising the new pair through serialization and display.

4. **Execute database and immutable-storage checks.** Run `make fmt` before
   unfiltered `make ci` in each changed folder, or once from the root. Then run
   `make test-postgres` in `services/retailer_sourcing`, `services/offers`, and
   `services/customer_facing`, with each folder's test database configured.
   Run sourcing's `make test-object-storage` with its documented test variables.
   Confirm additive upgrade, paired-null rejection, downgrade/re-upgrade, and
   preservation of legacy offer fields, receipts and pending work. Migration
   down/up checks belong in disposable test schemas; down migrations remove
   comparison data and are not a lossless operational rollback. Run `make audit`
   only if a Cargo manifest dependency or lockfile changes.

5. **Verify the live path and card presentation.** Start object storage and the
   three pipeline services against disposable databases, with distinct ports
   and explicit downstream origins; start customers with `CUSTOMER_FACING_URL`
   pointing at the test consumer. Use `PORT=<port> make serve` and check service
   `/api/health` routes plus the customers `/` page. Through the documented
   pipeline APIs prepare a pinned source containing a valid comparison, then
   explicitly run sourcing commands/events/outbox, offers inbox/commands/events/
   outbox, and customer-facing inbox/commands. Check both projections, REST
   browse/search, GraphQL and customer homepage/search cards. Repeat with a
   comparison-only change and an explicit clear. On desktop and narrow mobile
   layouts, check smaller `del` text, current-price prominence, no clipped action,
   existing currency formatting and a single accessible “Was …; now …” phrase.
   Replaying a delivered message must not regress the projection. Queue workers
   remain explicitly driven; the walkthrough must not assume background runners.

6. **Record evidence and complete this TODO independently.** Update the shared
   verification report with commands, revisions and results, including the new
   pricing persistence cases and live observations. Describe any proven defect
   and its narrowly scoped fix; do not classify environmental failures as code
   defects. Mark only the comparison-price TODO `[x]` once the criteria below
   pass. Other items can remain verification-pending.

## Dependencies and rollout

Pricing shares source variants, creation records, projections and fixtures with
[availability/condition](25-offer-availability-condition.md) and
[content](26-offer-content.md). Coordinate changes to those shared files without
reverting current v8 writing to v6. Their independent display behavior remains
unchanged. [Identity columns](27-identity-product-identifiers.md) do not introduce
a pricing dependency, but the normal migration chain includes that migration
and its requirement to pause event runners during backfill.

Deploy migrated customer-facing readers first, then migrated offers readers and
publishers, then customers, and finally the sourcing writer. Before rolling back
any reader, stop newer publication and account for queued newer events; an old
consumer cannot be assumed to understand `OfferPricingChanged`. Existing
snapshots receive no inferred comparison price. Population requires a new
source observation through the normal pipeline.

## Completion criteria and boundaries

Completion requires successful unfiltered CI, the new persisted pricing cases,
both projection migration checks, actual HTTP delivery with replay protection,
REST/GraphQL parity, and live desktop/mobile sale/clear rendering. Record blocked
checks honestly and leave `[.]` until they pass. This planning review does not
claim that previously unavailable integration checks have run.

No historical-price inference, coupon or percentage calculations, cross-variant
price copying, historical snapshot rewrite, automatic backfill, new matching or
identity rules, or additional sale badge is included. No new product decision
is needed for the accepted behavior; any proposed change to that behavior must
be raised before implementation and recorded in a superseding ADR if accepted.
