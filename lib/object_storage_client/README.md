# object_storage_client

Async Rust client for [`object_storage`](../../services/object_storage/README.md).
Use `Transport::Rest` or `Transport::Graphql` with the same `create`, `get`,
record types, and error classifications. The client uses reqwest and a shared
connection pool; clones are inexpensive. HTTPS uses rustls and system trust.

## Usage

Add this crate as a path dependency from a service:

```toml
object_storage_client = { path = "../../lib/object_storage_client" }
```

Supply the server address and bearer token explicitly. The client does not
embed the service's development token or read configuration implicitly.

```rust,no_run
use object_storage_client::{Client, CreateRecord, Error, Transport, Uuid, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::var("OBJECT_STORAGE_TOKEN").expect("set the bearer token");
    let rest = Client::new("http://localhost:3003", &token, Transport::Rest)?;
    rest.health().await?;

    let input = CreateRecord {
        idempotency_key: Uuid::now_v7(),
        payload: "<urlset/>".into(),
        metadata: Value::Object(Default::default()),
    };
    let id = rest.create("sitemap_raw", &input).await?;

    let graphql = rest.with_transport(Transport::Graphql);
    assert_eq!(graphql.create("sitemap_raw", &input).await?, id);
    let record = graphql.get("sitemap_raw", id).await?;
    assert_eq!(record.payload, input.payload);
    Ok(())
}
```

Creation returns only the new or existing UUID. Keep the input's UUIDv7 key
across retries: a matching request returns the same record ID, and differing
content returns a conflict. The database column is `external_id`; its API names
remain `idempotency_key` in REST and `idempotencyKey` in GraphQL. The client uses
GraphQL aliases to deserialize both protocols into the same `Record` type.

`get` requires both bucket and record ID; a missing record is an error in either
transport. `health` always uses the authenticated REST health route because the
GraphQL schema has no health field. Bucket allowlist, UUIDv7, payload, and
metadata validation remain the server's responsibility.

`Value` and `Uuid` are re-exported for convenience. Metadata uses serde_json's
`arbitrary_precision` feature; parse precise numbers from JSON text instead of
first converting them to `f64`. Returned timestamps are `chrono::DateTime<Utc>`.
No request or response body-size limit is added.

## Configuration and errors

The base URL must be HTTP(S), without credentials, query parameters, or a
fragment. A path prefix is supported: `https://host/storage` uses
`/storage/api/...` and `/storage/graphql`. Path segments are URL-encoded, and
GraphQL inputs are sent as variables. Every request carries the bearer token.
Client debug output omits credentials.

`Client::new` disables redirects and automatic retries. It uses reqwest's
default timeout behavior (no overall request timeout). For application-specific
timeouts, TLS, or proxy settings, construct a `reqwest::Client` and pass it to
`Client::with_http_client(http, base_url, token, transport)`. That constructor
retains the supplied client's redirect and retry policies.

`Error::kind()` gives a common classification such as `NotFound`, `Conflict`,
`Unauthorized`, `BadUserInput`, or `ServiceUnavailable`. Additional variants
identify transport failures, malformed responses, and GraphQL request errors.
For detailed handling:

- `Error::Http` retains the HTTP status and optional RFC 9457 `ProblemDetails`.
  Non-JSON proxy errors still retain their HTTP status.
- `Error::Graphql` retains the HTTP status, every GraphQL error with its code,
  path, locations and extensions, and any partial `data`. An errors array is
  always treated as failure, even at HTTP 200. `kind()` uses its first code.
- `Error::Protocol` identifies an unexpected media type, malformed response,
  or missing operation data. A null record without a GraphQL error is invalid.
- `Error::Transport` retains the underlying reqwest error with its URL removed.

Display messages exclude response contents. Structured GraphQL partial data
can contain artifacts, so avoid logging it indiscriminately. No background
retry, automatic UUID replacement, deduplication, or sourcing integration is
performed by this crate.

## Tests

```sh
make fmt
make ci
make doc
# From lib/, after Cargo dependency changes:
make audit
```

`make ci` runs local HTTP fixture servers on ephemeral loopback ports. It tests
both wire formats, authentication, URL prefixes, numeric precision, timestamps,
error classifications and partial data, malformed responses, redirects,
connection failures, and payloads/metadata above the framework body defaults.
No PostgreSQL or external network service is needed for these tests.

Opt-in integration tests run both transports against the real service:

```sh
OBJECT_STORAGE_URL=http://localhost:3003 \
OBJECT_STORAGE_TOKEN='<service bearer token>' \
make test-live
```

It fails clearly when configuration or the service is unavailable. Use a
disposable database: the tests create four immutable artifacts. The dedicated
round-trip test creates a record through each transport and checks its ID,
bucket, idempotency key, exact Unicode payload, nested metadata, numeric
precision, and timestamp consistency across both APIs. The other test exercises
retries, conflicts, wrong-bucket/missing retrievals, and authentication failures.
Ordinary `make test` leaves these tests ignored. For manual API testing, use the
[Bruno collection](../../bruno/object_storage/README.md).
