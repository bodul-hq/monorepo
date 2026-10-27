# Offers requests

Open the monorepo's `bruno` workspace and select the global `local` environment.
It sets `offersBaseUrl` to `http://172.28.100.41:3000`. Select the `offers`
collection's `local` environment for the sample `retailerCode` and `offerUrl`.
The collection does not override the global base URL.

The offers service requires a reachable `DATABASE_URL` at startup, including
for health and OpenAPI. To test a service started with `PORT=3001 make serve`,
override the global `offersBaseUrl` with `http://localhost:3001`.

For a quick test, send **Import directly (testing)**, which stores the import in
the inbox, then **Run inbox**, **Run commands** and **Run events**.
The temporary endpoint validates the import and returns `202` with the inbox
`message_id`; nothing is executed until the queues run.

## Manual queue processing

Send **Submit inbox message** to persist a generic envelope through
`POST /api/inbox`. It returns `202` with a `message_id` and leaves all processing
for the run requests. The sample payload is the serialized `ImportOfferDetails`
command, with enum retailer name `AnkerAu`, `price.amount` and `price.currency`.
The direct import request uses retailer slug `ankerau` and `price_amount`.

`payload` is required and must be nonempty: `null`, blank strings, empty arrays
and empty objects are rejected, while numbers (including zero), booleans and
nonempty containers are accepted. `routing_key` and `source` must both contain
non-whitespace text; unknown routing keys are accepted. Optional `message_id`
and `correlation_id` must be UUID strings. Omit `message_id` to generate one.
Reusing an ID succeeds while preserving the original row, even if the new
content differs. Invalid envelopes return `400`; storage or blocking-task
failures return a sanitized `500`.

After submission, send these requests in order:

1. **Run inbox** — converts eligible stored inbox messages into queued commands;
   it does not fetch messages from a broker.
2. **Run commands** — executes eligible queued commands and queues their events.
3. **Run events** — dispatches eligible events to the read model projectors.

**Run outbox** delivers one batch to customer-facing's `/api/inbox` when offers
runs with `CUSTOMER_FACING_URL`. Without it, the request returns `503` and does
not reserve, modify, or deliver rows. A `202` from customer-facing means stored
only; run its inbox, commands and events afterwards. See
[HTTP delivery to customer-facing](../../services/offers/README.md#http-delivery-to-customer-facing).

All four run endpoints accept JSON with an integer `limit` from 1 through 1000.
Missing or invalid limits and malformed JSON return `400`. Each request runs
one batch from one queue; nothing processes queues in the background.

Inbox, commands, events, and a configured outbox return `200` with
`{"processed":N,"errors":[]}` on success, including `N=0` when no work is
eligible. Consumer failures return `200` with `{"processed":null,"errors":["…"]}`;
inspect `errors` even when the HTTP status is successful. Successful items in a partially failed batch remain
committed. Retry scheduling and reservations still apply. A failed blocking
task returns `500`. These operational routes have no authentication.

**Queue offer import** and **Look up offer** are retained requests for planned
routes; `/api/offers/import` and `/api/offers` are not active in the current
server. Use the direct import and event run requests for the current flow.
Prices are integer minor units: `999` AUD means AUD 9.99.

These are manual requests with no JavaScript scripts or assertions. For the
service's automated checks, see
[the offers README](../../services/offers/README.md#checks).
