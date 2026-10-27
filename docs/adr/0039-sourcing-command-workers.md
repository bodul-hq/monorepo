# ADR 39: Background command workers in retailer sourcing

Status: Accepted; supersedes ADR 17's prohibition on background command workers.

Date: 2026-10-27

## Context

Retailer sourcing currently advances only when an HTTP runner is called. The
user requested startup workers and selected command-only execution, with four
concurrent workers by default and a configurable count. Events and outbox
delivery retain their explicit runners.

## Decision

Read `COMMAND_WORKERS` as a nonnegative integer, defaulting to four. Zero selects
manual operation. Reject invalid configuration at startup. Start no workers
without database-backed features. Keep bus construction worker-free; the
executable starts workers only after features, listener and signal setup succeed.

Reuse Mulac's existing `run_command_worker` consumer loop and cancellation token.
Each loop polls every second and reserves batches of ten commands, processing
them sequentially on Tokio's blocking pool. This controls concurrent background
handlers rather than creating a dedicated OS thread for each worker. Keep queue
records, retries, manual endpoints and zero inline drain rounds unchanged.

Own every worker join handle in the application. On a shutdown signal or server
failure, request cancellation, drain HTTP requests and join every worker before
leaving the runtime. Already reserved batches finish; there is no forced abort
or new stale-reservation sweeper. Record worker configuration/startup/shutdown
in logs, retaining Mulac's existing failure logs.

Apply the separately requested fetch pacing: each offer-download invocation
waits six seconds before fetching, and each sitemap crawl waits a random one to
five whole seconds between actual fetch attempts. Sitemap pacing replaces the
old pause after groups of three. These pauses belong to handlers/crawls; workers
may run concurrently and manual command batches can add concurrency.

## Consequences

Command progression begins automatically with a configured database. Operators
must still run events to dispatch reactions and outbox to deliver exports.
Set `COMMAND_WORKERS=0` for manual walkthroughs and controlled tests. Existing
manual tests remain deterministic because assembling a bus starts no workers.

Shutdown can wait through the remaining sleeps and network calls in a reserved
batch. The count is not a shared retailer/global request-rate limit. No Mulac
changes or queue migration are required, and other services keep their existing
manual queue behavior.
