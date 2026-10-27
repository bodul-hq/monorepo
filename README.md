# monorepo

Rust monorepo with shared libraries, backend services and web apps. Each
top-level folder is its own Cargo workspace, with its own `Cargo.lock`.

## Layout

```
.
├── lib/        Libraries shared by services and apps
├── services/   Backend HTTP services (REST + GraphQL)
├── apps/       Web apps that render HTML on the server
├── make/       Shared Make targets
├── bruno/      Bruno workspace with collections for calling the services by hand
└── docs/adr/   Architecture decision records
```

### lib

| Crate                     | Purpose                                                                     |
| ------------------------- | --------------------------------------------------------------------------- |
| `shared`                  | Code shared across the monorepo, including the `health()` liveness response |
| `object_storage_client`   | Async object storage client using REST or GraphQL                           |
| `money`                   | Money handling                                                              |
| `everyday`                | Common business, date/time, country, and geographic value types             |
| `everyday_infra`          | Serialization and chrono adapters for everyday value types                  |
| `retailer_sauce`          | Retailer integrations                                                       |
| `retailer_data_extractor` | Extracting data from retailers                                              |
| `retailers_anker`         | Integration for the Anker retailer                                          |
| `retailers_mi`            | Integration for the Mi retailer                                             |
| `retailers_minisforum`    | Integration for the Minisforum retailer                                     |
| `retailers_ugreen`        | Integration for the Ugreen retailer                                         |
| `retailers_wip`           | Integration for the WIP retailer                                            |
| `sitemap`                 | Parsing sitemap XML and building a crawl's sitemap tree                     |
| `testimony`               | Event sourcing abstractions, aggregate repositories, and query dispatch     |
| `testimony_infra`         | In-memory and PostgreSQL event stores for testimony                         |

Each crate has its own README. To add a retailer storefront, or finish one,
follow the [retailer integration playbook](docs/retailer-integration.md).

### services

| Service             | HTTP framework | OpenAPI                        | GraphQL                            |
| ------------------- | -------------- | ------------------------------ | ---------------------------------- |
| `customer_facing`   | poem           | poem-openapi                   | async-graphql, async-graphql-poem  |
| `offers`            | poem           | poem-openapi                   | async-graphql, async-graphql-poem  |
| `retailer_sourcing` | axum           | utoipa, utoipa-axum            | async-graphql, async-graphql-axum  |
| `object_storage`    | axum           | OpenAPI 3.1, utoipa-swagger-ui | async-graphql, custom HTTP adapter |

Services expose these standard routes (object storage requires bearer
authentication on every route and accepts only POST at `/graphql`):

| Path                | Purpose                                                   |
| ------------------- | --------------------------------------------------------- |
| `/api/health`       | Liveness check, returns `ok`                              |
| `/api/openapi.json` | OpenAPI specification                                     |
| `/api/docs`         | Swagger UI                                                |
| `/graphql`          | GraphQL over `GET` (query string) and `POST` (body)       |
| `/graphql/ide`      | GraphiQL, in debug builds only; release builds return 404 |

The original services have a single GraphQL `health` query that returns `ok`.
`object_storage` instead exposes immutable artifact creation and retrieval, with
health only at `/api/health`. See its
[setup, API examples and tests](services/object_storage/README.md) and [ADR 6](docs/adr/0006-object-storage-startup-and-access.md).

`retailer_sourcing` also runs the sitemap discovery pipeline: it crawls each
retailer's sitemaps and sorts the page URLs into products, catalogs and other
content. Offer discovery then records every product URL as an offer and
downloads its page. The code is organised as feature slices
([ADR 4](docs/adr/0004-feature-slices-for-retailer-sourcing.md)), and the
pipeline runs only when `DATABASE_URL` is set (see below).

`offers` imports retailer page snapshots through Mulac commands, saves Testimony
history and retry receipts in one transaction, and projects events for REST
lookups. Both queues run explicitly through the API. Set `DATABASE_URL` to
enable persistence; see its [setup and walkthrough](services/offers/README.md)
and [ADR 15](docs/adr/0015-offers-testimony-and-mulac.md).

### apps

`customers` and `development` are poem servers that render HTML on the server.
Each serves an index page at `/`. The
[development app](apps/development/README.md) also provides `/dashboard`,
combining authenticated database table reports from retailer sourcing and object
storage through private server-side clients.

## Requirements

- [rustup](https://rustup.rs/). `rust-toolchain.toml` pins Rust 1.98.0 with clippy and rustfmt, and rustup installs it on the first cargo command.
- libpq (for example `libpq-dev` on Debian/Ubuntu, `libpq` on macOS with Homebrew). The services and `testimony_infra` depend on diesel with the PostgreSQL backend, which links against it.
- GNU Make.
- A sibling `mulac` checkout next to `monorepo` for the services workspace's
  local Mulac dependencies. The queue adapters were verified with revision
  `9474319fff2237d81828a23d97df5d2f97741858`; Cargo path dependencies use the
  checked-out files rather than pinning that revision. See [ADR 17](docs/adr/0017-retailer-sourcing-local-mulac.md).
- PostgreSQL 16 or later for `object_storage` (a separate UTF8 database), and to run the `retailer_sourcing` pipeline or persisted `offers` imports.
- The `retailer_sourcing` pipeline also requires a running object storage service, configured with `OBJECT_STORAGE_URL` and `OBJECT_STORAGE_TOKEN`; see its [setup](services/retailer_sourcing/README.md).
- [Diesel CLI](https://diesel.rs/guides/getting-started) with PostgreSQL support, only for `make migrate` and `make migrate-redo`: `cargo install diesel_cli --no-default-features --features postgres --locked`.
- [cargo-audit](https://crates.io/crates/cargo-audit), only for `make audit`: `cargo install cargo-audit --locked`.

## Make targets

Run targets from the root to cover every workspace, from a workspace folder
(`lib`, `services`, `apps`) to cover its crates, or from a crate folder to cover
that crate.

| Target         | What it does                                                                                     |
| -------------- | ------------------------------------------------------------------------------------------------ |
| `ci`           | `fmt-check`, `lint` and `test`; run it before finishing a change                                 |
| `test`         | `cargo test`                                                                                     |
| `fmt`          | `cargo fmt`                                                                                      |
| `fmt-check`    | `cargo fmt --check`                                                                              |
| `lint`         | `cargo clippy --all-targets -- -D warnings`                                                      |
| `clippy`       | `cargo clippy`                                                                                   |
| `check`        | `cargo check`                                                                                    |
| `build`        | `cargo build`                                                                                    |
| `release`      | `cargo build --release`                                                                          |
| `fix`          | `cargo fix`                                                                                      |
| `clean`        | `cargo clean`                                                                                    |
| `audit`        | `cargo audit`, once per workspace                                                                |
| `doc`          | `cargo doc --no-deps` (lib crates only)                                                          |
| `serve`        | `cargo run` (service and app crates only)                                                        |
| `migrate`      | Apply pending migrations using `DATABASE_URL` (`retailer_sourcing`, `object_storage`, and `offers`)         |
| `migrate-redo` | Revert and reapply every migration, deleting the data (`retailer_sourcing`, `object_storage`, and `offers`) |

`object_storage`, `retailer_sourcing`, and `offers` also provide `make test-postgres`,
which uses `DATABASE_URL_TEST`, testing each case in an
isolated schema. Retailer sourcing separates domain entities from PostgreSQL
records throughout sitemap and offer discovery; see [ADR 7](docs/adr/0007-domain-entities-and-postgres-records.md).

More targets run only from the root, against `DATA_DIR`, which defaults to the
`data` folder next to the monorepo:

| Target                        | What it does                                                                                                                                                                                                                                                                                                                                                                                                  |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `export-offer-sources`        | Write every downloaded offer page to `<DATA_DIR>/<retailer>/offer_details/sources` (needs `DATABASE_URL`)                                                                                                                                                                                                                                                                                                     |
| `export-sitemaps`             | Write each retailer's latest sitemap crawl, its sources, processed tree and consolidated links, to `<DATA_DIR>/<retailer>/sitemap` (needs `DATABASE_URL`)                                                                                                                                                                                                                                                     |
| `dump-sitemap-sources`        | Download every fetched sitemap file from `retailer_sourcing`'s `/api/development/sitemaps/sources` to `<DATA_DIR>/<retailer>/sitemaps/sources/<retrieval id>/<url slug>.xml`; set `RETAILER_SOURCING_URL` (default `http://localhost:3000`) and, to override the hardcoded bearer token, `RETAILER_SOURCING_TOKEN`                                                                                            |
| `dump-offer-details`          | List offer metadata from `retailer_sourcing`'s `/api/development/offer-details/sources`, then download the newest HTML from `object_storage` to `<DATA_DIR>/<retailer>/offer_details/sources/<url path>.html`; set `RETAILER_SOURCING_URL` and `RETAILER_SOURCING_TOKEN` as for `dump-sitemap-sources`, plus `OBJECT_STORAGE_URL` (default `http://172.28.100.43:3000`) and optionally `OBJECT_STORAGE_TOKEN` |
| `extract-offer-details`       | Extract each page in `<DATA_DIR>/<retailer>/offer_details/sources` into `offer_details/extracted/<page>.json`; set `RETAILERS` to limit them                                                                                                                                                                                                                                                                  |
| `normalize-offer-details`     | Normalize each page in `offer_details/extracted` into `offer_details/normalized/<page>.json`; set `RETAILERS` to limit them                                                                                                                                                                                                                                                                                   |
| `structuralize-offer-details` | Structuralize each page in `offer_details/normalized` into `offer_details/structuralized/<page>.json`; set `RETAILERS` to limit them                                                                                                                                                                                                                                                                          |
| `process-sitemap-sources`     | Build a sitemap tree from each dumped crawl in `<DATA_DIR>/<retailer>/sitemaps/sources`, as `retailer_sourcing` does, and write it to `<DATA_DIR>/<retailer>/sitemaps/processed/<retrieval id>.json`                                                                                                                                                                                                          |
| `consolidate-sitemap-sources` | Sort the URLs of each tree in `<DATA_DIR>/<retailer>/sitemaps/processed` by kind of page, as `retailer_sourcing` does, and write them to `<DATA_DIR>/<retailer>/sitemaps/consolidate/<retrieval id>.json`                                                                                                                                                                                                     |
| `process-offer-details`       | Write the extracted, normalized and structurized stages of those pages; set `RETAILERS` to limit them                                                                                                                                                                                                                                                                                                         |

## Running a service or app

Every service and app reads its listen port from `PORT`, defaults to 3000 and
binds to `0.0.0.0`. An invalid `PORT` stops startup with an error. Give each
server its own port to run several at once:

```sh
cd services/offers && PORT=3001 make serve
curl localhost:3001/api/health
```

Servers shut down gracefully on Ctrl+C or SIGTERM.

`retailer_sourcing` configures its durable command and event queues when
`DATABASE_URL` points at a PostgreSQL database. On startup it applies pending migrations
from `services/retailer_sourcing/migrations`. Without `DATABASE_URL` it serves
HTTP only.

Its Makefile reads `DATABASE_URL` from `services/retailer_sourcing/.env`
(git-ignored) unless it is already set in your shell. `make .env` copies
`.env.example`, which points at the `retailer_sourcing-db` container from the
workspace's `compose-infra.yaml`:

```sh
cd services/retailer_sourcing
make .env      # once
make migrate   # optional; `make serve` also migrates on startup
PORT=3002 make serve
```

Nothing processes the queues in the background. Queue retrievals, then run the
queued commands and events in steps; each run waits for its batch to finish:

| Route                          | Body                          | Purpose                                                            |
| ------------------------------ | ----------------------------- | ------------------------------------------------------------------ |
| `POST /api/sitemap-retrievals` | `{"retailers": ["ugreenca"]}` | Queue one sitemap retrieval per retailer                           |
| `POST /api/commands/run`       | `{"limit": 10}`               | Process up to `limit` queued commands                              |
| `POST /api/events/run`         | `{"limit": 10}`               | Process up to `limit` queued events, which queue the next commands |

Without `DATABASE_URL` these routes return 503.

The command and event runners delegate to local Mulac's Axum adapters. Invalid
batch requests return plain-text `400` errors, while a valid request without a
database retains the existing JSON `503` error. Queue failures are reported in
the `200` batch response. See the [queue contract](services/retailer_sourcing/README.md#driving-the-workflow).

The `bruno/` folder is a [Bruno](https://www.usebruno.com/) workspace that lists
every collection. Open the `bruno` folder as a workspace in Bruno to load them
all at once. The workspace's `local` environment
(`bruno/environments/local.yml`) holds each service's address, with one variable
per collection:

| Variable                  | Collection          | Address                     |
| ------------------------- | ------------------- | --------------------------- |
| `objectStorageBaseUrl`    | `object_storage`    | `http://172.28.100.43:3000` |
| `retailerSourcingBaseUrl` | `retailer_sourcing` | `http://172.28.100.40:3000` |

Both addresses are containers from the `compose-apps.yaml` file.

The workspace's `office0-dev` environment (`bruno/environments/office0-dev.yml`)
uses the office deployment through Caddy:

| Variable                  | Address                                            |
| ------------------------- | -------------------------------------------------- |
| `objectStorageBaseUrl`    | `https://object-storage.service-dev.boduli.com`    |
| `retailerSourcingBaseUrl` | `https://retailer-sourcing.dev-service.boduli.com` |

Select `office0-dev` as the workspace environment while on the office network or
VPN. For object storage, also select the collection's `local` environment for
its development bearer token and request variables.

The Bruno collection in `bruno/retailer_sourcing` has a request for each route.

The [object storage Bruno collection](bruno/object_storage/README.md) covers
REST and GraphQL creation/retrieval, cross-API retries, health, and
documentation. Its own `local` environment holds the public development bearer
token. Its JavaScript request tests run with Bruno CLI; see the collection
README for Node.js setup and commands. Rust callers can use
[`object_storage_client`](lib/object_storage_client/README.md) from either
transport.

## Security advisories

`services/.cargo/audit.toml` and `apps/.cargo/audit.toml` ignore two quick-xml
advisories (RUSTSEC-2026-0194, RUSTSEC-2026-0195) that poem pulls in. The
affected XML parsing is unreachable because no service or app parses XML request
bodies, and each workspace's `clippy.toml` disallows poem's `Xml` types so `make
lint` fails if one is added. Remove these exceptions once quick-xml 0.41 or
later is resolved.

`make audit` also warns that `unsafe-libyaml` is unmaintained
(RUSTSEC-2024-0449). poem pulls it in through `serde_yaml`, and no fixed version
exists yet.

## Decisions

Framework and architecture decisions are recorded in [docs/adr](docs/adr/README.md).

## Contributing

See [AGENTS.md](AGENTS.md) for commit, dependency and crate conventions, and
[CHANGELOG.md](CHANGELOG.md) for notable changes.
