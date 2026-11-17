# TODO

Status follows the [TODO workflow](AGENTS.md#todo-workflow):

| Marker | Phase                      |
| ------ | -------------------------- |
| `[ ]`  | Pending                    |
| `[.]`  | Planning in progress       |
| `[>]`  | Planning complete          |
| `[~]`  | Implementation in progress |
| `[x]`  | Implemented                |

Flow: `[ ]` → `[.]` → `[>]` → `[~]` → `[x]`.

- [x] Move external HTML corpus tests into retailer `tests/html/`, retaining test-only module wiring for private parser access. All 73 corpus checks and 25 stage writers are explicitly ignored; missing data fails when selected. JSON integration and self-contained unit tests remain enabled. `make ci` passes in all three brand crates.

- [x] Add offline integration tests with three raw/extracted fixture pairs per storefront from `data2` in the retailer crates. Copied 141 unchanged pairs across 47 storefronts into each brand's `tests/fixtures`, documented their origins, and verified full extraction and JSON round trips. `make ci` passes in all three brand crates.

- [x] Sync `retailers_anker`, `retailers_minisforum` and `retailers_ugreen` into the standalone `~/work/space/bodul_hq/lib_retailers_*` repositories, keeping them buildable and publishable to `bodul`. [Plan](docs/plans/retailer-crates-bodul-hq-sync.md).
