# Sync retailer crates to bodul_hq

Status: `[x]` implemented on 2026-11-17; push, PR and publish are pending the user's go-ahead.

## Goal and boundaries

Bring `lib/retailers_anker`, `lib/retailers_minisforum` and `lib/retailers_ugreen`
into the standalone repositories at
`~/work/space/bodul_hq/lib_retailers_{anker,minisforum,ugreen}`. Each must still
build, pass `make ci` and publish to the `bodul` registry on its own. The source
is monorepo commit `1b6ba6af` on `chore/use-bodul-registry`. If the branch moves
before implementation, record the new SHA.

Out of scope: `lib_retailers_mi`, switching the monorepo to the registry crates,
and changes to the monorepo crates themselves.

## Current state

- The targets are separate git repositories (`bodul-hq/lib_retailers_*`). They
  were split from monorepo history at "Add Minisforum and Ugreen initial fills"
  (monorepo `22813172`, 2026-11-10). On top of that they have standalone-only
  commits: pinned dependencies plus `.cargo/config.toml` registries, an inlined
  Makefile with `publish`, CI, `.idea/` ignore, and "Release 0.1.1". Version
  0.1.1 of all three is published.
- Since the split, only two `wip` commits in the monorepo (`689ad4e6`,
  `1b6ba6af`) touch these crates, but the changes are large. 240, 173 and 316
  `src` entries differ. Each crate also gained a `tests/` folder of 8.1 MB,
  3.8 MB and 11 MB (offline fixtures, plus HTML corpus tests that are ignored
  by default). Replaying that history adds nothing, so a snapshot copy is enough.
- `lib_retailers_anker` is checked out on `dev`, one commit ahead of `main`.
  That commit, `eef3c69 wip`, adds ankerau offer_details v2, and the monorepo
  version supersedes it. The other two repositories are on `main`. All three
  are clean.
- The monorepo crates now need `retailer_sauce` 0.3.0 and `money` 0.1.2, both
  published. All three now use `chrono`, and the workspace sets `scraper` to
  `default-features = false`.

## Monorepo couplings to resolve

| Coupling                                                                    | Handling                                                                                                                                         |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `name.workspace = true` dependencies                                        | Pin the versions from `lib/Cargo.toml` `[workspace.dependencies]`, with registries.                                                               |
| Makefile includes `../../make/rust.mk`                                      | Keep the target's standalone Makefile unchanged.                                                                                                 |
| README links to `../../docs/…`, `../../services/…`, `../../README.md`       | They already dangle in 0.1.1. See decision 2.                                                                                                     |
| Ignored corpus tests read `CARGO_MANIFEST_DIR/../../../data/…`              | The data folder is missing in both locations. Keep the tests: they are `#[ignore]` and fail only when selected, as the monorepo documents.        |
| Rustdoc plain links to `../../../../docs/retailer.md`                       | rustdoc doesn't check them. Leave them.                                                                                                          |
| `#[cfg(test)] #[path = "../../../../tests/html/…"]` in anker `src`          | Works inside the crate. It is test-only, so excluding `tests/` from the package is safe.                                                         |
| `retailers_anker/.idea/`                                                    | Don't copy it (already ignored in the target).                                                                                                   |
| Toolchain pinned to 1.98.0 in the monorepo; target CI uses `stable`         | No change needed. Confirm `make ci` with the local 1.98.0.                                                                                       |

## Rules for the implementer

- Source crates: `/home/nulllabs/work/bodul/monorepo/lib/retailers_<x>`.
  Targets: `/home/nulllabs/work/space/bodul_hq/lib_retailers_<x>`, where `<x>`
  is `anker`, `minisforum` or `ugreen`. Read-only: the monorepo crates and
  `/home/nulllabs/work/space/bodul_hq/crates`.
- Commit rules from the monorepo `AGENTS.md` apply to the target repositories
  too:
  - One change per commit, with a short subject and a body that explains why.
  - Never add `Co-Authored-By` or other signature lines, even though earlier
    target commits have them.
  - Check `git status` and the current branch before each commit.
- Do not push, open PRs, tag or publish. Stop after step 9 and report.
- Track progress in the monorepo: set this plan's status and its `TODO.md`
  entry to `[~]` when you start and `[x]` when done, and fill in the
  verification record below.
- If a step fails in a way the plan doesn't cover (for example, the code
  doesn't compile against the published `retailer_sauce` 0.3.0), stop and
  report. Don't patch the copied source.

## Implementation (per repository)

1. **Branch.** Check `git status` and the current branch first. Anker: continue
   on `dev`. Its `eef3c69 wip` commit stays in history and is overwritten by
   the copy. Minisforum and Ugreen have no `dev`: branch
   `sync-monorepo-1b6ba6af` from `main`.
2. **Copy.** `rsync -a --delete <source>/src/ <target>/src/`, the same for
   `tests/`, and copy `README.md`. Do not touch `Cargo.toml`,
   `Makefile`, `.cargo/`, `.github/`, `.gitignore`, `Cargo.lock`, `target/` or
   `.idea/`.
3. **Manifest.** Rewrite the dependencies to match the monorepo workspace,
   sorted:
   ```toml
   chrono = { version = "0.4.45", default-features = false, features = ["std", "serde"] }
   graphql_client = "0.16.0"
   html-extractor = { version = "0.3.3-dev.2", registry = "null-labs-hq" }
   money = { version = "0.1.2", registry = "bodul" }
   retailer_sauce = { version = "0.3.0", registry = "bodul" }
   scraper = { version = "0.27.0", default-features = false }
   serde = { version = "1.0.229", features = ["derive"] }
   serde_json = "1.0.154"
   shared = { version = "0.1.1", registry = "bodul" }
   thiserror = "2.0.20"
   url = "2.5.8"
   ```
   Keep the `preserve_order` dev-dependency and its comment.
4. **Lockfile.** Run `cargo check` to update the existing `Cargo.lock`
   incrementally. Don't regenerate it. Then compare resolved versions with
   `/home/nulllabs/work/bodul/monorepo/lib/Cargo.lock` and align any differences with
   `cargo update -p <crate> --precise <version>`. That way the standalone
   crate is tested against the same dependency graph as the monorepo.
5. **Verify.** `make fmt`, then `make ci`. Expect the offline fixture tests and
   unit tests to run, and the corpus tests to be ignored. Run `cargo audit`,
   because the lockfile changed, and report any new advisories.
6. **Commit** the copy together with the manifest and lockfile. They only build
   together. Subject: "Sync crate from monorepo 1b6ba6af". The body explains
   the move to retailer_sauce 0.3 and the new offline tests.
7. **README links:** rewrite every relative link that leaves the crate
   (`../../docs/…`, `../../services/…`, `../../README.md`) as an absolute
   `https://github.com/nl-miro/monorepo/blob/main/<path>` link, keeping anchors.
   Leave in-crate links such as `tests/fixtures/README.md` relative. Afterwards,
   `grep -n '\.\./' README.md tests/fixtures/README.md` must find no
   out-of-crate links. Commit separately.
8. **Package size:** add `exclude = ["tests/"]` under `[package]` and commit
   separately. Check with `cargo package --list` that no `tests/` paths remain,
   and with `cargo package` that the package still builds and verifies (the
   `#[path]` test modules are `#[cfg(test)]` only). Don't use
   `make publish-dry-run`: it refuses branches that aren't pushed and in sync
   with their upstream. Record the `.crate` size from `target/package/`.
9. **Release:** bump `version` to `0.2.0`, refresh the lockfile, and commit as
   "Release 0.2.0", following the earlier "Release 0.1.1" commits.
   Run `make ci` once more afterwards.
10. **Not part of this handoff:** push, PR and `make publish` need an explicit
    go-ahead from the user. `make publish-dry-run` belongs here too, because it
    needs the pushed branch. The three crates don't depend on each other, so the
    publish order doesn't matter.

## Decisions

Confirmed on 2026-11-17:

1. **Anker branch:** continue on `dev`.
2. **README links:** update them to point at the monorepo on GitHub. They
   target `main`, so documents that exist only on `chore/use-bodul-registry`
   (for example ADRs 80–86) resolve once that branch is merged. The monorepo is
   private, so the links work only for readers with access.
3. **Package contents:** exclude `tests/` from the published crate.
4. **Version:** release as `0.2.0`, since the public types now come from
   `retailer_sauce` 0.3 and the change breaks the API.

## Verification record

Source: monorepo `1b6ba6af` (the last commit touching the three crates; the
branch HEAD was `5a0fe4b8`).

| Crate      | Branch                   | Commits (copy, README, exclude, release)  | `make ci`                         | `.crate` size |
| ---------- | ------------------------ | ----------------------------------------- | --------------------------------- | ------------- |
| anker      | `dev`                    | 961fd43, 11a4e1e, 8ea11c0, 07f3713       | pass: 565 + 17 tests, 17 ignored  | 626,588 B     |
| minisforum | `sync-monorepo-1b6ba6af` | 4ab1e79, bd8a475, d8fdcdf, 8e63546       | pass: 472 + 9 tests, 28 ignored   | 234,338 B     |
| ugreen     | `sync-monorepo-1b6ba6af` | 08db2ea, 1117632, c615b7a, 91df8d0       | pass: 759 + 21 tests, 53 ignored  | 449,246 B     |

- Lockfiles: after `cargo check`, every resolved version already matched the
  monorepo `Cargo.lock`; no `cargo update --precise` was needed.
- `cargo audit`: no vulnerabilities. Four allowed warnings, the same in all
  three: `chrono`, `fxhash` and `kuchiki` unmaintained, `rand` unsound
  (RUSTSEC-2026-0097).
- `cargo package`: no `tests/` paths, and the package builds and verifies. Sizes
  were measured at version 0.1.1, before the bump.
- The README grep finds one hit, `../data/ankeritalycom/...` in anker. It is
  code text about the missing corpus data, not a link. The fixtures README
  `../offer_details_v2.rs` reference is in-crate.
- Nothing was pushed, tagged or published.
