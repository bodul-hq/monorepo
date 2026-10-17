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
├── bruno/      Bruno collections for calling the services by hand
└── docs/adr/   Architecture decision records
```

### lib

| Crate                     | Purpose                                                                     |
| ------------------------- | --------------------------------------------------------------------------- |
| `shared`                  | Code shared across the monorepo, including the `health()` liveness response |
| `object_storage_client`   | Async object storage client using REST or GraphQL |
| `money`                   | Money handling                                                              |
| `retailer_sauce`          | Retailer integrations                                                       |
| `retailer_data_extractor` | Extracting data from retailers                                              |
| `retailers_anker`         | Integration for the Anker retailer                                          |
| `retailers_mi`            | Integration for the Mi retailer                                             |
| `retailers_minisforum`    | Integration for the Minisforum retailer                                     |
| `retailers_ugreen`        | Integration for the Ugreen retailer                                         |
| `retailers_wip`           | Integration for the WIP retailer                                            |
| `sitemap`                 | Parsing sitemap XML and building a crawl's sitemap tree                     |

Every crate except `shared` has no code yet, only a placeholder test. Each crate
has its own README.

### services

| Service             | HTTP framework | OpenAPI             | GraphQL                           |
| ------------------- | -------------- | ------------------- | --------------------------------- |
| `customer_facing`   | poem           | poem-openapi        | async-graphql, async-graphql-poem |
| `offers`            | poem           | poem-openapi        | async-graphql, async-graphql-poem |
| `retailer_sourcing` | axum           | utoipa, utoipa-axum | async-graphql, async-graphql-axum |
| `object_storage` | axum | OpenAPI 3.1, utoipa-swagger-ui | async-graphql, custom HTTP adapter |

Every service serves the same routes:

| Path                | Purpose                                                   |
| ------------------- | --------------------------------------------------------- |
| `/api/health`       | Liveness check, returns `ok`                              |
| `/api/openapi.json` | OpenAPI specification                                     |
| `/api/docs`         | Swagger UI                                                |
| `/graphql`          | GraphQL over `GET` (query string) and `POST` (body)       |
| `/graphql/ide`      | GraphiQL, in debug builds only; release builds return 404 |

The original services have a single GraphQL `health` query that returns `ok`.
`object_storage` instead exposes immutable artifact creation and retrieval,
with health only at `/api/health`. See its [setup, API examples and tests](services/object_storage/README.md)
and [ADR 6](docs/adr/0006-object-storage-startup-and-access.md).

`retailer_sourcing` also runs the sitemap discovery pipeline: it crawls each
retailer's sitemaps and sorts the page URLs into products, catalogs and other
content. Offer discovery then records every product URL as an offer and
downloads its page, skipping URLs downloaded in the last 6 hours. The code is organised as feature slices
([ADR 4](docs/adr/0004-feature-slices-for-retailer-sourcing.md)), and the
pipeline runs only when `DATABASE_URL` is set (see below).

### apps

`customers` and `development` are poem servers that render HTML on the server.
Each serves an index page at `/`.

## Requirements

- [rustup](https://rustup.rs/). `rust-toolchain.toml` pins Rust 1.98.0 with clippy and rustfmt, and rustup installs it on the first cargo command.
- libpq (for example `libpq-dev` on Debian/Ubuntu, `libpq` on macOS with Homebrew). The services depend on diesel with the PostgreSQL backend, which links against it.
- GNU Make.
- PostgreSQL, only to run the `retailer_sourcing` pipeline.
- [Diesel CLI](https://diesel.rs/guides/getting-started) with PostgreSQL support, only for `make migrate` and `make migrate-redo`: `cargo install diesel_cli --no-default-features --features postgres --locked`.
- [cargo-audit](https://crates.io/crates/cargo-audit), only for `make audit`: `cargo install cargo-audit --locked`.

## Make targets

Run targets from the root to cover every workspace, from a workspace folder
(`lib`, `services`, `apps`) to cover its crates, or from a crate folder to cover
that crate.

| Target         | What it does                                                                     |
| -------------- | -------------------------------------------------------------------------------- |
| `ci`           | `fmt-check`, `lint` and `test`; run it before finishing a change                 |
| `test`         | `cargo test`                                                                     |
| `fmt`          | `cargo fmt`                                                                      |
| `fmt-check`    | `cargo fmt --check`                                                              |
| `lint`         | `cargo clippy --all-targets -- -D warnings`                                      |
| `clippy`       | `cargo clippy`                                                                   |
| `check`        | `cargo check`                                                                    |
| `build`        | `cargo build`                                                                    |
| `release`      | `cargo build --release`                                                          |
| `fix`          | `cargo fix`                                                                      |
| `clean`        | `cargo clean`                                                                    |
| `audit`        | `cargo audit`, once per workspace                                                |
| `doc`          | `cargo doc --no-deps` (lib crates only)                                          |
| `serve`        | `cargo run` (service and app crates only)                                        |
| `migrate`      | Apply pending migrations to `DATABASE_URL` (`retailer_sourcing` only)            |
| `migrate-redo` | Revert and reapply every migration, deleting the data (`retailer_sourcing` only) |

## Running a service or app

Every service and app reads its listen port from `PORT`, defaults to 3000 and
binds to `0.0.0.0`. An invalid `PORT` stops startup with an error. Give each
server its own port to run several at once:

```sh
cd services/offers && PORT=3001 make serve
curl localhost:3001/api/health
```

Servers shut down gracefully on Ctrl+C or SIGTERM.

`retailer_sourcing` starts its sitemap discovery workers when `DATABASE_URL`
points at a PostgreSQL database. On startup it applies any pending migrations
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

The [Bruno](https://www.usebruno.com/) collection in `bruno/retailer_sourcing`
has a request for each route. Open the folder in Bruno and pick the `local`
environment, which targets the `retailer_sourcing` container from the
workspace's `compose-apps.yaml` at `http://172.28.100.40:3000`.

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
