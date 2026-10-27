# Sitemap fetch delay

Status: implemented on 2026-10-27. The user selected replacement of the existing pause after
every three downloads, rather than stacking the two policies.

## Implementation

Define bounds of one and five seconds. Sample uniformly from inclusive whole
seconds using `fastrand` 2.5, already present transitively in the services lockfile.
Declare it directly for retailer sourcing and keep dependencies alphabetized.

In the synchronous crawler, sleep immediately before every real fetch attempt
after the first, following all skip checks. Failed HTTP requests and responses
that fail parsing count as attempts. Do not sleep before the first fetch, after
the final fetch, or for duplicate, greylisted, cyclic or depth-rejected URLs.
Preserve traversal order, error handling, persistence and events. Pacing is
per crawl; multiple workers can crawl concurrently.

Use an internal injectable pause callback so tests do not wait. Remove the
service's old batch counter/pause configuration plumbing while retaining public
retailer-library pause functions for compatibility. Update the service README
and retailer integration playbook to explain the replacement policy.

## Validation

For N actual requests assert exactly N−1 pauses and verify ordering across roots
and nested children, zero/one request, fetch/parse failures and all skip paths.
Verify inclusive bounds and whole-second durations without wall-clock tests.
Retain document/event/storage assertions. Run fmt/CI and the services dependency
audit, recording new advisories. Check live health and a controlled sitemap
fixture when runtime prerequisites are available. No migration or event change
is needed.

## Completion

Per-attempt pacing replaces batch pauses, with injectable tests for traversal,
failures, skip paths and duration bounds. The direct fastrand dependency reuses
the locked version; the audit comparison found no new advisories. See the
[verification record](28-31-verification.md).
