# Changelog

Notable changes to this repository. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- `lib` Cargo workspace with `shared`, `money` and `retailer_sauce`, plus `retailer_data_extractor` and one crate per retailer: `retailers_anker`, `retailers_mi`, `retailers_minisforum`, `retailers_ugreen` and `retailers_wip`. Every library has a README, and crates without code have a placeholder test.
- `shared::health()`, the liveness response every service returns.
- Service routes in `customer_facing`, `offers` and `retailer_sourcing`: `/api/health`, `/api/openapi.json`, `/api/docs` (Swagger UI), `/graphql` over `GET` and `POST`, and `/graphql/ide` (GraphiQL) in debug builds only.
- Server-rendered index pages in the `customers` and `development` apps.
- `PORT` support in every service and app. It defaults to 3000, and an invalid value stops startup with an error.
- Graceful shutdown on Ctrl+C and SIGTERM in every service and app.
- Smoke tests for every route and for `PORT` parsing.
- Make targets shared through `make/rust.mk` and `make/workspace.mk`, available from the root, each workspace and each crate, including `ci`, `audit` and `serve`.
- Rust toolchain pinned to 1.98.0 in `rust-toolchain.toml`.
- ADRs 1 to 3 in `docs/adr`: web frameworks, workspace dependencies and port validation, and GraphQL over GET with debug-only GraphiQL.
- `cargo audit` exceptions for the quick-xml advisories RUSTSEC-2026-0194 and RUSTSEC-2026-0195, with clippy rules that forbid the Xml types the exceptions rely on staying unused.
- Root README describing the layout, requirements, Make targets and how to run servers.
- Sitemap discovery pipeline in `retailer_sourcing`, organised as three feature slices (`retrieve_sitemap`, `process_sitemap`, `consolidate_sitemap`) on the Mulac command and event kernel. It runs when `DATABASE_URL` is set and applies its own migrations on startup. ADR 4 records the structure.
- `retailer_sourcing` routes to queue sitemap retrievals (`POST /api/sitemap-retrievals`) and to process queued commands and events on demand (`POST /api/commands/run`, `POST /api/events/run`). Nothing processes the queues in the background.
- Offer discovery in `retailer_sourcing`, ported from the MVP as two feature slices: `discover_offers` records an offer for every product URL of a consolidated sitemap, and `download_offer_page` downloads each offer's page into `raw_offers`, skipping URLs downloaded in the last 6 hours.
- Logging in `retailer_sourcing` to stderr and daily files under `logs/`: each pipeline step, skipped sitemap files, failures, HTTP requests, and a separate timing log for fetches, commands and reactions. It replaces the `sitemap_retrievals` and `offers` status tables, which migrations drop; an offer is now just the `offer_id` that discovery hands out.
- `object_storage` service: immutable records in its own PostgreSQL database, created and retrieved over REST and GraphQL, with bearer authentication on every route. ADRs 5 and 6 record why it uses axum and how it starts and is accessed. A Bruno workspace lists the collections and adds one for `object_storage`.
- `object_storage_client`, an async client for the object storage service over REST or GraphQL.
- `Money` and `Currency` in `money`. `Money::parse` and `Money::from_major` are not implemented yet.
- `sitemap` library: parsing sitemap XML and building a crawl's sitemap tree.
- `Availability`, `ItemCondition`, validated identifiers and `StructuralizedData` in `retailer_sauce`.
- Sitemap filtering and pacing per retailer: crawls skip child sitemaps a retailer rejects, such as Shopify metaobject sitemaps, and pause after every three downloads, for five seconds on the Ugreen stores that rate-limit.
- Anker offer details: Next.js, App Router and Liquid storefronts are extracted, normalized and structuralized into offers, with tests over saved product pages.
- Minisforum offer details: storefront product islands are extracted, normalized and structuralized into offers, with tests over saved product pages.
- Ugreen offer details, NAS storefronts included: extracted, normalized and structuralized into offers, with tests over saved product pages.
- `retailer_data_extractor::extract_for` and `structuralize_for`, with a command that processes saved pages.

### Fixed

- Anker regional sitemaps no longer accept out-of-region URLs listed by a sitemap they do not recognize.
- `minisforumau` sitemaps no longer report other hosts, the homepage, `agents.md`, `/pages/` or service products such as shipping protection as interesting.
- `ankercom` sitemaps no longer treat German and Mexican storefront URLs as global.
