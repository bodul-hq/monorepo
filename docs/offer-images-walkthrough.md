# Offer image walkthrough

Use isolated test databases, one per service. Do not reset operational queues.
Deploy customers, customer-facing, offers, then retailer sourcing. The consumers
must apply their additive image migrations before images are published. Retain
v1 exports and all original-message receipts; never roll offers back to a binary
that cannot read `OfferCreated.v2` and `OfferImageAdded.v1` history.

Start customer-facing with `PORT=3003 make serve`, offers with
`CUSTOMER_FACING_URL=http://localhost:3003 PORT=3001 make serve`, sourcing with
`OFFERS_URL=http://localhost:3001 PORT=3002 make serve` (plus its object-storage
configuration), and customers with
`CUSTOMER_FACING_URL=http://localhost:3003 PORT=3004 make serve` in their respective
folders. All services expose `/api/health`; customers serves `/`.

For a real sourcing input, retrieve and structuralize an image-bearing retailer
page through the documented sourcing queues. A new `OfferWasStructuralized`
event queues `ProcessOfferForExport` with schema 2. Run sourcing events,
commands, events and outbox to prepare the immutable export, record its messages
and deliver them to offers. The v2 import payload includes the variation's
filtered ordered images; old queued export commands still select v1.

For a deterministic downstream check, submit directly to offers:

```sh
curl http://localhost:3001/api/tmp-import \
  -H 'content-type: application/json' \
  -d '{"retailer_code":"ankerau","title":"USB charger","price_amount":1999,"currency":"AUD","url":"https://shop.example/image-demo","images":["https://shop.example/A.jpg","https://shop.example/B.jpg"]}'
for queue in inbox commands events outbox; do
  curl "http://localhost:3001/api/$queue/run" \
    -H 'content-type: application/json' -d '{"limit":100}'
done
for queue in inbox commands; do
  curl "http://localhost:3003/api/$queue/run" \
    -H 'content-type: application/json' -d '{"limit":100}'
done
curl http://localhost:3003/api/offers
curl http://localhost:3003/graphql -H 'content-type: application/json' \
  -d '{"query":"{ offers { items { id title images } } }"}'
curl http://localhost:3004/
```

The first import produces one `OfferCreated` with `images: [A,B]` and no
`OfferImageAdded` events. Both projections save the parent and images together.
Both APIs return `[A,B]`, and the HTML contains only A in `img src`. The example
URLs are placeholders; use a controlled image server to verify successful loads.
Repeat the import with the same offer URL and `[B,C]`, finishing all queues in
order. This update emits one `OfferImageAdded` for C at position 2. Expect `[A,B,C]` and A still shown. Empty/reordered imports are no-ops for
images. Invalid URL strings are skipped; malformed arrays fail processing.

The current offers handler relies on its asynchronously projected identity.
Complete its events before the next import for that identity. This item does
not add durable offers command receipts or close its history/publication crash
window; that scope was explicitly excluded by the user.

To exercise legacy or delayed-update receiver ordering in a test schema, submit a parent `OfferCreated`,
then image position 1 before position 0 through customer-facing's inbox. The
first read temporarily contains B; after position 0 arrives it returns `[A,B]`.
Resubmit identical additions under new message IDs and replay original inbox
messages: images remain unique and receipts prevent repeated effects. A missing
parent or a conflicting position/URL must fail without claiming a receipt.

Inspect `read_offer_images` in offers and `offer_images` in customer-facing using
`ORDER BY offer_id, position`; both projections should agree after queues finish.
Review desktop/mobile light/dark pages with loaded, missing and failed images.
Keep price and purchase links usable in all three cases.

See [item 11's verification record](plans/11-offer-images.md) for actual commands,
revisions, test results and the local fixture run.
