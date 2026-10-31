
GOAL: create full flow from structuralize to show on web

[x] Replace regional market labels with canonical codes ([issue #24](https://github.com/bodul-hq/monorepo/issues/24)).

[~] Count of commands and events by status are not taking on ONLY records scheduled for today — all three dashboard services, UTC ([plan](docs/plans/33-today-entry-status-counts.md))
[x] Support inbox and outbox statistics, like for commands and events ([plan](docs/plans/34-inbox-outbox-statistics.md))


PR #23 selected review fixes (implementation plan supplied; [verification and blockers](docs/plans/pr23-selected-review-fixes-verification.md)):
[x] Freeze reporting time in tests (comment 1)
[x] Fix compact empty-box CSS (comment 2)
[x] Pass icon slugs explicitly (comment 3)
[x] Correct the development README (comment 4)
[x] Add date-range indexes (comment 6)
[x] Replace obsolete rendering assertions (comment 9)
[x] Update Bruno documentation (comment 11)



Status follows the [TODO workflow](AGENTS.md#todo-workflow):

| Marker | Phase |
| --- | --- |
| `[ ]` | Pending |
| `[.]` | Planning in progress |
| `[>]` | Planning complete |
| `[~]` | Implementation in progress |
| `[x]` | Implemented |

Flow: `[ ]` → `[.]` → `[>]` → `[~]` → `[x]`.

Items 24–27 are marked done at the user's request. Their implementation
and planned acceptance tests are present; database, object-storage and live
execution remain pending. The 2026-10-27
follow-up records passing portable checks and the environmental blockers in the
[verification record](docs/plans/24-27-offer-facts-verification.md#completion-follow-up-2026-10-27).

Items 28–31 are implemented. Their [verification record](docs/plans/28-31-verification.md)
lists passing portable checks and the PostgreSQL/live checks blocked by this
environment.

NOTES:
- We are aware that `For now` is going against ADR-16, but this is shortterm
- depending on local mulac will be needed in this phase and possibly next ones
- `customer-facing` will take given events and persist them into `offers` table



BACKLOG: leave this alone
- Determine item condifition from title (refusrbished ...)
- Refactor how offer duplication is handled: use gtin or sku per retailer?
- Refactor commanding by introducing priority column, first command starts with priority 100 and every after it does -1 on current priority. If we reach 10, we stop with -1 and use last one. That way we fast track first batch to finish point. When reserving commands, it should also order by priority asc
- Drop descripton column on `customer_facing`'s 'offer' table



NOTES

Codex without prompting every 7-33 seconds
```
codex --ask-for-approval never --sandbox workspace-write
```

Restore retailer sourcing db locally
```
PGPASSWORD='retailer_sourcing' pg_restore   --host=172.28.100.10   --port=5432   --username=retailer_sourcing   --dbname=retailer_sourcing_dev   --clean --if-exists --no-owner --single-transaction   'retailer-sourcing-2026-10-26-173835-t7CMR1.dump'
```


Restore object storage db locally
```
PGPASSWORD='object_storing' pg_restore   --host=172.28.100.31   --port=5432   --username=object_storing   --dbname=object_storing_dev   --clean --if-exists --no-owner --single-transaction   'object-storing-2026-10-26-173837-kMcsVD.dump'
```

[x] Move customer-facing offer fields into content and weight selected-language search ([issue #25](https://github.com/bodul-hq/monorepo/issues/25); [verification](docs/plans/36-content-storage-verification.md)).

## PR #27 review fixes

[>] Comment 1: Two-release compatibility rollout
[>] Comment 2: EU-only projection migrations (Release 2)
[>] Comment 4: Customers-first deployment
[~] Comment 5: Export version validation
[x] Comment 6: Explicit market decoder
[>] Comment 7: Canonical runtime boundaries
[x] Comment 8: Reuse export localization
[x] Comment 9: Exhaustive region string mapping
[~] Comment 10: GitHub Actions verification

Comment 3 is intentionally skipped. Release 2 activation requires all Release 1 readers deployed.

## After a full regional-data reset

[ ] Remove LegacyRegion, legacy label decoding/validation, EU-label canonicalization and legacy() helpers.
[ ] Remove temporary Release 1 legacy writer adapters.
[ ] Retire label-era Offers record variants and compatibility fixtures.
[ ] Retire pre-v9 export paths requiring legacy labels.
[ ] Remove legacy-region API/client fallback branches; retain nullable-field handling required by the current API.
[ ] Remove region backfill/downgrade logic only within a reviewed fresh migration baseline; retain schema creation.
[ ] Replace obsolete compatibility tests and rollout documentation with code-only contract coverage.

Gate cleanup on resetting or retiring event stores, queues, projections, stored source/export artifacts, replay inputs and old producers/clients. A database reset alone is insufficient if old payloads can be restored or redelivered. Unrelated compatibility remains out of scope.

## Broken CI tests

[x] Run smoke-test builds inside the checkout so Cargo loads the registry configuration.
