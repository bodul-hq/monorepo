#![doc = include_str!("../README.md")]

//! Async client for the object storage service, with interchangeable REST and
//! GraphQL transports. Both transports share record types and error kinds.
//! Callers supply and retain UUIDv7 idempotency keys; operations are not retried
//! automatically. See the crate README for setup and runnable examples.
mod error;

use chrono::{DateTime, Utc};
pub use error::{Error, ErrorKind, ErrorLocation, GraphqlError, ProblemDetails};
use reqwest::{
    Client as HttpClient, RequestBuilder, Url,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderValue},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
pub use serde_json::Value;
use serde_json::json;
pub use uuid::Uuid;

/// Selects the wire protocol for creation and retrieval. Health is always REST.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Rest,
    Graphql,
}

/// Reuse the entire input, including its UUIDv7 key, when retrying a creation.
/// The service stores `idempotency_key` in its `records.external_id` column.
#[derive(Debug, Clone, Serialize)]
pub struct CreateRecord {
    pub idempotency_key: Uuid,
    pub payload: String,
    pub metadata: Value,
}

/// Immutable API-visible record. Internal data IDs are never exposed.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Record {
    pub id: Uuid,
    pub bucket: String,
    pub idempotency_key: Uuid,
    pub payload: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct Created {
    id: Uuid,
}
#[derive(Deserialize)]
struct GraphqlEnvelope {
    data: Option<Value>,
    #[serde(default)]
    errors: Vec<GraphqlError>,
}

const CREATE: &str =
    "mutation CreateRecord($input: CreateRecordInput!) { createRecord(input: $input) { id } }";
const GET: &str = "query GetRecord($bucket: String!, $id: ID!) { record(bucket: $bucket, id: $id) { id bucket idempotency_key: idempotencyKey payload metadata created_at: createdAt } }";

/// Cloneable client. Its HTTP connection pool is shared between clones.
/// Credentials are marked sensitive and excluded from its Debug output.
#[derive(Clone)]
pub struct Client {
    http: HttpClient,
    base_url: Url,
    authorization: HeaderValue,
    transport: Transport,
}
impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("transport", &self.transport)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Creates a pooled HTTP client with redirects and automatic retries disabled.
    /// HTTP and HTTPS are supported. A base-path prefix is preserved.
    pub fn new(base_url: &str, token: &str, transport: Transport) -> Result<Self, Error> {
        let http = HttpClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .build()?;
        Self::with_http_client(http, base_url, token, transport)
    }

    /// Uses a caller-configured HTTP client (for timeouts, TLS, or proxies).
    /// Its redirect and retry policies apply; configure these deliberately.
    pub fn with_http_client(
        http: HttpClient,
        base_url: &str,
        token: &str,
        transport: Transport,
    ) -> Result<Self, Error> {
        let base_url = Url::parse(base_url)
            .map_err(|_| Error::Configuration("base URL must be an absolute HTTP(S) URL"))?;
        if !matches!(base_url.scheme(), "http" | "https")
            || base_url.host_str().is_none()
            || !base_url.username().is_empty()
            || base_url.password().is_some()
            || base_url.query().is_some()
            || base_url.fragment().is_some()
        {
            return Err(Error::Configuration(
                "base URL must use HTTP(S), without credentials, query, or fragment",
            ));
        }
        if token.is_empty() || token.chars().any(char::is_whitespace) {
            return Err(Error::Configuration(
                "bearer token must be non-empty and contain no whitespace",
            ));
        }
        let mut authorization = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| Error::Configuration("bearer token contains invalid header characters"))?;
        authorization.set_sensitive(true);
        Ok(Self {
            http,
            base_url,
            authorization,
            transport,
        })
    }

    /// Creates a client using the same configuration and pool but another protocol.
    pub fn with_transport(&self, transport: Transport) -> Self {
        Self {
            transport,
            ..self.clone()
        }
    }

    fn endpoint(&self, segments: &[&str]) -> Url {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("HTTP(S) base URL")
            .pop_if_empty()
            .extend(segments);
        url
    }
    fn authorize(&self, request: RequestBuilder, accept: &'static str) -> RequestBuilder {
        request
            .header(AUTHORIZATION, self.authorization.clone())
            .header(ACCEPT, accept)
    }

    /// Liveness check through `/api/health`, regardless of selected transport.
    pub async fn health(&self) -> Result<(), Error> {
        let response = self
            .authorize(
                self.http.get(self.endpoint(&["api", "health"])),
                "text/plain, application/problem+json",
            )
            .send()
            .await?;
        let status = response.status().as_u16();
        let body = response.bytes().await?;
        if status != 200 {
            return Err(http_error(status, &body));
        }
        if body.as_ref() != b"ok" {
            return Err(Error::Protocol {
                status,
                detail: "expected health response 'ok'",
            });
        }
        Ok(())
    }

    /// Returns the new or existing record ID. Matching retries return the same
    /// ID; differing content yields [`ErrorKind::Conflict`] in either protocol.
    pub async fn create(&self, bucket: &str, input: &CreateRecord) -> Result<Uuid, Error> {
        let created: Created = match self.transport {
            Transport::Rest => {
                let request = self
                    .http
                    .post(self.endpoint(&["api", "buckets", bucket, "records"]))
                    .json(input);
                self.rest(request, &[200, 201]).await?
            }
            Transport::Graphql => {
                self.graphql(
                    CREATE,
                    "CreateRecord",
                    json!({"input": {
                        "bucket":bucket, "idempotencyKey":input.idempotency_key,
                        "payload":input.payload, "metadata":input.metadata,
                    }}),
                    "createRecord",
                )
                .await?
            }
        };
        Ok(created.id)
    }

    /// Retrieves a record matching both bucket and ID. Missing records yield
    /// [`ErrorKind::NotFound`] in either protocol.
    pub async fn get(&self, bucket: &str, id: Uuid) -> Result<Record, Error> {
        match self.transport {
            Transport::Rest => {
                let request = self.http.get(self.endpoint(&[
                    "api",
                    "buckets",
                    bucket,
                    "records",
                    &id.to_string(),
                ]));
                self.rest(request, &[200]).await
            }
            Transport::Graphql => {
                self.graphql(GET, "GetRecord", json!({"bucket":bucket,"id":id}), "record")
                    .await
            }
        }
    }

    async fn rest<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
        expected: &[u16],
    ) -> Result<T, Error> {
        let response = self
            .authorize(request, "application/json, application/problem+json")
            .send()
            .await?;
        let status = response.status().as_u16();
        let media = media_type(&response);
        let body = response.bytes().await?;
        if !expected.contains(&status) {
            return Err(http_error(status, &body));
        }
        if media != "application/json" {
            return Err(Error::Protocol {
                status,
                detail: "expected application/json",
            });
        }
        decode(&body, status)
    }

    async fn graphql<T: DeserializeOwned>(
        &self,
        query: &str,
        operation_name: &str,
        variables: Value,
        field: &str,
    ) -> Result<T, Error> {
        let request = self
            .http
            .post(self.endpoint(&["graphql"]))
            .json(&json!({"query":query,"operationName":operation_name,"variables":variables}));
        let response = self
            .authorize(
                request,
                "application/graphql-response+json, application/problem+json",
            )
            .send()
            .await?;
        let status = response.status().as_u16();
        let media = media_type(&response);
        let body = response.bytes().await?;
        if media == "application/problem+json" || !matches!(status, 200 | 400) {
            return Err(http_error(status, &body));
        }
        if media != "application/graphql-response+json" {
            return Err(Error::Protocol {
                status,
                detail: "expected application/graphql-response+json",
            });
        }
        let envelope: GraphqlEnvelope = decode(&body, status)?;
        if !envelope.errors.is_empty() {
            return Err(Error::Graphql {
                status,
                errors: envelope.errors,
                data: envelope.data,
            });
        }
        if status != 200 {
            return Err(Error::Protocol {
                status,
                detail: "GraphQL request failure omitted errors",
            });
        }
        let value = envelope
            .data
            .and_then(|mut data| data.as_object_mut()?.remove(field))
            .filter(|value| !value.is_null())
            .ok_or(Error::Protocol {
                status,
                detail: "GraphQL response omitted operation data",
            })?;
        serde_json::from_value(value).map_err(|_| Error::Protocol {
            status,
            detail: "invalid operation response",
        })
    }
}

fn media_type(response: &reqwest::Response) -> String {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}
fn decode<T: DeserializeOwned>(body: &[u8], status: u16) -> Result<T, Error> {
    serde_json::from_slice(body).map_err(|_| Error::Protocol {
        status,
        detail: "invalid JSON response",
    })
}
fn http_error(status: u16, body: &[u8]) -> Error {
    Error::Http {
        status,
        problem: serde_json::from_slice(body).ok(),
    }
}
