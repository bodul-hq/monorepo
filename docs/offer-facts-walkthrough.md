# Verify offer scrape time, localization and identifiers

Start customer-facing, offers, then retailer sourcing with their own databases.
Use a Mulac checkout containing the optional-sequence prerequisite. Run migrations
at startup and check each `/api/health`. Newly generated sourcing exports use v5;
queued v1–v4 commands keep their own format.

The shared contract fixtures are in
`services/retailer_sourcing/tests/fixtures/offer_export/import_payload_v{3,4,5}.json`.
Submit a fixture as the `payload` of an offers `/api/inbox` message with routing
key `ImportOfferDetails`. Use an explicit unique `message_id` and source. The
Bruno `Import via inbox` example shows all the fields.

Run these batches explicitly, each with `{"limit":100}`:

1. Offers `/api/inbox/run`, `/api/commands/run`, then `/api/events/run`.
2. Offers `/api/outbox/run` with `CUSTOMER_FACING_URL` configured.
3. Customer-facing `/api/inbox/run`, then `/api/commands/run`.

Finish event processing before submitting another import of the same retailer/URL;
identity lookup still depends on the projection. Browse customer-facing
`/api/offers` and `/graphql`. The Bruno GraphQL request selects all new fields.

Repeat the same import with a newer scrape time: only `OfferScrapedAtUpdated`
should appear. Repeat with an older time: time must stay at the newer value.
Change only localization, then explicitly set both facts to null. Both observations
must reach the projection. Omit localization in a legacy import: known facts stay.
Change only SKU: one identifier event must retain the other identifiers. Omit/null
identifiers in a later import: known values stay. Replay the same inbox message ID:
the customer-facing receipt prevents a second application.

Compare Testimony's `event_store.sequence` and `metadata.sequence` against
`event_entries.meta.sequence` and `outbox_entries.meta.sequence` in offers. A
creation starts at one; every emitted change consumes its own revision. No-op
imports do not advance the sequence. No sequence field belongs in the payload.

Run `make fmt`, root `make ci`, and each affected service's `make test-postgres`
using a dedicated `DATABASE_URL_TEST` (the tests create isolated schemas). Use
`RUST_TEST_THREADS=4` if the database's connection limit is small. Run `make audit`
when dependency manifests or lockfiles change. Database/live checks are required
before marking all planned items complete; unit tests alone are insufficient.

## Fresh locale-derived observations

Under [ADR 31](adr/0031-source-locale-market-fallback.md), a new extraction from
`<html lang="it-IT">` without explicit market evidence selects country IT and
language it. Inspect `_localization_evidence.candidates` for the original full
tag and `selected locale-derived country market` decision. Drive the ordinary
new download/extraction and explicit queue flow to observe this in customer-facing
REST/GraphQL and the customers response model. Replaying an old null/`it-IT`
export must retain those original facts; it cannot apply the new policy. A new
IT/it observation uses `OfferLocalizationChanged.v1`, an identical reimport emits
none, and an explicit null/null observation still clears both fields. No source
corpus, immutable export or event history is rewritten by this change.

Under [ADR 33](adr/0033-european-union-locale-market.md), fresh Anker EU pages
declaring `html[lang="en-EU"]` follow the same flow with market
`{"kind":"region","label":"European Union"}` and language `en`. The `/eu-en/`
URL alone is not evidence. Legacy null/`en-EU` observations remain unchanged.
