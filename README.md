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

`shared` provides the liveness response used by the services’ HTTP and GraphQL adapters. Other library crates are empty for now.

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

## Dependency conventions

Each Cargo workspace declares dependencies shared by its members under `[workspace.dependencies]`. Member crates inherit them with `name.workspace = true`. Framework and runtime versions live in `services/Cargo.toml` and `apps/Cargo.toml`; the workspaces remain independent. Poem's `test` feature belongs in each member's dev-dependencies.

## Requirements

- A Rust toolchain with Rust 2024 edition support
- The PostgreSQL client library for diesel (`libpq-dev` on Debian/Ubuntu)
- GNU Make
- [cargo-audit](https://crates.io/crates/cargo-audit) for `make audit` (`cargo install cargo-audit`)

## Running

Services and apps read their listen port from the `PORT` environment variable and default to 3000:

```sh
cd services/offers
make serve # listens on port 3000
# Set PORT to run another server alongside it:
PORT=3001 make serve
```

`PORT` must be an integer between 0 and 65535. Invalid or non-Unicode values cause startup to return an error before binding; an absent value uses the documented default. Port 0 asks the operating system to select an available port. Use distinct explicit ports when running several servers together.

On Ctrl+C or `SIGTERM`, a server stops accepting connections, finishes the requests in progress and exits with status 0. It waits for those requests without a time limit, so deployments rely on the orchestrator's grace period (for example, Kubernetes' `terminationGracePeriodSeconds`) to bound shutdown.

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

## Security audit

`services/.cargo/audit.toml` temporarily ignores the two quick-xml advisories below because no service accepts XML request bodies. `make audit` still reports the unsafe-libyaml warning in `services/`. All of them come from poem 3.1.12, which `customer_facing` and `offers` use. poem-openapi always enables poem's `xml` and `yaml` features, so they can't be turned off.

- [ ] **quick-xml 0.36.2: memory-exhaustion denial of service** ([RUSTSEC-2026-0195](https://rustsec.org/advisories/RUSTSEC-2026-0195), high). Fixed in quick-xml 0.41.
- [ ] **quick-xml 0.36.2: quadratic run time on duplicate attribute checks** ([RUSTSEC-2026-0194](https://rustsec.org/advisories/RUSTSEC-2026-0194), high). Fixed in quick-xml 0.41.
- [ ] **unsafe-libyaml 0.2.11 is unmaintained** ([RUSTSEC-2024-0449](https://rustsec.org/advisories/RUSTSEC-2024-0449), warning). It comes in through serde_yaml, which poem's `master` branch still uses.

Things to consider:

- quick-xml only runs when a handler accepts an XML request body, through poem's `Xml` extractor or poem-openapi's `Xml` payload. No service does this today, so the vulnerable code is compiled in but unreachable.
- poem's `master` branch moves to quick-xml 0.42 ahead of poem 4.0, which isn't released yet. async-graphql-poem still requires poem 3, so upgrading also needs a matching async-graphql-poem release.
- The exceptions in `services/.cargo/audit.toml` name only RUSTSEC-2026-0194 and RUSTSEC-2026-0195, so other advisories remain visible. Do not introduce XML request handlers while these exceptions are present: `services/clippy.toml` disallows poem's `Xml` extractor and poem-openapi's `Xml` payload, so `make lint` fails if either is used. On each Poem or poem-openapi upgrade, check the resolved quick-xml version and remove the exceptions, and the clippy entries, once it is at least 0.41. Track compatible Poem and async-graphql-poem releases for that upgrade.

## Decisions

Architecture decisions are recorded in [docs/adr](docs/adr/README.md):

- [ADR 1](docs/adr/0001-web-frameworks-for-apps-and-services.md): web frameworks, routes, default port and bind address for apps and services
- [ADR 2](docs/adr/0002-workspace-dependencies-and-port-validation.md): workspace dependency inheritance and `PORT` validation
