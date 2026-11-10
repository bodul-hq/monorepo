# 49. Lowercase retailer serialization

Status: Accepted

## Decision

The shared `RetailerCode` Serde boundary writes lowercase slugs and accepts
both those slugs and exact historical Rust variant spellings. Unknown codes
and other casing are rejected. Historical `AnkerNordicsCom` remains readable
and writes as `ankernordicscom`, without joining the active retailer lists.

This changes all typed JSON consumers, including nested command/event payloads,
exports and API responses. Rust variant names, Debug output, slug parsing,
database retailer columns and message type names remain unchanged. Stored
messages, artifacts, migrations and historical fixtures are not rewritten.
Opaque JSON and immutable payload bytes are not implicitly normalized.

Initial-fill detail command identity must continue to use the original
PascalCase retailer spelling independently of payload serialization, so old
and new catalog events target the same command rows.

## Rollout

Upgrade consuming services/apps before starting upgraded producers. Old
binaries that only accept PascalCase cannot read lowercase payloads. Stop
producers during a coordinated rollout if needed. Existing queued messages
are readable after upgrade without a database migration. Existing rows keep
their stored casing; new serialization uses lowercase.

This supersedes the unchanged retailer serialization statement in ADR 46;
its local ownership and the Nordic market decisions in ADRs 47–48 still apply.
