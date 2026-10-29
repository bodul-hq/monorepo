# ADR 40: Canonical regional market codes

Status: Accepted; supersedes ADR 33's regional market representation.

Date: 2026-10-29

## Decision

New European Union observations use `{"kind":"region","code":"eu"}`.
RegionCode is separate from CountryCode; `EU` is not a country. Locale evidence,
precedence and language normalization remain unchanged.

Keep historical label decoding and serialization explicit through LegacyRegion.
Canonicalization maps only the exact label `European Union` to `eu`; `Europe`
and all other labels remain historical values, never guessed codes. New export
and Offers import writers reject unmapped legacy regions until an explicit
mapping is agreed; historical readers remain lossless. Unknown
new codes and ambiguous label/code objects are rejected. Legacy serialization
retains original labels; no stored event, export or outbox bytes are rewritten.

Source export writer v9 uses codes. Versions 1–8 remain readable, and explicit
regeneration of those versions uses historical label encoding. Offers writes
OfferCreated.v10 and OfferLocalizationChanged.v2; earlier versions keep their
own record variants and remain readable. Existing routing keys are unchanged.

Offers import and both projection writers canonicalize known EU labels. Current
customer-facing REST and GraphQL regional objects provide code `eu`; customers
accepts new codes and old labels. Historical unknown labels remain readable
until explicitly mapped. Both projection migrations preflight their entire
regional inventory and fail before rewriting any row if an unmapped label or
unknown code exists. Down migrations refuse unknown codes.

## Rollout and consequences

Deploy compatible readers before producer v9/v10/v2. Pause projection writers
while applying the transactional migrations. Inventory regional labels in each
active schema; agree explicit mappings for unmapped labels before migration.
Replaying old EU events produces the same canonical projection as migration.
Do not reset queues, delete receipts or rewrite immutable historical payloads.
Retire legacy readers only after an independently agreed compatibility window.
