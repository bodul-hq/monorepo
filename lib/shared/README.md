# shared

Code shared across the monorepo.

`shared::health()` returns the liveness response (`"ok"`) that every service serves at `/api/health` and through the GraphQL `health` query. It reports that the process is running; it doesn't check dependencies such as the database.

This crate is part of the `lib` workspace. See the [root README](../../README.md) for requirements and Make targets.
