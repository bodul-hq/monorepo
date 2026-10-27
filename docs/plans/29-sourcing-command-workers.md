# Retailer sourcing command workers

Status: implemented on 2026-10-27. Planned against monorepo `9fb43bc` and Mulac `fd29f72`.
The user selected command-only automatic consumption and `COMMAND_WORKERS`
configuration defaulting to four; zero preserves manual operation.

## Implementation

Parse a nonnegative `usize`; reject empty, negative, malformed, overflowing or
non-Unicode values. Preserve explicit environment/Make overrides, including zero.
Log configured/started workers and disabled or database-free startup.

Reuse `kernel::io::run_command_worker`, the existing command consumer and child
cancellation tokens. Its loops poll every second and process batches of ten
sequentially via Tokio's blocking pool. The count controls concurrent background
handlers, not dedicated OS threads. Keep bus construction and feature assembly
worker-free. Spawn workers only from executable startup after features, listener
binding and shutdown-signal setup succeed. HTTP-only startup starts none.

Retain every worker handle. On signal or server failure, request cancellation,
drain HTTP and await all workers before leaving the runtime. Reserved batches
finish; cancellation takes effect between batches. Collect join errors without
abandoning remaining joins. The six-second download delay may lengthen shutdown.

Keep commands/events/outbox HTTP endpoints and existing queue/retry semantics.
Events/outbox remain manual; manual command runs can add concurrency beyond the
background count. No new sweeper, queue migration or Mulac modification is needed.
Document the lifecycle and [ADR 39](../adr/0039-sourcing-command-workers.md).

## Validation

Test parser/default/zero/error cases, deterministic manual bus construction,
automatic command completion with events still queued, disabled workers, shared
reservations, multiple gated workers and shutdown waiting for active work.
Use isolated PostgreSQL schemas for worker integration tests. Run service fmt/CI,
PostgreSQL tests and live startup/health plus command progression without calling
`/commands/run`. Implement the two pacing plans before enabling workers.

## Completion

Configuration, explicit startup, cancellation and joining are implemented with
manual event/outbox runners retained. Portable parser/join tests pass; isolated
PostgreSQL lifecycle/concurrency tests compile but could not run here. See the
[verification record](28-31-verification.md).
