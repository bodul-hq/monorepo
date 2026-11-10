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

NOTES:
- We are aware that `For now` is going against ADR-16, but this is shortterm
- depending on local mulac will be needed in this phase and possibly next ones
- `customer-facing` will take given events and persist them into `offers` table


BACKLOG: leave this alone
- Determine item condifition from title (refusrbished ...)
- Refactor how offer duplication is handled: use gtin or sku per retailer?
- Refactor commanding by introducing priority column, first command starts with priority 100 and every after it does -1 on current priority. If we reach 10, we stop with -1 and use last one. That way we fast track first batch to finish point. When reserving commands, it should also order by priority asc
- Drop descripton column on `customer_facing`'s 'offer' table
