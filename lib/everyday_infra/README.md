# everyday_infra

Infrastructure adapters for [`everyday`](../everyday/README.md) value types.
Currently provides `AppDateTime`, a chrono UTC date/time wrapper that converts
to and from `everyday::DateTime` and serializes as an ISO 8601 string with
serde.

Import the wrapper from `everyday_infra::prelude`. Unit tests cover domain
conversions and serialization without external services.

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.
