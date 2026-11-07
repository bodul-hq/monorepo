
GOAL:



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
