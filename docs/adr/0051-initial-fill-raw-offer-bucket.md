# ADR 51: Initial fill uses the raw offer bucket

- Status: Accepted
- Date: 2026-11-09
- Supersedes: The `initial_offer_fill_details` bucket choice in ADR 50

## Decision

Store initial-fill GraphQL product artifacts in the existing `offer_details_raw`
bucket. Object storage accepts this bucket; it rejects the previously selected
`initial_offer_fill_details` name because that name is absent from its allowlist.

Use the same bucket when reading artifacts referenced by `OfferDetailsFetched`.
The artifact still contains `schema_version`, the assembled `product`, and raw
request/response `pages`. Its UUIDv7 idempotency key and record ID are unchanged.
The terminal event still has no subscriber, so these artifacts are not fed into
the legacy HTML extraction pipeline merely because they share a bucket.

No object-storage configuration or database migration is required. After
deploying sourcing, retry failed detail commands through the existing queue.
