# shared

Code shared across the monorepo.

`shared::health()` returns the liveness response (`"ok"`) that every service
serves at `/api/health` and through the GraphQL `health` query. It reports that
the process is running; it doesn't check dependencies such as the database.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.

`shared::images::is_valid` checks exact absolute HTTP(S) image URLs without
credentials, padding or controls, up to 2048 bytes. `images::filter` preserves
source order, removes exact duplicates, and returns the number of invalid URLs
skipped. It does not contact image servers or rewrite URLs.
