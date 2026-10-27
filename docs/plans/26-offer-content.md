# Offer description content

Status: Implementation and focused acceptance tests are present; database,
object-storage and live acceptance pending. The 2026-10-27 follow-up started
from `dfe0de1`; [current evidence](24-27-offer-facts-verification.md#completion-follow-up-2026-10-27)
records the results. Planning review HEAD: `d256fbc`.

## Confirmed contract

[ADR 36](../adr/0036-offer-content.md) fixes the scope: reuse
`retailer_sauce::structuralized::content::Content` with exactly
`description: Option<String>`, nested through structuralization, export/import,
history, projections, APIs and the customers response model. Titles/images stay
flat. Use only the already-normalized primary product description.

Preserve the decoded string exactly: Unicode, HTML, entities, newlines, tabs,
leading/trailing whitespace and empty strings. Do not trim, sanitize, decode
entities again or collapse empty strings to null. Transport JSON escaping may
differ; decoded values must match. Prepared immutable export retries must also
retain their original bytes and IDs.

| Context/input | Required behavior |
| --- | --- |
| Implemented structuralizer has no primary description | Emit `content: {"description": null}`; no inferred fallback. |
| Legacy structuralized/export/import payload omits content | No observation: retain an existing description; initialize unknown on creation. A v8 export built from an old artifact may also omit content. |
| Present `{"description":""}` or whitespace-only string | Preserve and replace with that exact known string. |
| Present `{"description":null}` | Explicitly clear the description. |
| `content:null`, array/scalar, `{}`, wrong member type, extra keys | Reject. Present content must be an object with exactly one required nullable description key. |
| NUL in description | Reject through `Content::guard()` before business persistence/events. Shape deserialization alone is not the guard. |
| Historical creation v1–v7 omits content | Default to unknown without rewriting history. New `OfferCreated.v8` includes the complete object. |
| `OfferContentChanged` / stored `OfferContentChanged.v1` | Require `id` and complete content; replace the decided value, including null. |
| Customer-facing REST/GraphQL | Always return nonnull `content` with nullable `description`, identically for browse/search. |
| Customers receives an old response without content | Default to `Content { description: None }`, preserve existing HTML. This does not make present `{}` valid. |

Typed temporary imports validate before queuing. A generic inbox may accept an
envelope before command processing rejects its payload; no offer change or
applied-message receipt may survive that failure.

## Existing implementation

| Layer | Current paths and behavior to retain |
| --- | --- |
| Shared value/source selection | [structuralized.rs](../../lib/retailer_sauce/src/structuralized.rs), [observation.rs](../../lib/retailer_sauce/src/observation.rs); `lib/retailers_{anker,minisforum,ugreen}/src/offer_facts.rs` and each `src/<storefront>/offer_details/v1/structuralize.rs`. Helpers copy the optional primary description unchanged. |
| Source boundary | [Coverage report](../offer-facts-source-coverage.md): 44 completed structuralizers; expected nulls are listed. Anker KR and Mi/WIP stubs are outside that completed set. |
| Export | `services/retailer_sourcing/src/offer_discovery/processing_for_export/{entity,conversion,handler,store}.rs`: write v8/read v1–v8; older versions omit content, old artifact omission remains distinct from clearing; pinned references/prepared bytes stay immutable. Fixtures are in `services/retailer_sourcing/tests/fixtures/offer_export/`. |
| Offers import/history/delivery | `services/offers/src/offers/{command,handler,aggregate,event}.rs`, `offers/http/rest.rs`, `infra/records.rs`, and `orchestration/customer_facing_publisher.rs`: preserve observations, compare full content, write creation v8/read v1–v8, publish original payload/identity. HTTP routing uses `OfferContentChanged`, without `.v1`. |
| Projections/API | `services/offers/src/read_offer/` writes description; `services/customer_facing/src/offer_projection.rs` couples writes with original-message receipts. `offer_read.rs` and `offer_read/facts.rs` share browse/search REST/GraphQL mapping. |
| Customers | `apps/customers/src/offers.rs`, `offers/facts.rs`, and `page/tests.rs` validate/default content and verify descriptions never enter browse/search HTML. |

Existing additive migrations are offers
[`2026-10-27-000004_offer_content`](../../services/offers/migrations/2026-10-27-000004_offer_content/up.sql)
(`read_offers.description TEXT NULL`) and customer-facing
[`2026-10-27-000008_offer_content`](../../services/customer_facing/migrations/2026-10-27-000008_offer_content/up.sql)
(`offers.description TEXT NULL`). Old rows become null; there is no backfill.
Down migrations drop the new fact, preserving preceding columns and queues.

## Evidence and pending runtime checks

[Shared verification](24-27-offer-facts-verification.md) separates historical
results from the 2026-10-27 completion work. Database/live
acceptance remains unverified: test DB configuration was absent, Docker access
was denied, and localhost sockets were restricted. Ignored/filtered cases do
not count as passes. Existing combined fact tests cover content but their
PostgreSQL cases have not executed in the recorded environment.

The [shared sourcing cases](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/postgres_tests/facts.rs)
now exercise content-bearing v8 through preparation/upload/load and recovery
after failed completion, comparing exact strings, bytes and identities. The
[live suite](../../services/retailer_sourcing/src/offer_discovery/processing_for_export/live_tests.rs)
adds a current-v8 round trip while retaining v1. Neither requires a new version.

Focused tests now cover content-only updates, empty/whitespace TEXT/API round
trips, whole-state preservation, exact record replay, forced content-write
rollback/retry, and pending legacy/content messages processed after migration.
Populated down/up checks explicitly expect the dropped content to return as
unknown. Customers asserts byte-identical HTML with and without data-only
content/statuses. No production defect was found; database tests remain unrun.

## Completion checklist

1. Reconcile current source and affected `AGENTS.md` with the contract. Keep
   ADR 36 and current version numbers; do not add replacement migrations just
   to finish verification. Preserve old artifact/event readers and byte IDs.
2. Check completed structuralizers against primary normalized descriptions and
   the coverage report. Preserve old normalized-document defaults and deliberate
   unknowns. Do not expand source selection or implement stubs.
3. Added the coordinated content-bearing v8 storage/live fixture. It asserts exact
   description, prepared-body bytes and export/entry IDs through prepare/upload,
   completion/load, failed completion/restart and repeated delivery. Retain
   legacy-version coverage; do not regenerate old snapshots.
4. Added the missing contract cases from the audit. A content-only update emits
   one content event, an identical observation none; omission retains, null
   clears. Replay must not restore stale content. Assert unrelated title/images,
   price/status/identifiers stay unchanged and malformed input leaves no partial
   state or receipt. Exercise strict shape/NUL checks at each external boundary.
5. Execute populated migration and projection suites, then live acceptance below.
   Test exact strings, null initialization, pending work and up/down/up. Down
   drops descriptions: use disposable data; recovery needs backup or replay.
6. Record results and deploy downstream migrations/readers before sourcing v8.
   Coordinate other migrations' maintenance requirements separately. Update the
   shared verification/TODO only after item-specific acceptance passes.

## Tests and runtime prerequisites

Commands below are for the implementation follow-up, from repository root.
Require pinned Rust, Make, libpq, the compatible sibling Mulac checkout, socket
permissions and separate PostgreSQL 16+ UTF8 test databases with schema
create/drop privileges. Provision `pg_trgm` in trusted
`customer_facing_extensions` with role USAGE before customer-facing migrations;
follow its [setup](../../services/customer_facing/README.md#title-search).

```sh
cargo test --manifest-path lib/Cargo.toml -p retailer_sauce content_tests
cargo test --manifest-path lib/Cargo.toml -p retailers_anker -p retailers_minisforum -p retailers_ugreen description
cargo test --manifest-path services/Cargo.toml -p retailer_sourcing --bin retailer_sourcing processing_for_export
cargo test --manifest-path services/Cargo.toml -p offers --bin offers facts_tests
cargo test --manifest-path services/Cargo.toml -p customer_facing --bin customer_facing offer_projection::tests
cargo test --manifest-path apps/Cargo.toml -p customers
DATABASE_URL_TEST="$OFFERS_TEST_DATABASE_URL" make -C services/offers test-postgres
DATABASE_URL_TEST="$CUSTOMER_FACING_TEST_DATABASE_URL" make -C services/customer_facing test-postgres
DATABASE_URL_TEST="$SOURCING_TEST_DATABASE_URL" RUST_TEST_THREADS=4 make -C services/retailer_sourcing test-postgres
DATABASE_URL_TEST="$SOURCING_TEST_DATABASE_URL" make -C services/retailer_sourcing test-object-storage
```

Set the three test URL variables to separate disposable databases. Real storage
requires exported `OBJECT_STORAGE_URL`/`OBJECT_STORAGE_TOKEN` for a test-backed
instance with the pipeline and `offer_export` buckets; immutable test objects
remain remotely. Require the new v8 live case, not just the existing v1 helper.
Use migration tests in both services' `src/postgres_tests/facts.rs`; apply pending
migrations with `DATABASE_URL=... make migrate` or startup. Inspect migration
versions before any targeted rollback; never use `make migrate-redo` here.

For an available external corpus at `../data/<retailer>/offer_details/sources`,
run `OFFER_FACTS_MAX_PAGES_PER_STOREFRONT=1 cargo test --manifest-path lib/Cargo.toml -p retailer_data_extractor --test offer_facts_corpus -- --ignored --nocapture`.
Require actual storefront rows/exact-description checks: the test returns
successfully if the corpus is absent. A bounded sample is not a full-corpus pass.
After code changes run each affected crate's `make fmt` then `make ci`; final
acceptance requires unfiltered root `make fmt`, `make ci`, and `git diff --check`.
Check nonzero test counts; audit dependencies only if manifests/locks change.

## Live acceptance

Use separate acceptance databases and exported test object-storage settings;
run each server in its own terminal:

```sh
DATABASE_URL="$CUSTOMER_FACING_ACCEPTANCE_DATABASE_URL" PORT=3003 make -C services/customer_facing serve
DATABASE_URL="$OFFERS_ACCEPTANCE_DATABASE_URL" CUSTOMER_FACING_URL=http://127.0.0.1:3003 PORT=3001 make -C services/offers serve
DATABASE_URL="$SOURCING_ACCEPTANCE_DATABASE_URL" OFFERS_URL=http://127.0.0.1:3001 PORT=3002 make -C services/retailer_sourcing serve
CUSTOMER_FACING_URL=http://127.0.0.1:3003 PORT=3004 make -C apps/customers serve
curl --fail-with-body http://127.0.0.1:3001/api/health
curl --fail-with-body http://127.0.0.1:3002/api/health
curl --fail-with-body http://127.0.0.1:3003/api/health
curl --fail-with-body http://127.0.0.1:3004/
```

Run the v8 fixture/storage/delivery path to certify sourcing. For downstream
checks, POST `/api/tmp-import` on port 3001 with `content-type: application/json`
and this body (require 202):

```json
{"retailer_code":"ankerau","title":"ContentAcceptance","price_amount":1999,"currency":"AUD","url":"https://example.com/content-acceptance","content":{"description":" \t<p>ž &amp; source</p>\n"}}
```

POST `{"limit":100}` to offers' `/api/inbox/run`, `/api/commands/run`,
`/api/events/run`, `/api/outbox/run`, then customer-facing's `/api/inbox/run`
and `/api/commands/run`. Require successful processed counts and empty errors,
not merely HTTP 200. From sourcing, first run its producing command/event
batches and outbox. Processing is explicit, not automatic.

Check port 3003 GET `/api/offers?limit=100`, GET
`/api/offers/search?q=ContentAcceptance`, and POST `/graphql` with
`{"query":"{ offers(limit:100) { items { id content { description } } } searchOffers(q:\"ContentAcceptance\") { items { id content { description } } } }"}`.
Compare decoded descriptions with the input and both databases' description
columns. Repeat with a content-only change, omission, null, empty and whitespace
strings; keep every other import field fixed. Verify no event for identical
reimports. Use isolated DB tests for original-message replay/missing-parent
retry. Check customers `/search?q=ContentAcceptance`: description-only tokens
and markup must be absent from HTML, with existing cards unchanged.

## Done criteria and non-goals

Complete when source equality, strict presence semantics, independent changes,
legacy compatibility, v8 persistence/recovery, populated migrations, atomic
receipts, API parity and non-rendering have passing recorded evidence, including
required CI/database/live checks. Environmental blockers remain pending work.
No title/image nesting, unused generic Offer/Product scaffold adoption, SEO/DOM
fallback, new description extraction, rendering/sanitization feature, description
search, new endpoints, stub implementation or historical rewrite is included.
Pricing, statuses and identifier backfill retain their separate plans.
