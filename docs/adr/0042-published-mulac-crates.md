# ADR 42: Published Mulac crates

Status: Accepted; supersedes ADR 17's local-checkout dependency decision.

Date: 2026-10-31

## Decision

Resolve all direct Mulac dependencies through the null-labs-hq sparse registry
at https://null-labs-hq.github.io/crates/, starting at version 0.4.1.
Retain existing Rust aliases for renamed API packages and existing features.
Commit the resolved services lockfile, including registry sources and checksums.

## Consequences

Development and CI no longer require a sibling Mulac checkout. Registry access
is required for uncached builds. CI checks out only this repository.
Queue contracts, frameworks and worker configuration remain unchanged.
Other local libraries within this monorepo remain path dependencies.
