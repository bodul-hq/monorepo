- [x] Implement lowercase retailer serialization and shared/service compatibility tests
  ([ADR 49](docs/adr/0049-lowercase-retailer-serialization.md)).
- [.] Preserve and test initial-fill deterministic detail command IDs. This checkout
  has no initial-fill/catalog event flow; awaiting its branch or commit.


GOAL:
Implement the split of Anker Nordics into Anker Sweden, Norway, Finland, Iceland
and Denmark, covering `RetailerCode` and the existing Anker integration.

- [x] Import `bodul_shared` 0.1.1 into `lib/shared` ([plan](docs/localize-shared-plan.md)).
- [x] Migrate all monorepo consumers to local `shared` and validate.
- [x] Write the implementation report and improvement review ([report](docs/localize-shared-report.md)).

- [x] Plan the Nordic retailer split and its validation.
- [x] Implement and validate the Nordic retailer split ([report](docs/nordic-retailer-split-report.md)).

Implementation plan:

1. Establish the five storefront boundaries from actual sitemap/product
   sources, following `docs/retailer-integration.md`. Record each root sitemap,
   country URL scope, canonical URL behavior, language and currency evidence.
   The current integration uses `https://www.ankernordics.com/sitemap.xml`
   and classifies known sitemap sources without a country guard. Do not assume
   that adding `/se`, `/no`, `/fi`, `/is` or `/dk` produces real storefront URLs.
   The expected `../data/ankernordicscom` checkout is absent in this environment;
   obtain representative sources for all five countries before implementation.
2. Extend the local `RetailerCode` and lookup tables supplied by
   `refactor/localize-shared-library`, preserving its MIT notice and existing
   health/image APIs. Country names follow the existing convention:

   | Storefront | Enum variant | Slug/module |
   | ---------- | ------------ | ----------- |
   | Anker Sweden | `AnkerSe` | `ankerse` |
   | Anker Norway | `AnkerNo` | `ankerno` |
   | Anker Finland | `AnkerFi` | `ankerfi` |
   | Anker Iceland | `AnkerIs` | `ankeris` |
   | Anker Denmark | `AnkerDk` | `ankerdk` |

   Update enum serialization, slug parsing/formatting, `code_for_name`, and
   `RETAILERS`, with round-trip and enumeration tests.
3. Define the legacy-code transition before removing `AnkerNordicsCom`.
   Compatibility policy: retain decoding of historical Nordic records, replace the
   Nordic entry with the five new entries in active processing lists, and
   preserve old event/export payloads. Inventory queued commands and persisted
   retailer/URL identities before switching producers. Determine whether
   existing offers require migration from country evidence or a fresh crawl;
   ambiguous records must not be assigned to an arbitrary country or copied
   into all five. Record the agreed compatibility decision in a new ADR.
4. Reuse the completed local `shared` migration from the base branch. Verify
   all three workspaces resolve that crate without a published `bodul_shared`
   dependency. Deploy compatible readers before producing the new codes.
5. Split `lib/retailers_anker/src/ankernordicscom` into five country modules
   with the standard versioned sitemap/offer-detail/prelude layout. Reuse
   shared parsing where actual source schemas match; keep country routing
   explicit. Wire all seven hooks through `src/registry.rs` and register the
   modules in `src/lib.rs`. Guard the shared Nordic origin and paths before
   sitemap classification. Keep market isolation in API country context and
   response validation, preserving variant URLs, currency and localization.
6. Update the extractor facade tests and
   `lib/retailer_data_extractor/tests/normalization_report.rs`, data-directory
   selection and any active retailer lists found during implementation.
   Add portable country fixtures and corpus coverage for extraction through
   structuralization. Verify all five accept shared Nordic catalog URLs, reject
   foreign origins/invented country paths, and reject other-market responses.
   Verify new-code and
   historical-code handling through service serialization and offer imports.
7. Update the Anker README, retailer notes and coverage documents to describe
   the five storefronts and transition policy. Preserve historical coverage
   results as historical evidence; do not relabel Nordic samples as five
   independently verified country corpora.
8. Validate implementation with targeted registry, routing, parsing and
   compatibility tests, followed by `make fmt` and `make ci` in affected
   workspaces. Run `make audit` for changed dependency manifests/lockfiles and
   report new advisories. Run the real-data offer and sitemap pipelines for
   each country; require nonempty country-scoped products and explain unknown
   links. For changed services/apps, run smoke tests and
   `PORT=<port> make serve`, then check the health endpoint/page. Run database
   tests if persistence or migration behavior changes.

Implementation findings and authorization:
- Country-specific fetching/pricing was explicitly requested. The later user
  instruction moves the shared definitions into `lib/shared`, replacing the
  earlier external-release plan. No publishing or data migration is needed.
- All five countries share the Nordic sitemap and headless URLs. Use explicit
  Storefront API country context; country prefixes and HTML cookies do not
  supply country-specific source prices. Sweden remains the existing Nordic
  market rather than switching to the separate `anker.com/se` catalog.
- The public browser Storefront token is required for the headless catalog;
  anonymous API responses omit products. Configure
  `ANKER_NORDICS_STOREFRONT_TOKEN` in sourcing.
- Saved Nordic sources exist under `../data2` and `../data3`. Portable live
  API fixtures for all five markets now cover a product in the live sitemap.
- All consumers now use local `shared` definitions copied from the supplied
  Cargo package with its MIT notice. The temporary source override is removed.
  ADR 48 records local ownership and supersedes the external-release step.
- Rebased on `refactor/localize-shared-library`; ADR 46 and the existing shared
  provenance remain intact. Nordic market and rollout decisions are ADRs 47–48.


Status follows the [TODO workflow](AGENTS.md#todo-workflow):

| Marker | Phase                      |
| ------ | -------------------------- |
| `[ ]`  | Pending                    |
| `[.]`  | Planning in progress       |
| `[>]`  | Planning complete          |
| `[~]`  | Implementation in progress |
| `[x]`  | Implemented                |

Flow: `[ ]` → `[.]` → `[>]` → `[~]` → `[x]`.

RETAILER GRAPHQL ISOLATION:
- [x] Give all 48 configured retailers independent queries, schemas, raw types,
  token policy, market parsing, catalog traversal and detail pagination.
- [x] Replace service Shopify dispatch with neutral integration/HTTP contracts.
- [x] Separate the five legacy Nordic GraphQL queries, raw models and parsers;
  preserve extracted/normalized formats and compatibility routing.
- [x] Add isolation/compatibility coverage, ADR and run affected checks.
  CI passes for Anker (907 tests), Minisforum (658), Ugreen (1,186), shared
  sauce (19) and the extractor facade (20), including 53 distinct raw product
  types and local query ownership. Service initial-fill (24), route guards,
  health and OpenAPI checks pass; development app compilation passes.
  Full service CI and live startup are limited by sandbox socket restrictions;
  database checks require `DATABASE_URL_TEST`. Cached dependency audits show
  no new advisories; advisory refresh is blocked by the read-only cache.
  Saved-corpus reruns became unavailable when the external data directory was
  removed; checked-in fixture tests pass. See ADRs 61 and 62.

PUBLIC STOREFRONT TOKEN ACQUISITION:
- [x] Discover optional public tokens for the 33 previously token-free retailers,
  carry them through market discovery/catalog/details, and preserve public access
  when no token is exposed. Keep existing required-token policy and page sizes.
  Nine sanitized storefront fixtures exercise discovery; 29 initial-fill tests
  and shared-library/facade CI pass. Live token probes could not resolve DNS;
  full service CI and startup hit sandbox socket restrictions. PostgreSQL tests
  require DATABASE_URL_TEST. New fills resolve tokens; old queued work is unchanged.

MINISFORUM AND UGREEN GRAPHQL:
- [x] Share Shopify operations and raw representations while preserving Anker interfaces.
  Shared crate CI passes (29 tests), Anker CI passes (596 tests), and app/service
  compile checks pass. Market discovery helpers are implemented; durable wiring
  remains below. Dependency audit is blocked by the read-only advisory-cache lock.
- [x] Register all 31 storefronts and discover every advertised country/language context.
- [x] Carry contexts through durable commands, pagination, artifact storage and v2 dumping.
- [x] Add retailer raw fixtures, Bruno requests, documentation and integration verification.
  CI passes for Minisforum (470 tests), Ugreen (789), Anker (597), shared sauce
  and the extractor facade. All 40,406 saved Minisforum and 78,875 Ugreen product
  snapshots deserialize. Service initial-fill tests pass (26), as do source-route,
  health/OpenAPI and offline app checks. Full app/service CI and live startup
  checks hit sandbox socket restrictions. PostgreSQL tests require DATABASE_URL_TEST.
  Live probes for all 31 endpoints were attempted but DNS resolution failed;
  current live access remains unverified. No production fills were queued.

INITIAL OFFER FILL — REMAINING ANKER RETAILERS:
- [x] Define offer details v2 raw representations for Anker AU and CA.
  Survey saved GraphQL products and response pages, model the source fields with
  typed Serde structures, and test representative fixtures and the full corpus.
  Leave extracted representations and extraction behavior unchanged.
  Anker CI passes (596 tests). Raw tests pass with arbitrary-precision JSON,
  including 2,304 saved products and 784 full AU/CA response pages.
- [x] Add development dumping for GraphQL offer details v2.
  Plan: list initial-fill storage references through an authenticated development
  endpoint; dump the newest artifact per retailer/product into
  `offer_details_v2/sources/`. Keep the complete source JSON for raw deserialization.
  App/service formatting and Clippy pass, as do offline path/page checks and
  route authentication, pagination bounds, health and OpenAPI smoke tests.
  Full app/service CI and live startup checks hit sandbox socket restrictions;
  database-backed pagination coverage requires `DATABASE_URL_TEST`.
- [x] Apply verified operation-specific initial-fill page sizes to every retailer.
  Plan: initial lists of 5, image continuations of 25, variant continuations of 10;
  retain other continuation sizes, verify all markets and document ADR 56.
  Anker CI (590 tests) and 23 initial-fill tests pass. The final query documents
  pass live IT/JP probes at cost 919 (initial) and 814 (variant continuation).
- [x] Remove authentication-dependent initial-fill page sizes.
  Plan: use the existing query page sizes for every retailer, update the
  cross-market regression, and supersede ADR 53's one-item tokenless policy.
  Anker CI (590 tests) and 23 initial-fill tests pass. Live IT/JP probes reject
  standard OfferDetails (cost 1525) and 20-variant continuation (1017) against
  a 1000 limit. Initial size 5 (919), images 25 (26), and variants 10 (814)
  succeed; the operation-specific follow-up is tracked separately above.
- [x] Discover Nordic initial-fill tokens automatically when the environment override is absent.
  Plan: resolve the Nordic frontend's explicit storefrontAccessToken import for
  each market, preserve the environment override, and record the policy in ADR 54.
  Captured-bundle regression and live token/market probes pass for all five countries.
- [x] Fix regional Anker token acquisition for FR, PL, NZ, VN, EU and US.
  Plan: resolve the regional browser bundle's imported token-map constant from
  its explicit storeTokens reference; retain strict market selection and conflict
  rejection. Cover the downloaded regional bundle and all six acquisition paths.
  Live token/market probes pass for all six retailers. Sauce, Anker and extractor
  CI pass, as do 23 initial-fill tests. Full service CI and health smoke checks
  hit sandbox socket restrictions; PostgreSQL tests need `DATABASE_URL_TEST`.
- [x] Upgrade initial-fill requests to accept multiple retailers and include all supported Anker retailers in Bruno.
  Plan: replace the singular API with a validated, deduplicated retailer list;
  return ordered per-retailer runs and confirmed successes on dispatch failure.
  Update Bruno, OpenAPI, documentation, and offline/PostgreSQL coverage.
  Formatting, Clippy, and 22 initial-fill tests pass; Bruno covers all 17 codes.
  Full CI and the live health check hit sandbox socket restrictions;
  PostgreSQL verification requires `DATABASE_URL_TEST`.
- [x] Configure all 17 active Anker markets except Korea with evidence-backed endpoints, local languages, authentication and full pagination.
  Anker (590 tests), shared library and extractor CI pass; 171 socket-free service
  tests pass. Full workspace/service CI and smoke checks hit sandbox socket restrictions;
  PostgreSQL tests require `DATABASE_URL_TEST`. Live retailer verification remains pending.

  Plan: use captured data3 endpoint/localization evidence; add all 17 active Anker
  configurations, local language variables and explicit tokenless paths for Italy and Japan;
  reuse the Nordic token environment setting and browser tokens elsewhere.
  Paginate smaller tokenless detail pages, retain the full field selection, test every market and
  preserve AU/CA payload compatibility. Historical Nordic aggregate remains inactive.

INITIAL OFFER FILL — ANKER CA (ENGLISH ONLY):
- [x] Configure Canadian token acquisition, English GraphQL catalog/detail pagination, CAD validation, tests and documentation.
  Anker/facade CI and offline initial-fill tests pass. PostgreSQL/live checks
  require a configured test database and a runner with network access.

  Plan: reuse typed Shopify operations with a country variable and fixed EN; configure
  `/ca`, `storeTokens.ca`, `anker-canada.myshopify.com`, CA/CAD; validate every
  continuation, currency and event chain; add offline and opt-in live checks.

INITIAL OFFER FILL — ANKER AU:
- [x] Preserve sanitized HTTP causes, route and request IDs in logs and command errors for all retailer fetches; add failure-reporting and persistence tests. Socket checks require an unrestricted runner; persistence checks require `DATABASE_URL_TEST`.
- [x] Store initial-fill GraphQL artifacts in the existing `offer_details_raw` bucket accepted by object storage.
- [x] Serialize retailer codes as lowercase while reading existing PascalCase payloads; preserve replay command identities and verify consumers.
  Shared and service compatibility tests pass; see ADR 49 for rollout. Full-suite
  verification is limited by sandbox sockets and numeric-decoding failures when
  retailer crates are tested with combined dependency features.
- [x] Capture the actual AU storefront inputs and assert the expected Shopify token.
  The user's HTML and 42 captured scripts reproduce the real acquisition flow.
  Support `storeTokens: JSON.parse('...')` from `assets/16.js`; assert the exact
  AU token in the acquired event and subsequent catalog command. The HTML-only
  regression rejects unrelated credentials. Live Shopify acceptance remains unverified.
- [x] Implement AU storefront configuration, public token extraction, and typed Shopify operations.
- [x] Implement catalog identifier batches and complete product pagination with object storage.
- [x] Wire replay-safe reactions, REST entrypoint, and terminal events.
- [~] Verify the pipeline and document operation, dependencies, and architecture.
  Documentation and offline checks are complete. PostgreSQL/live verification
  needs `DATABASE_URL_TEST` and `OBJECT_STORAGE_URL`/`OBJECT_STORAGE_TOKEN`;
  this sandbox blocks sockets, including existing service tests and the health
  smoke check. Dependency audit against the cached advisory database found no
  new advisories; refreshing that database is blocked here.

NOTES:
- We are aware that `For now` is going against ADR-16, but this is shortterm
- depending on local mulac will be needed in this phase and possibly next ones
- `customer-facing` will take given events and persist them into `offers` table


BACKLOG: leave this alone
- Determine item condifition from title (refusrbished ...)
- Refactor how offer duplication is handled: use gtin or sku per retailer?
- Refactor commanding by introducing priority column, first command starts with priority 100 and every after it does -1 on current priority. If we reach 10, we stop with -1 and use last one. That way we fast track first batch to finish point. When reserving commands, it should also order by priority asc
- Drop descripton column on `customer_facing`'s 'offer' table
