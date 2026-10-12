# Changelog

Notable changes to this repository are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Added

- Monorepo skeleton setup ([#3](https://github.com/bodul-hq/monorepo/pull/3)):
  - `lib`, `services` and `apps` Cargo workspaces, with shared dependency versions declared once per workspace.
  - Library crates `shared`, `money`, `retailer_sauce`, `retailer_data_extractor` and `retailers_*` (Anker, Mi, Minisforum, Ugreen, WIP), each with a README and a test.
  - `customer_facing` and `offers` services (poem, poem-openapi, async-graphql) and `retailer_sourcing` (axum, utoipa, async-graphql), serving REST under `/api`, OpenAPI at `/api/openapi.json`, Swagger UI at `/api/docs`, GraphQL over GET and POST at `/graphql`, and GraphiQL at `/graphql/ide` in debug builds.
  - `customers` and `development` apps that render HTML with poem.
  - Servers listen on a validated `PORT` (default 3000) and shut down gracefully on `SIGTERM` or Ctrl+C.
  - Smoke tests for every service and app, and tests for `PORT` parsing.
  - Make targets (`make ci`, `make audit`), a pinned Rust 1.98.0 toolchain and a GitHub Actions CI workflow.
  - ADRs 1–3, a README and AGENTS.md.

### Security

- quick-xml advisories RUSTSEC-2026-0194 and RUSTSEC-2026-0195 (from poem 3.1.12) are ignored in `services/` and `apps/` while no server parses XML request bodies. Clippy rejects the XML extractors to keep that true. See "Security audit" in the README.
