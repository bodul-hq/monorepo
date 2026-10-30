
GOAL: create full flow from structuralize to show on web

[~] Count of commands and events by status are not taking on ONLY records scheduled for today — all three dashboard services, UTC ([plan](docs/plans/33-today-entry-status-counts.md))
[x] Support inbox and outbox statistics, like for commands and events ([plan](docs/plans/34-inbox-outbox-statistics.md))


PR #23 selected review fixes (implementation plan supplied):
[>] Freeze reporting time in tests (comment 1)
[x] Fix compact empty-box CSS (comment 2)
[x] Pass icon slugs explicitly (comment 3)
[x] Correct the development README (comment 4)
[>] Add date-range indexes (comment 6)
[>] Replace obsolete rendering assertions (comment 9)
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
