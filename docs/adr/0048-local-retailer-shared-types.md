# ADR 48: Local ownership of retailer shared types

- Status: Accepted
- Date: 2026-11-09
- Supersedes: ADR 47's requirement to publish and consume a new `bodul_shared` version

## Decision

At the user's request, adapt the retailer identities and small pipeline
contracts from the cached `bodul_shared` 0.1.1 package into `lib/shared`.
Retain the original MIT notice. The registry cache is read only and unchanged.

All monorepo consumers import the same local `shared` types. Remove the old
dependency from manifests and lockfiles, along with the temporary source
override. Keep existing health and image-validation APIs. The new country
identities, historical Nordic decoding, and other wire representations remain
as described in ADR 47.

## Consequences

Retailer changes can be tested and delivered together in this repository,
without access to an external source repository or a separate package release.
If independent consumers later need these contracts, extracting a repository
and publishing it is a separate decision. No publishing is needed for this
change. Reader-before-producer rollout still applies to the new country codes
and currencies.
