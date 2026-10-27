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
