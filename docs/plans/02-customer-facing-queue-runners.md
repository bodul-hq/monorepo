# Customer facing Mulac queue runners

Status: Planned. Covers TODO item 2: set up `customer_facing` with Mulac queue
runners and a health endpoint. It can start without another TODO item being
implemented.

The service will accept durable inbox messages and expose explicit batch runners
using the local Mulac checkout. Preserve its existing REST and GraphQL health
behavior. Offer projection, offer retrieval, and incoming offer event handling
belong to later TODO items.

## Current implementation

- [The server](../../services/customer_facing/src/main.rs) already serves
  `GET /api/health`, GraphQL health over GET and POST, Swagger UI, OpenAPI, and
  debug-only GraphiQL. It parses `PORT`, defaults to 3000, and shuts down
  gracefully. Keep these existing contracts.
- [The manifest](../../services/customer_facing/Cargo.toml) already includes
  Diesel and migrations, but there is no pool, migration directory, queue state,
  service README, or database test target.
- [Offers startup](../../services/offers/src/main.rs) assembles a persistent
  kernel and its command/event adapters. Its
  [queue endpoint declarations](../../services/offers/src/orchestration/mod.rs)
  delegate validation and blocking execution to Mulac's Poem adapters.
- [The services workspace](../../services/Cargo.toml) currently declares a Git
  kernel, while [offers](../../services/offers/Cargo.toml) directly uses local
  Mulac paths. These sources must remain consistent within each service.

## Bootstrap policy

The proposed bootstrap follows offers: require a valid `DATABASE_URL`, migrate
before accepting traffic, expose all four queue runners, and leave outbox
delivery unconfigured. The database startup policy and whether to expose the
unused outbox runner need confirmation before implementation. These choices do
not block drafting the common implementation below.

Health remains a liveness response, `ok`, rather than a database probe. Startup
failure is separate from health after a server has started. Do not add a broker
dependency or background queue processing to this bootstrap.

## Implementation steps

### 1. Share the local Mulac dependencies

Declare dependencies reused by offers and customer facing in
`services/Cargo.toml`, then use workspace inheritance in both service manifests.
Resolve paths relative to the services workspace:

| Dependency | Local source | Needed features |
| --- | --- | --- |
| `kernel`, package `mulac-kernel` | `../../mulac/kernel` | Existing default features |
| `inbox_api` | `../../mulac/libs/inbox_api` | Default `diesel` |
| `commanding_api` | `../../mulac/libs/commanding_api` | Tokio support through the Poem adapter |
| `eventing_api` | `../../mulac/libs/eventing_api` | Tokio support through the Poem adapter |
| `mulac_poem` | `../../mulac/libs/mulac_poem` | Default `diesel`, plus `commanding`, `eventing`, and `outbox` |

The available feature names are defined in
[Mulac's Poem manifest](../../../mulac/libs/mulac_poem/Cargo.toml). Add
`outbox_api` only if customer facing constructs a configured `OutboxService` or
uses its testing fixtures; `OutboxAdapter::unconfigured()` alone does not require
a direct dependency. Keep test-only `testing` features in dev dependencies.

Coordinate the shared manifest changes with
[the retailer sourcing upgrade](01-retailer-sourcing-local-mulac.md). If this
plan is implemented first, preserve retailer sourcing's Git kernel dependency
by moving its existing exact Git/tag declaration into that crate's manifest.
It then has one consumer, while offers and customer facing inherit the shared
local kernel. The retailer upgrade later changes sourcing back to workspace
inheritance. Keep the Rust dependency name `kernel` in all three crates; do not
silently migrate sourcing or rename imports as part of customer-facing setup.

Use `kernel::io::DbPool` or an equivalent Diesel pool alias; customer facing does
not need a Testimony dependency merely to obtain a pool type. Keep dependency
lists alphabetical and regenerate the services lockfile.

### 2. Add service-owned persistence and startup

Add `src/bootstrap.rs` with a pool constructor and embedded migrations. Follow
[offers bootstrap](../../services/offers/src/bootstrap.rs), but return a clear
startup error rather than using `expect` for missing required configuration.
Validate missing, empty, invalid-Unicode, malformed, unreachable, and
migration-failing database configuration according to the selected startup
policy. Do not include credentials in error messages.

Use a separate customer-facing development database and a separate test database.
Create an initial migration for `inbox_entries`, `command_entries`,
`event_entries`, and `outbox_entries`, including their scheduling/reservation
indexes and `extra_info` columns. Start from
[offers' queue migration](../../services/offers/migrations/2026-10-19-000001_mulac/up.sql)
and check it against the local Mulac storage schemas. Include a matching down
migration. Do not copy `event_store`, `read_offers`, or offer identity tables.

Add `.env.example`, ignored `.env` setup, Diesel CLI configuration, and Make
targets for `migrate`, `migrate-redo`, and `test-postgres`. Preserve explicit
shell and command-line overrides when reading `.env`, including empty values.
Document that `migrate-redo` deletes data; it is never part of normal startup.

### 3. Assemble the queue runtime

Introduce `start_mulac(pool)` and a cloneable `AppState`. Assemble:

- `kernel::boot(KernelConfig::default()).start_persistent(pool, 0)` for command
  and event consumers. Zero drain rounds keeps accidental calls to the typed
  dispatch helper from processing other queues inline.
- `InboxAdapter::new(InboxService::from_postgres(pool))` for durable submission
  and inbox processing.
- `CommandingAdapter::new(CommandingService::new(kernel.command_consumer()))`
  and `EventingAdapter::new(EventingService::new(kernel.event_consumer()))`.
- `OutboxAdapter::unconfigured()` if the four-runner bootstrap is selected.

Keep the kernel handle in the application lifecycle, let the HTTP server finish
in-flight requests on shutdown, then call `kernel.shutdown()`. Do not spawn
command/event workers or invoke a queue while building routes.

The bootstrap has no production command handlers or event subscribers yet.
Unknown inbox routing keys are accepted durably; when commands run, missing
handlers are reported through Mulac's retry/error behavior. Do not install a
catch-all handler that discards messages. Use handlers defined only in tests to
verify successful command-to-event dispatch.

The local [inbox service](../../../mulac/libs/inbox_api/src/service.rs) queues
commands from inbox messages; it does not dispatch directly to event
subscribers. Later offer persistence must provide the handler that interprets
incoming offer events and writes the projection, or explicitly introduce an
event bridge. Receiving a message here does not mean an offer was persisted.

### 4. Declare and document the HTTP routes

Change route construction to `app(state)` and keep it independent of process
startup so tests can inject state without opening a port. Split monitoring and
orchestration declarations into small modules as needed. Keep one
`OpenApiService` containing all endpoint declarations.

| Method and path | Behavior |
| --- | --- |
| `GET /api/health` | Existing `200`, text `ok` |
| `POST /api/inbox` | Mulac `SubmitInboxRequest`; durable acceptance returns `202` and `message_id` |
| `POST /api/inbox/run` | Queue commands from up to `limit` due inbox messages |
| `POST /api/commands/run` | Execute up to `limit` due commands |
| `POST /api/events/run` | Dispatch up to `limit` due events |
| `POST /api/outbox/run` | When included, valid requests return `503` while delivery is unconfigured |

All runners accept the shared `RunRequest` with required integer `limit` from
1 through 1000 and return Mulac's `RunResponse`. Full success reports
`{"processed":N,"errors":[]}`; batch failures report `processed: null` and
errors while retaining successful items, including storage/reservation failures
reported by a runner. Preserve the adapter's `400` input and `415` content-type
responses. Inbox submission storage failures and blocking-task failures return
sanitized `500` responses. Use the
existing shared schema wrappers, avoiding duplicate OpenAPI schema definitions.

Preserve `/api/openapi.json`, `/api/docs/`, GET and POST `/graphql`, and
debug-only `/graphql/ide`. Queue mutations remain REST-only; GraphQL keeps its
existing health field. Update route smoke tests to exercise the proper HTTP
method, since the current route enumeration sends GET to every documented path.

### 5. Add operational documentation

Create `services/customer_facing/README.md` covering local Mulac checkout layout,
database setup, startup configuration, migrations, manual queue ordering, retry
reports, and test commands. State that there are no background workers and that
the projection handler is not part of this item. Document operator recovery for
stale reservations; this bootstrap does not introduce a recurring sweeper.

Update root README requirements and service descriptions. Add Bruno requests
for health, inbox submission, and the selected runners, using a
`customerFacingBaseUrl` environment variable. Test payloads should use an
explicit development routing key and explain that no production handler exists
yet, rather than pretending that bootstrap imports offers successfully.

## Validation and acceptance

Run ordinary HTTP tests without a live database using Mulac fixtures or a lazy
pool with a deterministic unavailable endpoint. Verify existing health,
GraphQL, docs, `PORT`, and shutdown contracts; inbox envelope validation;
duplicate OpenAPI schema avoidance; runner limit bounds; malformed JSON;
content-type errors; and unavailable outbox behavior. Test database startup
configuration separately from route construction.

Use `DATABASE_URL_TEST` and one disposable schema per PostgreSQL test. Verify
migrations down/up, empty queues, durable submission and original-row retention
for repeated message IDs, payload/metadata preservation, batch limits, and retry
scheduling. Use a test-only command handler and event subscriber to prove that
submission, inbox, commands, and events advance separately and only when their
respective runner is invoked. Verify an unavailable outbox leaves seeded rows
unchanged. Never reset an existing development database for a test.

From the repository root, complete the implementation checks:

```sh
make -C services fmt
make -C services ci
make -C services audit
make -C services/customer_facing test-postgres
PORT=3108 make -C services/customer_facing serve
```

Supply the configured development and test database URLs through the environment
or `.env`. With the server running, request
`http://localhost:3108/api/health` and each implemented runner; check GraphQL
health and OpenAPI as well. Run the debug/release GraphiQL smoke test in both
profiles. Dependency/lockfile changes make the audit required; report new
advisories.

This item is complete when customer facing starts with its own persistent queue
runtime, the selected endpoints retain Mulac's contract, health still returns
`ok`, tests demonstrate separate explicit queue steps, and documentation explains
what is ready for the later offer projection work.
