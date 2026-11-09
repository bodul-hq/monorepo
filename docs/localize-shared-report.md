# Local shared library implementation report

Date: 2026-11-09

## Outcome and commits

The monorepo now owns the previously published `bodul_shared` 0.1.1 code in
`lib/shared`. All 12 direct consumers use `shared`, and all three workspaces
resolve the same local crate. Existing health/image helpers, retailer slugs,
serialization, errors, and link labels are preserved. No database migration or
HTTP contract change was introduced.

The [saved plan](localize-shared-plan.md) was written before implementation.
Changes were committed in order on `refactor/localize-shared-library`:

| Commit | Change |
| --- | --- |
| `e2d7fc6` | Import the resolved upstream source, tests, MIT notice, provenance, documentation, and ADR 46. Consumers still use the published package at this commit. |
| `fd02fc3` | Migrate consumer imports and manifests across `lib`, `services`, and `apps`; refresh all three lockfiles. |
| `2df5c17` | Correct an Anker module-name collision discovered during full validation. |

There are three implementation commits rather than the planned two: the small
validation fix is recorded separately, preserving the two requested migration
steps and their history. This report and final TODO statuses form a separate
documentation commit.

## Implementation process

1. Inspected the repository, existing `shared` helpers, dependency declarations,
   lockfiles, and cached upstream source. Although the request referred to Margo,
   the resolved source was `null-labs-hq` version 0.1.1. The agreed scope was this
   monorepo, with the existing `shared` crate name retained.
2. Copied the retailer/link modules, six upstream tests, `SitemapConfig`,
   `NormalizedData`, and prelude exports. Added only the used `serde` and
   `thiserror` dependencies; unused upstream `serde_json` and `uuid` dependencies
   were not carried over. Preserved the MIT notice and package checksum.
3. Local Clippy exposed upstream warnings that dependency builds did not report.
   Preserved the public inherent `from_str` API with a documented, narrowly
   scoped lint allowance. Changed one test's infallible conversion from
   `try_into().unwrap()` to `into()` and repaired stale module documentation.
   The shared crate's eight tests passed before the first commit.
4. Replaced consumer imports and registry declarations, formatted the code, and
   refreshed lockfiles without unrelated upgrades. The migration commit changed
   199 files, mostly import paths and their formatting.
5. Full CI found that Anker already declares a private `mod shared`. Its root
   import needed `::shared::link::LinkKind` to select the external crate. Applied
   that single-line correction, committed it separately, and validated the
   Anker integration and downstream consumers.
6. Completed CI, database tests, dependency audits, and running-server checks.
   A user interruption stopped the original root CI run during retailer-sourcing
   compilation; completed checks were retained and remaining targets resumed.

## Validation results

| Check | Result |
| --- | --- |
| Formatting | Root `make fmt` completed; all subsequent CI formatting checks passed. |
| Import check | `make ci` in `lib/shared` passed, including 8 tests. |
| Library CI | Root `make ci` completed all 16 library crates after the collision fix. An additional Anker check passed all 577 tests. |
| Service CI | Root CI completed customer-facing and offers. Resumed `make -C services/retailer_sourcing ci` and `make -C services/object_storage ci` both passed. |
| App CI | `make -C apps ci` passed both apps and development utility binaries. |
| Customer-facing PostgreSQL | 45 passed. |
| Offers PostgreSQL | 38 passed. |
| Retailer-sourcing PostgreSQL | 70 passed. |
| Object-storage PostgreSQL | 10 passed. |
| Dependency resolution | Locked, offline Cargo metadata confirmed exactly one local `shared` and no `bodul_shared` in each workspace. |
| Lockfile comparison | Every other registry package's version, source, and checksum matched the pre-import lockfiles. |
| Diff hygiene | `git diff --check` passed. |

All constituent CI targets passed; the interrupted root invocation itself did
not return a successful final exit code. The 163 PostgreSQL tests were run with
each service's `make test-postgres`, `RUST_TEST_THREADS=4`, and
`DATABASE_URL_TEST=postgresql://postgres@127.0.0.1:55432/localize_shared_test`.
The database was newly created in the existing local test container, with
`pg_trgm` provisioned in `customer_facing_extensions`. Tests used their existing
isolated-schema setup. Existing development databases were not repurposed.

The metadata command used `--locked --offline --filter-platform
x86_64-unknown-linux-gnu --format-version 1` with each workspace manifest.
Unfiltered metadata initially requested an uncached dependency for another target;
filtering to the host avoided an unnecessary download. Runtime testing covers
Linux on this host, not other platforms.

Existing ignored live-service and external-corpus suites were not enabled as
part of ordinary CI. The four service PostgreSQL suites were explicitly enabled
and passed. Live retail crawling and a complete end-to-end import were not run.

## Running-server checks and cleanup

Each server was started through its `make serve` target with explicit `PORT` and
isolated database configuration where required. Make command-line assignments
passed these settings into the server environment.

| Server | Port | Request and result |
| --- | --- | --- |
| Object storage | 38460 | Authenticated `GET /api/health`: 200, `ok`. |
| Customer-facing | 38461 | `GET /api/health`: 200, `ok`. |
| Offers | 18462 | `GET /api/health`: 200, `ok`. |
| Retailer sourcing | 38463 | `GET /api/health`: 200, `ok`; command workers disabled. |
| Customers | 38464 | `GET /`: 200, rendered offers page backed by the test customer-facing service. |
| Development | 38465 | `GET /`: 200, rendered development page. |

Offers initially failed to bind port 38462 with `AddrInUse`; retrying on 18462
succeeded. No application changes were needed.

**Cleanup remains outstanding:** after these checks, the session policy rejected
Ctrl+C through the process-control tool because approval was required while the
approval policy was `Never`. The six test servers above were left running at
report time. Stop only these verification servers when process-control permission
is available. The task-created databases `localize_shared_test`,
`localize_shared_customer`, `localize_shared_offers`, `localize_shared_storage`,
and `localize_shared_sourcing` were also retained in the local test container.

## Dependency audit

Root `make audit` stopped on library advisories, so services and apps were audited
separately. **Audits did not pass**, but no advisory was introduced by this change:
the affected registry package versions and checksums were already present before
the import.

| Workspace | Vulnerabilities | Existing affected packages |
| --- | --- | --- |
| lib | 2 | `rustls-webpki` 0.103.15: RUSTSEC-2026-0382; `webpki-root-certs` 1.0.9: RUSTSEC-2026-0383. |
| services | 9 | `rustls` 0.23.45: RUSTSEC-2026-0346/0347/0348/0349; `rustls-webpki` 0.103.15: RUSTSEC-2026-0382; `zip` 8.6.0: RUSTSEC-2026-0367/0368/0369/0429. |
| apps | 2 | Same two vulnerability advisories as lib. |

The audit also reported existing warnings: unmaintained `fxhash` and `kuchiki`,
plus `rand` unsoundness in all workspaces; `tokio` unsoundness in services/apps;
and unmaintained `unsafe-libyaml` plus yanked `num-modular` in services. These
dependency upgrades remain separate follow-up work.

## What we could have done better

- **Check name collisions before mechanical import replacement.** Searching for
  `mod shared` during planning would have revealed the Anker collision and avoided
  the extra correction commit. Dependency graph validation alone cannot detect
  Rust name-resolution errors.
- **Compile migrated consumers before committing the migration.** The requested
  full validation after the second commit was followed, but a fast compile check
  before that commit would have caught the collision without changing that order.
- **Use the permitted Make invocations from the start.** Shell redirection and
  subprocess wrappers ran some attempts inside a restrictive sandbox. They failed
  on socket access, Git index writes, or advisory-cache writes. Direct permitted
  Make commands succeeded for the same code and test database, reducing uncertainty
  about whether failures belonged to the migration or the environment.
- **Run audits sequentially and retain per-workspace results.** Concurrent audits
  contended on the shared advisory database lock, and root audit stopped at the
  first workspace's existing findings. Sequential workspace audits would avoid
  that contention while still reporting every workspace.
- **Choose verification ports outside the ephemeral port range and plan cleanup.**
  The initial offers bind failed. Reserving distinct lower ports and confirming
  process-control permissions would reduce startup retries and prevent leftover
  servers after a permission-policy change.
- **Make interrupted validation easier to resume.** Recording completed targets,
  exit codes, and test totals in one durable checklist would reduce log inspection
  and repeat checks after an interruption. The final report distinguishes completed
  targets from the interrupted root process rather than claiming a root success.

The staged import/migration commits, preserved source provenance, and exact
lockfile comparison worked well: they kept ownership changes reviewable and
provided evidence that this migration introduced no unrelated dependency changes.
