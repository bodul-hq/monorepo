# Nordic retailer split: implementation and verification

Date: 2026-11-09

## Delivered behavior

`refactor/split-ankernordics` is rebased onto `refactor/localize-shared-library`
at `c45230b`. The original Nordic work was `c35b1c5`; its pre-rebase state is
retained locally as `backup/split-ankernordics-before-localize-rebase-20261109`.

Five active codes replace the combined Nordic entry: `ankerse`, `ankerno`,
`ankerfi`, `ankeris`, and `ankerdk`. All share the existing Nordic catalog URLs
and sitemap. Downloads select country context in the Storefront API and reject
wrong-market, wrong-product, partial, or error responses before storage. The
legacy `ankernordicscom` code still decodes historical records and queued work;
this change does not reassign historical data.

The implementation includes the country modules and registry hooks, direct JSON
extraction, exact decimal prices, source-backed localization, and NOK/DKK/ISK
money and display support. Non-Nordic downloads retain GET behavior. Nordic
downloads require `ANKER_NORDICS_STOREFRONT_TOKEN`, now documented in both the
sourcing README and `.env.example`.

## Rebase and documentation finalization

Both branches independently migrated the published shared types. The rebase
resolved 185 conflicted files by retaining the base's local dependency wiring
and import conventions while preserving the Nordic implementation and tests.
An automatically merged duplicate `RetailerCode` import was also removed.

The base's license, source checksum, saved plan, and implementation report remain
intact. The duplicate Nordic copy of the same MIT notice was removed and its
references now point to `lib/shared/LICENSE-MIT`. Shared documentation was
consolidated without removing the Nordic compatibility policy.

ADR 46 remains the base's local-library decision. The Nordic decisions were
renumbered to [ADR 47](adr/0047-nordic-retailer-markets.md) and
[ADR 48](adr/0048-local-retailer-shared-types.md), preserving their decisions and
the latter's supersession of the former's external-publication requirement.
Links and TODO assumptions now reflect shared URLs with API market isolation.
The integration playbook describes the JSON-source exception and no longer
claims that the now-passing Anker sitemap tests are known failures.

## Verification

All checks below ran against the rebased implementation:

| Check | Result |
| --- | --- |
| Root `make fmt`, followed by root `make ci` | Passed across lib, services, and apps; includes 582 Anker tests. |
| Retailer-sourcing `make test-postgres` | 70 passed. |
| Offers `make test-postgres` | 38 passed. |
| Customer-facing `make test-postgres` | 45 passed. |
| Locked offline Cargo metadata, host platform | Every workspace resolves exactly one local `shared`; none contains `bodul_shared`. |
| Lockfile package comparison with `c45230b` | No package versions, registry sources, or checksums changed. |
| Service/app startup | Sourcing, offers, customer-facing health and both app index pages returned HTTP 200. Verification processes were stopped afterward. |

PostgreSQL runs used `RUST_TEST_THREADS=4` and dedicated local test databases on
port 55432. Sourcing and offers used `anker_split_test`. The first customer-facing
attempt used that database too, but it lacked the required `pg_trgm` extension.
Rerunning against the already provisioned `localize_shared_test` passed all 45
tests. This was test setup, not an application regression.

The existing saved source corpus at `/tmp/anker-split-corpus` was exercised with:

```sh
make process-offer-details DATA_DIR=/tmp/anker-split-corpus RETAILERS='ankerse ankerno ankerfi ankeris ankerdk'
make process-sitemap-sources DATA_DIR=/tmp/anker-split-corpus
make consolidate-sitemap-sources DATA_DIR=/tmp/anker-split-corpus
```

Each market's independently fetched API fixture passed all three offer stages.
Each sitemap run consumed six files, produced 870 URLs, and left zero files
unused. Consolidation produced 492 products, 114 catalogs, 164 content links,
94 not-interested links, and zero unknown links per market. Sharing that sitemap
is intentional; it does not imply that one market's price is reused for another.

The saved product `t90r0121` retained these exact prices and English localization:

| Market | Currency | Minor-unit amount |
| --- | --- | ---: |
| Sweden | SEK | 49900 |
| Norway | NOK | 49900 |
| Finland | EUR | 4499 |
| Iceland | ISK | 6490 |
| Denmark | DKK | 34900 |

These are saved-source regression results, not a fresh live-price survey or
coverage of every product in every market. Historical Nordic HTML coverage was
not relabeled as country-specific evidence. A production crawl and inventory of
historical Nordic queues remain deployment activities.

## Audit and rollout

`make audit` ran separately in all three workspaces. It reports the same existing
vulnerabilities as the shared-library base: 2 in lib, 9 in services, and 2 in apps.
No new advisories or dependency upgrades were introduced. Existing warning and
advisory details are recorded in the
[shared-library report](localize-shared-report.md#dependency-audit).

Deploy compatible readers and currency support before enabling new Nordic
producers. Supply the public browser Storefront token to sourcing; do not use an
Admin API token. Missing configuration explicitly fails Nordic downloads, and
there is no fallback to default-market prices. No publication, database migration,
or historical offer reassignment is required for this code change.

## Process improvements

- Base dependent work on the shared-library branch before implementing consumers;
  this avoids duplicated migration edits and most of the rebase conflicts.
- Check semantic invariants after automatic merges, especially duplicate imports,
  active versus historical retailer lists, and shared dependency resolution.
- Verify database prerequisites before starting suites. The missing extension
  caused an avoidable failed run despite a suitable test database being available.
- Keep corpus evidence precise: shared discovery URLs and separate market API
  responses prove different properties and should be reported separately.
- Check ADR numbering and documentation claims during rebases, not just Rust
  compilation. The numbering collision and stale test-failure notes survived
  otherwise mechanical file merges.
