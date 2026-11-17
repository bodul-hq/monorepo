# ADR 66: Explicit raw-to-extracted conversions

Status: Accepted

Supersedes ADR 65's unfiltered field-preservation requirement and extends ADR 64's
brand ownership for v2 extraction.

Each brand defines v2 raw models in graphql/raw.rs, extracted models in
extracted/extracted.rs, and explicit per-type TryFrom implementations in
extracted/raw_to_extracted.rs. Conversion files contain imports and implementations
only, with no helper functions, macros, generic conversions, or JSON round trips.
Nested conversions propagate String errors. Raw validation remains strict.

Anker retains its grouped artifact and image-reference ordering. Every extracted
connection is an ordered node vector; existing Anker root connection metadata is
preserved. Acquisition time is required. Optional extracted values serialize as
null; recorded pages, explicit type-name fields, and product price ranges remain
excluded. Enum discriminators remain available for deserialization.

Legacy Nordic, HTML, download/market models, and generated query bindings are
outside this decision. Saved raw artifacts and service contracts do not change;
extracted artifacts can be regenerated from timestamped raw input.
