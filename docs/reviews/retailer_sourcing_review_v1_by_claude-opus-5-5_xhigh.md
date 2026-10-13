# Review: services/retailer_sourcing (v1)

- Date: 2026-10-13
- Reviewer: Claude Opus 5.5 (`claude-opus-5-5`), effort `xhigh`
- Branch: `feat/retailer-sourcing-service-skeleton` at `a3a74a5`
- Scope: the whole service, with most attention on the sitemap pipeline that was
  ported from the MVP (`src/sitemap_discovery/` and the crate-level modules it
  uses)

Paths are relative to `services/retailer_sourcing/src/` unless stated otherwise.

## Method

Four reviewers each took a different lens:

- **Software architect:** module boundaries, how the pipeline stages connect,
  the data model, and fit with the ADRs.
- **Principal engineer:** correctness, idempotency, transactions, the retrieval
  status flow, and the parser.
- **Staff engineer:** security, resource limits, blocking I/O, observability,
  and supply chain.
- **Team lead / senior engineer:** whether `make ci` passes, repo conventions,
  tests, and readability.

The reviewers read the service, the MVP it was ported from, the mulac-kernel
0.3.0 source and the locked dependency sources. Findings marked ✔ were checked
again against the code or the Mulac source after the reviews came in.

## Summary

The HTTP server is in good shape and the sitemap parsing code is well tested.
The ported sitemap pipeline isn't safe to wire up yet:

- When something fails, it records the run as a success with no data.
- A single retry by Mulac can leave a command permanently failed.
- Nothing limits what it fetches from retailer sites.

Separately, one problem is live today: `/graphql` accepts request bodies of any
size.

**Current state:** `make ci` fails at the lint step with 113 clippy errors: 102
dead code (nothing in `main.rs` uses the pipeline yet), 8 unused `io` re-exports
and 7 lints. `cargo test` passes all 27 tests.

## Critical

### 1. Failures are recorded as successful empty runs ✔

All four reviewers flagged this.

- `sitemap_discovery/retrieval.rs:604` returns only `result.documents` and drops the collected errors.
- In processing, one bad child sitemap fails its whole root (`sitemap_discovery/processing/sitemap.rs:136`), and `sitemap_discovery/processing.rs:135` ignores the errors (`_errors`).
- If a retailer's CDN returns 403 or 503, the run still finishes as `grouped` with `error` empty and 0 products. Downstream can't tell "retailer has no products" from "fetch failed".

**Fix:** return the errors, skip and record a bad child instead of failing its
root, and define when a run counts as failed or partial.

### 2. The handlers break when Mulac retries them ✔

Mulac delivers each command at least once. It runs the handler, then records the
events as a separate step, and retries any error up to 6 times
(`libs/commanding/src/dispatcher.rs:108-114` and
`libs/commanding/src/command_consumer.rs` in mulac).

- Retrieval inserts the command's own `retrieval_id` as primary key (`sitemap_discovery/retrieval.rs:138`), so every retry fails on a duplicate key and the command ends up permanently failed.
- Processing and grouping insert a row, then update the status over a separate connection (`sitemap_discovery/processing.rs:187-188`, `sitemap_discovery/grouping.rs:215-216`). A failure between the two leaves duplicate rows, or a retrieval stuck mid-pipeline.
- The subscribers give each new command a random id (`sitemap_discovery/retrieval.rs:241`, `sitemap_discovery/processing.rs:257`), so Mulac's de-duplication by command id never kicks in.

**Fix:** one transaction per handler; if the step is already done, re-send the
event and succeed; derive command ids from the event id; port the MVP's unique
constraints.

## High

### 3. Nothing limits what the crawler fetches ✔

- Response bodies and gzip decompression have no size cap (`retailer_data_ingestion.rs:53,70`), so a gzip bomb can exhaust memory.
- Child `<loc>` URLs are followed to any host or scheme, and reqwest follows up to 10 redirects anywhere. A hostile or compromised sitemap could point the crawler at internal addresses such as `169.254.169.254`, and the response gets stored.
- There is no cap on the number of child sitemaps or total bytes.
- Processing rebuilds a child sitemap every time an index lists it, and listing it twice at each level grows exponentially (`sitemap_discovery/processing/sitemap.rs:107-151`).
- All documents go into one INSERT statement, which breaks above roughly 9,362 documents because of Postgres's bind-parameter limit (65,535 parameters, 7 per row).

### 4. `/graphql` reads request bodies with no size limit, live today ✔

`async-graphql-axum` reads the raw body stream, which bypasses axum's default
limit (`main.rs:91`). An unauthenticated client can stream an endless POST until
the process runs out of memory, and multipart uploads are written to temp files
with no limit.

**Fix:** add a request body limit, a timeout and multipart limits.

### 5. The quick-xml audit exception is now wrong ✔

`services/.cargo/audit.toml` ignores two quick-xml advisories (RUSTSEC-2026-0194
and RUSTSEC-2026-0195) on the grounds that "no service parses XML". This crate
now parses untrusted XML with its own quick-xml 0.40.1, which those advisories
cover. Today's parser doesn't reach the affected code, but nothing enforces
that.

**Fix:** bump this crate to quick-xml 0.41 or later and narrow the ignore to
poem's copy.

### 6. There are no migrations

`schema.rs` claims to be generated by Diesel CLI, but there's no `migrations/`
folder or `diesel.toml`, and `diesel_migrations` is unused. The unique
constraints the retry logic depends on, and Mulac's own tables, exist nowhere in
this repo.

### 7. There's no plan for how the pipeline runs

It uses blocking HTTP and database calls inside an async service. reqwest's
blocking client panics when it's built inside an async runtime, and the code
builds a new client for every URL. In the MVP this ran as separate synchronous
programs. A crawl can't be cancelled on shutdown.

### 8. The commit history doesn't follow AGENTS.md

Nine commits are titled "1", "2", "ee", "wip" and similar, with no bodies, and
the `lib` changes are mixed in with service changes.

## Medium

### 9. The retrieval status rules are too loose

In `sitemap_discovery/retrieval.rs:404-496`:

- A run can go from `failed` straight to `processed` or `grouped`, skipping stages.
- `mark_failed` works from any state, so it can turn a `grouped` run back into `failed`.
- Processing and grouping never mark a run as failed.
- The "not in expected state" error names only one of the two allowed states.

### 10. Every error is treated as a storage error

`storage_err` (`main.rs:20-28`) maps every failure to `CommandError::Storage`.
Permanent problems like a missing sitemap config are retried 6 times and then
dropped. The shared error types also sit in `main.rs`.

### 11. The parser is stricter than real sitemaps ✔

`sitemap_discovery/processing/parse.rs:54` uses quick-xml's default settings:

- An unescaped `&` in a `<loc>` fails the whole document.
- So does an unknown entity like `&nbsp;` in an image title, or a root element written as `<urlset/>`.
- `lastmod` values without seconds are silently dropped.

Combined with finding 1, one bad character can wipe out a retailer's whole
result.

### 12. Processing re-reads the current config

`sitemap_discovery/processing.rs:172` looks up the retailer's current config
instead of using the roots that were actually fetched. A config change between
the two steps gives an empty result.

### 13. The sitemap tree is walked by two implementations

Fetching walks it breadth-first (`sitemap_discovery/retrieval.rs:580-767`) and
processing walks it depth-first (`sitemap_discovery/processing/sitemap.rs`),
with different rules. `SitemapItem` is defined twice.

### 14. Module boundaries are tangled

- Everything goes through `io` re-exports, and `prelude.rs` isn't even declared.
- The stages import each other's internals in a cycle.
- The central `MvpEvent`/`AppCommand` enums tie every stage together, and their serde tags become a stored contract once events are persisted.

### 15. Nothing is logged

No tracing subscriber is installed, so the fetch timing records and Mulac's
errors go nowhere. Health is a static `ok` with no readiness check.

### 16. The crawler isn't polite to retailers

It sends a Chrome User-Agent and has no rate limiting, backoff or robots.txt
check. The MVP logs, which used the same client, show about 87k "429 Too Many
Requests" responses from eu.ugreen.com.

### 17. The pipeline pulls in a heavy dependency stack

`retailer_data_extractor` brings the whole HTML extraction stack, including the
`fxhash` and `kuchiki` advisories, just for two registry functions. The `rand`
advisory comes in only at build time and isn't in the binary.

### 18. The supply chain and docs need attention

- Mulac is pinned to a git tag rather than a commit, and CI runs cargo without `--locked`.
- `margo` points at plain-HTTP `localhost:8080`.
- There's no ADR for Mulac, Diesel/Postgres or the registries, and the README's requirements are out of date.

## Low

- **Naming:** `MvpEvent`, `i` for a sitemap item, `parse_it`, and "roadmap Stage B" in the module docs.
- **Duplication:** the two subscribers are about 45 nearly identical lines each, and the "too large for storage" check is repeated 5 times.
- **Extra copies:** raw bodies are cloned 3–4 times between fetching and processing.
- **Tests:** the fetch traversal has an injection point for the HTTP call (`getter: fn`) but no tests, and `GroupedLinks::from_document` is untested.
- **Minor bugs:** a failed fetch leaves its URL marked active, so a later reference to it is misreported as a cycle. `fetched_at` is one timestamp for the whole crawl.

## What's fine

- The event and command serialization round-trips correctly.
- No panic is reachable from fetched input.
- quick-xml doesn't expand DTD entities, so there is no XXE exposure.
- Integer conversions use `try_from`.

## Decisions to record as ADRs

1. **How the pipeline runs:** inside the axum process or as a separate worker, sync or async, and whether one command covers a whole crawl or one sitemap document.
2. **Who owns persistence:** database or schema per service, who migrates Mulac's tables, raw blobs vs relational rows, retention, and the idempotency rules every handler must follow.
3. **Module and crate boundaries:** `io` vs `prelude`, a `lib/sitemap` crate for the models, parser and single traversal, splitting the registry from the HTML stack, and the event this service publishes to `offers`.

## Suggested order

1. **`/graphql` body limit and timeouts:** small, and fixes the risk that's live today.
2. **Green CI and clean history:** fix the 7 lints, then either wire the pipeline in or put `#[expect(dead_code)]` on the modules until it is. Rewrite the commits; if the branch is pushed, that means a `--force-with-lease` push.
3. **Failure handling:** surface and store errors, tolerate a bad child, tighten the status rules, and add table tests for the traversal using the existing getter seam.
4. **Idempotency:** migrations with the unique constraints, one transaction per handler, re-send the event when a step is already done, deterministic command ids.
5. **Input hardening:** one shared HTTP client with size caps, a host allowlist and redirect policy; quick-xml 0.41 or later; a more lenient parser.
6. **The three ADRs above.**
