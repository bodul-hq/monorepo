# Customer facing requests

Open the parent Bruno workspace, select its local environment, and set
`customerFacingBaseUrl` to the server (default `http://localhost:3003`).
Start the service with its own database as described in its
[README](../../services/customer_facing/README.md).

Health returns `ok`. Submit inbox message stores an `OfferCreated` example and
returns a message ID. Run inbox queues its command; Run commands applies the
offer. List offers and GraphQL offers then retrieve the projection, 30 records
at a time by default. Run events processes only its own queue; Run outbox
returns 503 while delivery is unconfigured. No request automatically runs
other queues. Repeated creations require fresh offer IDs and URLs; explicit
message IDs support retries of the same delivery.

Read requests accept limit 1–100 and offset 0–2147483647, defaulting to 30 and
0. Results sort newest first by creation timestamp and ID, with `has_more`
(`hasMore` in GraphQL) for navigation. Read prices encode signed i64 minor-unit
amounts as decimal strings; inbox prices remain integer JSON values.

The source and routing key must contain non-whitespace text; the payload must
be nonempty. Optional message/correlation IDs must be UUID strings. Retrying the
same message ID preserves the original row. Run requests accept integer limits
from 1 through 1000. Inspect `errors` even when a runner returns HTTP 200.
