# everyday

Common value types for business and time: email addresses, phone numbers and
parsers, URLs, money, working hours, dates, UTC timestamps, weekdays, country
codes, and geographic coordinates.

Import common types from `everyday::prelude`. The `DateTime` type represents
whole seconds from the Unix epoch and is used by `testimony` event envelopes.
External date/time conversions live in [`everyday_infra`](../everyday_infra/README.md).

This crate is part of the `lib` workspace. See the
[root README](../../README.md) for requirements and Make targets.
