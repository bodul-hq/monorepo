# Issue 25 implementation and verification

Implemented independently from upstream/main `567bac8552c7dca5a6ade65f864e78cd08382279`.

Customer-facing now stores selected language and localized title/description/URL
entries in a private content envelope. Migration 2026-10-29-000011 validates and
backfills the envelope, removes four scalar columns, installs selected-URL
uniqueness and title/description trigram indexes, and refuses lossy rollback.
Projection replay retains receipt atomicity, locking and per-language URL rules.
The public array gains URLs while flat selected fields remain compatible.
Customers accepts historical and new responses; search uses selected language
only, eligibility in either field, and a 5:1 title/description score.

Validation on 2026-10-29:

- Customer-facing formatting, lint and CI pass: 43 tests passed and 41 database
  tests ignored. New unit cases cover selection, validation and URL reconciliation.
- Customers formatting and lint pass; 29 tests pass, including URL compatibility.
  Six existing HTTP listener tests fail because sockets are forbidden.
- PostgreSQL tests include physical dropped-column checks, reversible migration,
  downgrade refusal, constraints/uniqueness, selected-language weighted search,
  snapshot/legacy URL reconciliation and adapted historical replay fixtures.
  They compile but cannot run without DATABASE_URL_TEST. No local PostgreSQL
  server binary was available. EXPLAIN/representative-data performance checks
  remain unexecuted for the same reason.
- Live customer-facing startup requires DATABASE_URL; customers startup is
  blocked by socket permissions. Health/page/browser checks remain unverified.
- No dependency or lockfile changes; audit was not required. `git diff --check`
  passes. Deploy the compatible customers reader first and coordinate the
  customer-facing migration/binary switch as documented in ADR 41.
