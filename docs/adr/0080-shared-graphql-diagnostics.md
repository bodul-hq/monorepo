# 80. Shared GraphQL diagnostics

Status: Accepted

## Decision

`retailer_sauce::graphql::check` owns the GraphQL response diagnostics previously
in `retailers_wip::graphql_errors`. Anker, Minisforum and Ugreen call it before
typed response decoding. The old module is removed without an alias.

## Consequences

The shared GraphQL module owns both request envelopes and diagnostic inspection.
Existing signatures, messages, allowed diagnostic fields and error handling remain
unchanged. Brand engines retain token redaction. Unit tests move with the helper;
brand integration tests continue to cover partial data and redaction.
Existing local path dependencies supply the helper without a release or version bump.
