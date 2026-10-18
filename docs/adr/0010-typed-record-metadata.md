# ADR 10: Typed record metadata

- Status: Accepted
- Date: 2026-10-15
- Supersedes: ADR 9's unstructured Rust JSON values for record metadata

## Context

Pipeline records store known metadata shapes in JSONB, but `serde_json::Value`
allows missing keys, misspelled fields, and values of the wrong type to reach
record construction. The metadata contract should be explicit in Rust as well
as in the documented JSON shape.

## Decision

Use Serde structs for the metadata introduced in ADRs 8 and 9:

- Shared `Storage` contains `content_size: u64`.
- Consolidation's private `References` contains two UUIDs and a `RetailerCode`.
  Its serializer retains the existing lowercase retailer-code strings.
- Consolidation's private `Stats` contains the five named counts as `u32`.

The database layer supplies `Json<T>`, a Diesel JSONB adapter that serializes
and deserializes the struct directly. All metadata fields are required and
unknown fields are rejected. Invalid stored metadata fails deserialization
before a typed record is returned. Stage-specific size and count checks retain
the previous signed-integer limits required for downgrade compatibility.

These types cover record metadata; payload document representations are
unchanged. Storage metadata is shared technical plumbing. References and
stats remain private to the consolidation store.

## Consequences

- Record constructors use named Rust fields and concrete types. Metadata is
  never assembled or accessed through dynamic JSON keys in application code.
- Existing valid JSONB data and SQL schemas remain compatible; no migration is
  required. Missing, extra, null, or wrongly typed metadata fields now fail
  typed reads, as do invalid UUIDs and unknown retailer codes.
- PostgreSQL tests verify typed round trips, the unchanged serialized shapes,
  invalid-field rejection, and existing transactional storage behavior.
