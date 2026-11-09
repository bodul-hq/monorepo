# Localize `bodul_shared` and document the implementation

## Deliverables

Save this plan before implementation. Deliver the library import and consumer
migration in two commits, then validate the result and commit a separate
`docs/localize-shared-report.md` covering the process and improvements.

## Commit 1: Import the library

Subject: `Import bodul_shared into local shared library`

- Import the resolved `bodul_shared` 0.1.1 source into existing `lib/shared`:
  retailer and link modules, tests, `SitemapConfig`, `NormalizedData`, and prelude.
- Preserve existing health/image helpers and imported behavior, including
  serialization, errors, and existing label spellings.
- Preserve the MIT notice and document source provenance.
- Add required workspace dependencies; omit unused upstream dependencies.
- Update READMEs, add a short ownership ADR, and refresh affected lockfiles.
- Keep consumers on the published dependency for this commit.
- Format and run `make ci` for `lib/shared` before committing.

## Commit 2: Migrate consumers

Subject: `Use local shared library across all workspaces`

- Replace `bodul_shared::` with `shared::` in source, tests, binaries, and current
  usage documentation throughout this monorepo.
- Use local `shared` workspace dependencies across `lib`, `services`, and `apps`.
- Refresh all three lockfiles without unrelated upgrades.
- Verify every `shared` dependency resolves locally and no dependency graph
  retains `bodul_shared`; preserve unrelated registry dependencies/configuration.
- Format and commit the migration.

Track import and migration separately in `TODO.md`, preserving the backlog.
Check branch/status before each commit; use explanatory bodies without signatures.

## Validation

After migration, run root `make ci` and `make audit`, distinguishing existing
advisories from new ones. Run all four services' PostgreSQL test suites against
dedicated test databases with required extensions. Start all four services and
both apps on distinct ports with `PORT=<port> make serve` and check authenticated
health endpoints or app pages as appropriate.

Confirm existing serialization, event, extraction, sitemap, health, and image
tests pass. Fix migration regressions and rerun affected checks. Record any
environmental blockers explicitly instead of claiming skipped checks passed.

## Implementation report

After validation, document delivered changes and implementation commit hashes,
the actual sequence and deviations, commands/results/advisories/limitations,
problems and resolutions, and concrete improvements supported by the work.
Commit the report as `Document shared library migration results`.

## Assumptions

Scope is this monorepo only. The crate remains `shared`. No HTTP contract changes,
database migrations, or upstream repository edits are intended. The source is
the exact currently resolved 0.1.1 package from `null-labs-hq`, not a newer release.
