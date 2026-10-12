# monorepo

Rust monorepo for Bodul. It contains three independent Cargo workspaces:

| Folder      | Contents                              |
|-------------|---------------------------------------|
| `lib/`      | Shared libraries                      |
| `services/` | Backend services (REST and GraphQL)   |
| `apps/`     | Web apps (server-rendered HTML)       |

## Layout

### lib

| Crate                     | Purpose                                     |
|---------------------------|---------------------------------------------|
| `shared`                  | Code shared across the monorepo             |
| `money`                   | Money handling                              |
| `retailer_sauce`          | Retailer integrations                       |
| `retailer_data_extractor` | Extracting data from retailers              |
| `retailers_anker`         | Anker retailer                              |
| `retailers_mi`            | Mi retailer                                 |
| `retailers_minisforum`    | Minisforum retailer                         |
| `retailers_ugreen`        | Ugreen retailer                             |
| `retailers_wip`           | WIP retailer                                |

All library crates are empty for now.

### services

| Service             | Framework                          |
|---------------------|------------------------------------|
| `customer_facing`   | poem, poem-openapi, async-graphql  |
| `offers`            | poem, poem-openapi, async-graphql  |
| `retailer_sourcing` | axum, utoipa, async-graphql        |

Every service exposes the same routes:

| Path                | Purpose                         |
|---------------------|---------------------------------|
| `/api/health`       | Health check                    |
| `/api/openapi.json` | OpenAPI specification           |
| `/api/docs`         | Swagger UI                      |
| `/graphql`          | GraphQL (POST), GraphiQL (GET)  |

All services also depend on diesel (PostgreSQL), diesel_migrations, serde, serde_json and uuid. Their versions are declared once in `services/Cargo.toml`.

### apps

| App           | Framework |
|---------------|-----------|
| `customers`   | poem      |
| `development` | poem      |

## Requirements

- A Rust toolchain with Rust 2024 edition support
- The PostgreSQL client library for diesel (`libpq-dev` on Debian/Ubuntu)
- GNU Make
- [cargo-audit](https://crates.io/crates/cargo-audit) for `make audit` (`cargo install cargo-audit`)

## Running

Services and apps read their listen port from the `PORT` environment variable and default to 80:

```sh
cd services/offers
PORT=3000 make serve
```

## Tests

Each service and app has smoke tests in `src/main.rs`. They call the server's routes in memory, without opening a port, and check that:

- the health endpoint, GraphQL and Swagger UI respond (services)
- every path in the OpenAPI spec is actually served (services)
- the page renders (apps)

## Make targets

Every crate has a Makefile, and so do `lib/`, `services/`, `apps/` and the repo root. A target run in a folder runs in every crate below it. From the root, the order is `lib`, then `services`, then `apps`, and it stops at the first failure.

| Target      | Runs                                         |
|-------------|----------------------------------------------|
| `test`      | `cargo test`                                 |
| `check`     | `cargo check`                                |
| `build`     | `cargo build`                                |
| `release`   | `cargo build --release`                      |
| `fmt`       | `cargo fmt`                                  |
| `fmt-check` | `cargo fmt --check`                          |
| `fix`       | `cargo fix`                                  |
| `clippy`    | `cargo clippy`                               |
| `lint`      | `cargo clippy --all-targets -- -D warnings`  |
| `ci`        | `fmt-check`, `lint` and `test`               |
| `clean`     | `cargo clean`                                |

Some targets only exist in certain places:

| Target  | Where                    | Runs                    |
|---------|--------------------------|-------------------------|
| `serve` | each service and app     | `cargo run`             |
| `doc`   | each library and `lib/`  | `cargo doc --no-deps`   |
| `audit` | `lib/`, `services/`, `apps/` and the root | `cargo audit` on each workspace's `Cargo.lock` |

The shared targets live in `make/rust.mk`. `make/workspace.mk` runs a target across the crates listed in a folder's `MEMBERS`. To add a crate, give it a Makefile that includes `make/rust.mk` and add it to its folder's `MEMBERS`.

## Decisions

Architecture decisions are recorded in [docs/adr](docs/adr/README.md). The choice of frameworks for apps and services is explained in [ADR 1](docs/adr/0001-web-frameworks-for-apps-and-services.md).
