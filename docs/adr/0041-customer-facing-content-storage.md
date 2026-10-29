# ADR 41: Customer-facing content storage and selected-language search

Status: Accepted; supersedes ADR 38's scalar duplication in customer-facing and the title-only search policy in ADR 30.

Date: 2026-10-29

## Decision

Customer-facing stores one private `content` envelope containing required nullable
`selected_language` and nonempty `entries`. Each entry has language, title,
nullable description and URL. Language keys remain exact and unique; selection
must identify an entry. The scalar title, description, language and URL columns
are removed. UUID entity identity remains unchanged; retailer/selected-URL
uniqueness replaces retailer/scalar-URL uniqueness.

Existing URLs are copied to every existing entry. Existing creation messages
supply the URL for every initial entry. Content snapshots preserve URLs by
language; newly observed languages inherit the currently selected URL until an
upstream contract provides language-specific destinations. Historical language
moves carry the selected entry's URL even when replacing a destination language.
No locale-based URL inference or upstream event rewrite is introduced.

All content mutations use the inbox receipt transaction and row locking. The
public content remains an array with per-entry URLs; flat title, URL and language
are derived for compatible clients. Customers accepts old arrays without URLs by
using the flat offer URL, and validates selected title/URL consistency.

Only the selected language participates in fuzzy search. Either title or
description must satisfy the existing pg_trgm word threshold of 0.5. Ranking is
`5 * word_similarity(query,title) + word_similarity(query,description)`, with
missing description contributing zero, then creation time and UUID descending.
An exact-match flag does not override this weighting. Selected JSON expression
indexes support both predicates; pagination and query timeout remain unchanged.

## Rollout and recovery

Deploy the compatible customers reader first. Pause customer-facing readers and
projection writers for the migration and binary switch; the old binary cannot
operate on the dropped-column schema. Preflight selected/scalar disagreement
and validate the new envelope transactionally before dropping columns.

Rollback reconstructs the selected scalar tuple and old list. It refuses to run
when an unselected URL differs from the selected URL, because the old schema
cannot preserve that information. Backups and projection reconstruction remain
available recovery paths; never reset queues or delete receipts.
