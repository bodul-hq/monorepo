# inbox_client

Blocking HTTP client for a service's durable Mulac `/api/inbox`, used by
outbox publishers that deliver over HTTP. It owns only the transport; each
service decides what it submits and maps errors to its own messages.

- The destination is an HTTP(S) origin with a host and no credentials,
  non-root path, query, fragment, surrounding whitespace or control characters.
  `/api/inbox` is appended.
- Only `202 Accepted` with a JSON body whose `message_id` equals the submitted
  UUID confirms acceptance. Any other status, a redirect, or an invalid,
  mismatched or oversized (more than 64 KiB) acknowledgement fails.
- One ten-second deadline covers the request and the acknowledgement body.
- Redirects and automatic retries are disabled; callers retry through their
  outbox's schedule.
- Errors never contain the URL, credentials or response bodies.

## Usage

```toml
inbox_client = { path = "../../lib/inbox_client" }
```

```rust,no_run
use inbox_client::InboxClient;

let runtime = tokio::runtime::Runtime::new().unwrap();
// None when the variable is unset.
let client = InboxClient::from_env_value(std::env::var("OFFERS_URL"), runtime.handle().clone())
    .unwrap()
    .unwrap();
let message_id = uuid::Uuid::nil();
let submission = serde_json::json!({"payload": {}, "routing_key": "Example", "message_id": message_id});
// Blocks on the runtime; call it from a blocking thread, as Mulac's outbox does.
client.submit(&submission, message_id).unwrap();
```
