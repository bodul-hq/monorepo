# ADR 2: Workspace dependencies and port validation

- Status: Accepted
- Date: 2026-10-12
- Supplements: [ADR 1](0001-web-frameworks-for-apps-and-services.md)

## Context

Repeated framework versions can drift between members of one workspace. Passing
an unchecked `PORT` string to a listener produces framework-specific startup
errors.

## Decision

Declare each dependency shared by multiple members once in that workspace's
`[workspace.dependencies]` and inherit it in member manifests. Keep test-only
features in dev-dependencies. The library, service and app workspaces stay
independent.

Parse an explicitly configured `PORT` as a `u16` before binding. Reject invalid
or non-Unicode values with an input error instead of silently falling back or
panicking. An absent value uses the default established in ADR 1; port 0 retains
its usual operating-system-assigned semantics.

## Consequences

Shared framework and runtime versions have one source within each workspace.
Deployments get an actionable startup error for invalid port configuration.
Neither convention changes the HTTP frameworks, routes, or bind address.
