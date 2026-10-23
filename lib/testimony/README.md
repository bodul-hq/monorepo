# testimony

Event sourcing abstractions: aggregates, event envelopes, event stores,
publishers, and repositories that load and save aggregate histories. Also
provides event metadata and query dispatch through query handlers and a query
bus.

Import common types from `testimony::prelude`. Concrete event stores live in [`testimony_infra`](../testimony_infra/README.md).

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.
