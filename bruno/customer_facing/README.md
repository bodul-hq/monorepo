# Customer facing requests

Open the parent Bruno workspace, select its local environment, and set
`customerFacingBaseUrl` to the server (default `http://localhost:3003`).
Start the service with its own database as described in its
[README](../../services/customer_facing/README.md).

Health returns `ok`. Submit inbox message stores a development probe and returns
a message ID; Run inbox then queues its command. Run commands reports a missing
handler because production offer handlers are not installed yet. Run events
processes only its own queue; Run outbox returns 503 while delivery is
unconfigured. No request automatically runs the other queues.

The source and routing key must contain non-whitespace text; the payload must
be nonempty. Optional message/correlation IDs must be UUID strings. Retrying the
same message ID preserves the original row. Run requests accept integer limits
from 1 through 1000. Inspect `errors` even when a runner returns HTTP 200.
