# 65. Shared complete Anker extraction

Status: Accepted

Extends ADR 64 with shared extracted-model ownership for Anker. Supersedes its
preservation of separate AU/CA extracted formats.

All supported Anker storefronts use one typed extracted model and conversion.
Retain the AU/CA grouping, root variant/collection/image arrays, and every raw
field. Preserve connection metadata and reference images by root index plus
optional ID. Retain decimal/timestamp strings and optional acquisition time;
no filtering, inference, currency conversion or additional completeness checks
are performed. Raw validation remains unchanged. Cleanup will be a later change.

The Anker v2 extracted output changes, including AU/CA. Saved raw artifacts and
service commands/events are unchanged; regenerate extracted dumps from raw
artifacts when needed. HTML v1 and other brands remain unchanged.
