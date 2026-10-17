use axum::{
    Router,
    body::{Body, to_bytes},
    extract::Request,
    http::{HeaderMap, Response},
};
use object_storage_client::{Client, CreateRecord, Error, ErrorKind, Transport, Uuid, Value};
use serde_json::json;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

const TOKEN: &str = "client-test-credential";
const ID: &str = "019a1234-5678-7000-8000-000000000001";
const KEY: &str = "019a1234-5678-7000-8000-000000000002";

struct Reply {
    status: u16,
    media: &'static str,
    body: String,
    location: Option<String>,
}
impl Reply {
    fn json(status: u16, media: &'static str, body: Value) -> Self {
        Self {
            status,
            media,
            body: body.to_string(),
            location: None,
        }
    }
}
#[derive(Debug)]
struct Captured {
    method: String,
    path: String,
    headers: HeaderMap,
    body: Vec<u8>,
}
struct Server {
    url: String,
    requests: Arc<Mutex<Vec<Captured>>>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Server {
    async fn new(replies: Vec<Reply>) -> Self {
        let replies = Arc::new(Mutex::new(VecDeque::from(replies)));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let router = Router::new().fallback(move |request: Request| {
            let replies = replies.clone();
            let captured = captured.clone();
            async move {
                let (parts, body) = request.into_parts();
                let body = to_bytes(body, usize::MAX).await.unwrap().to_vec();
                captured.lock().unwrap().push(Captured {
                    method: parts.method.to_string(),
                    path: parts.uri.to_string(),
                    headers: parts.headers,
                    body,
                });
                let reply = replies
                    .lock()
                    .unwrap()
                    .pop_front()
                    .expect("unexpected HTTP request (or automatic retry)");
                let mut response = Response::builder()
                    .status(reply.status)
                    .header("content-type", reply.media);
                if let Some(location) = reply.location {
                    response = response.header("location", location);
                }
                response.body(Body::from(reply.body)).unwrap()
            }
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            url,
            requests,
            task,
        }
    }
    fn client(&self, transport: Transport) -> Client {
        Client::new(&self.url, TOKEN, transport).unwrap()
    }
}
fn input() -> CreateRecord {
    CreateRecord { idempotency_key: Uuid::parse_str(KEY).unwrap(), payload: "Živjo 雪 🌍\n".into(),
        metadata: serde_json::from_str(r#"{"integer":123456789012345678901234567890,"decimal":0.123456789012345678901234567890,"array":[1,2]}"#).unwrap() }
}
fn record() -> Value {
    let input = input();
    json!({"id":ID,"bucket":"sitemap_raw","idempotency_key":KEY,"payload":input.payload,"metadata":input.metadata,"created_at":"2026-10-15T12:34:56.123456Z"})
}
fn problem(status: u16) -> Value {
    json!({"type":"about:blank","title":"Test error","status":status,"detail":"Sanitized failure."})
}

#[tokio::test]
async fn rest_sends_auth_and_strict_body_and_preserves_precision() {
    let server = Server::new(vec![
        Reply::json(201, "application/json", json!({"id":ID})),
        Reply::json(200, "application/json; charset=utf-8", json!({"id":ID})),
        Reply::json(200, "application/json", record()),
    ])
    .await;
    let client = Client::new(&(server.url.clone() + "/storage/"), TOKEN, Transport::Rest).unwrap();
    let input = input();
    let id = client.create("sitemap_raw", &input).await.unwrap();
    assert_eq!(id.to_string(), ID);
    assert_eq!(client.create("sitemap_raw", &input).await.unwrap(), id);
    let stored = client.get("sitemap_raw", id).await.unwrap();
    assert_eq!(stored.metadata, input.metadata);
    assert_eq!(stored.payload, input.payload);
    assert_eq!(
        stored
            .created_at
            .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
        "2026-10-15T12:34:56.123456Z"
    );
    let requests = server.requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].path, "/storage/api/buckets/sitemap_raw/records");
    assert_eq!(
        requests[2].path,
        format!("/storage/api/buckets/sitemap_raw/records/{ID}")
    );
    assert_eq!(requests[2].method, "GET");
    for request in requests.iter() {
        assert_eq!(request.headers["authorization"], format!("Bearer {TOKEN}"));
    }
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].headers["content-type"], "application/json");
    let sent: Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(
        sent,
        json!({"idempotency_key":KEY,"payload":input.payload,"metadata":input.metadata})
    );
    assert_eq!(requests[0].body, requests[1].body);
    assert!(!format!("{client:?}").contains(TOKEN));
}

#[tokio::test]
async fn graphql_uses_variables_aliases_and_the_same_record_type() {
    let server = Server::new(vec![
        Reply::json(
            200,
            "application/graphql-response+json",
            json!({"data":{"createRecord":{"id":ID}}}),
        ),
        Reply::json(
            200,
            "application/graphql-response+json",
            json!({"data":{"record":record()}}),
        ),
    ])
    .await;
    let client = server
        .client(Transport::Rest)
        .with_transport(Transport::Graphql);
    let input = input();
    let id = client.create("sitemap_raw", &input).await.unwrap();
    let stored = client.get("sitemap_raw", id).await.unwrap();
    assert_eq!(stored.metadata, input.metadata);
    assert_eq!(stored.idempotency_key, input.idempotency_key);
    assert_eq!(stored.payload, input.payload);
    let requests = server.requests.lock().unwrap();
    for request in requests.iter() {
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/graphql");
        assert_eq!(request.headers["authorization"], format!("Bearer {TOKEN}"));
        assert!(
            request.headers["accept"]
                .to_str()
                .unwrap()
                .contains("application/graphql-response+json")
        );
    }
    let create: Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(create["operationName"], "CreateRecord");
    assert_eq!(
        create["variables"]["input"],
        json!({"bucket":"sitemap_raw","idempotencyKey":KEY,"payload":input.payload,"metadata":input.metadata})
    );
    assert!(!create["query"].as_str().unwrap().contains(&input.payload));
    let get: Value = serde_json::from_slice(&requests[1].body).unwrap();
    assert_eq!(get["operationName"], "GetRecord");
    assert_eq!(get["variables"], json!({"bucket":"sitemap_raw","id":ID}));
    assert!(
        get["query"]
            .as_str()
            .unwrap()
            .contains("idempotency_key: idempotencyKey")
    );
    assert!(
        get["query"]
            .as_str()
            .unwrap()
            .contains("created_at: createdAt")
    );
}

#[tokio::test]
async fn health_authenticates_even_with_graphql_transport() {
    let server = Server::new(vec![Reply {
        status: 200,
        media: "text/plain",
        body: "ok".into(),
        location: None,
    }])
    .await;
    server.client(Transport::Graphql).health().await.unwrap();
    let requests = server.requests.lock().unwrap();
    assert_eq!(requests[0].path, "/api/health");
    assert_eq!(
        requests[0].headers["authorization"],
        format!("Bearer {TOKEN}")
    );
}

#[tokio::test]
async fn both_transports_handle_problem_details_and_non_json_http_errors() {
    for transport in [Transport::Rest, Transport::Graphql] {
        for (status, expected) in [
            (400, ErrorKind::BadUserInput),
            (401, ErrorKind::Unauthorized),
            (404, ErrorKind::NotFound),
            (409, ErrorKind::Conflict),
            (415, ErrorKind::BadUserInput),
            (500, ErrorKind::InternalServerError),
            (503, ErrorKind::ServiceUnavailable),
        ] {
            let server = Server::new(vec![Reply::json(
                status,
                "application/problem+json",
                problem(status),
            )])
            .await;
            let error = server
                .client(transport)
                .create("sitemap_raw", &input())
                .await
                .unwrap_err();
            assert_eq!(error.kind(), expected);
            match error {
                Error::Http {
                    status: actual,
                    problem: Some(problem),
                } => {
                    assert_eq!(actual, status);
                    assert_eq!(problem.detail, "Sanitized failure.");
                }
                _ => panic!("expected Problem Details"),
            }
        }
        let server = Server::new(vec![Reply {
            status: 502,
            media: "text/html",
            body: "<h1>Gateway failure</h1>".into(),
            location: None,
        }])
        .await;
        assert!(matches!(
            server
                .client(transport)
                .create("sitemap_raw", &input())
                .await,
            Err(Error::Http {
                status: 502,
                problem: None
            })
        ));
    }
}

#[tokio::test]
async fn graphql_errors_never_become_false_successes() {
    for (status, code, kind) in [
        (200, "NOT_FOUND", ErrorKind::NotFound),
        (200, "CONFLICT", ErrorKind::Conflict),
        (200, "BAD_USER_INPUT", ErrorKind::BadUserInput),
        (200, "SERVICE_UNAVAILABLE", ErrorKind::ServiceUnavailable),
        (200, "INTERNAL_SERVER_ERROR", ErrorKind::InternalServerError),
        (400, "GRAPHQL_PARSE_FAILED", ErrorKind::GraphqlParseFailed),
        (
            400,
            "GRAPHQL_VALIDATION_FAILED",
            ErrorKind::GraphqlValidationFailed,
        ),
        (
            400,
            "OPERATION_RESOLUTION_FAILURE",
            ErrorKind::OperationResolutionFailure,
        ),
        (400, "BAD_USER_INPUT", ErrorKind::BadUserInput),
        (200, "FUTURE_CODE", ErrorKind::UnexpectedResponse),
    ] {
        let body = json!({"data":{"record":null,"other":"partial"},"errors":[{"message":"Failure","path":["record"],"locations":[{"line":1,"column":5}],"extensions":{"code":code,"extra":true}}]});
        let server = Server::new(vec![Reply::json(
            status,
            "application/graphql-response+json",
            body,
        )])
        .await;
        let error = server
            .client(Transport::Graphql)
            .get("sitemap_raw", Uuid::parse_str(ID).unwrap())
            .await
            .unwrap_err();
        assert_eq!(error.kind(), kind);
        match error {
            Error::Graphql {
                status: actual,
                errors,
                data,
            } => {
                assert_eq!(actual, status);
                assert_eq!(errors[0].code(), Some(code));
                assert_eq!(errors[0].path, vec![json!("record")]);
                assert_eq!(errors[0].locations[0].column, 5);
                assert_eq!(data.unwrap()["other"], "partial");
            }
            _ => panic!("expected GraphQL details"),
        }
    }
    let server = Server::new(vec![Reply::json(400,"application/graphql-response+json",json!({"errors":[{"message":"Invalid document","extensions":{"code":"GRAPHQL_PARSE_FAILED"}}]}))]).await;
    assert_eq!(
        server
            .client(Transport::Graphql)
            .create("sitemap_raw", &input())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::GraphqlParseFailed
    );
}

#[tokio::test]
async fn rejects_malformed_or_missing_success_data() {
    for (transport, media, bodies) in [
        (
            Transport::Rest,
            "application/json",
            vec!["{", "{}", "{\"id\":\"invalid\"}"],
        ),
        (
            Transport::Graphql,
            "application/graphql-response+json",
            vec![
                "{",
                "{}",
                "{\"data\":null}",
                "{\"data\":{\"createRecord\":null}}",
                "{\"data\":{\"createRecord\":{\"id\":\"invalid\"}}}",
            ],
        ),
    ] {
        for body in bodies {
            let server = Server::new(vec![Reply {
                status: 200,
                media,
                body: body.into(),
                location: None,
            }])
            .await;
            let error = server
                .client(transport)
                .create("sitemap_raw", &input())
                .await
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::UnexpectedResponse);
        }
        let server = Server::new(vec![Reply::json(200, "text/plain", json!({"id":ID}))]).await;
        assert_eq!(
            server
                .client(transport)
                .create("sitemap_raw", &input())
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::UnexpectedResponse
        );
    }
}

#[test]
fn validates_configuration_without_echoing_credentials() {
    for url in [
        "not a url",
        "file:///tmp/data",
        "https://user:password@example.test",
        "https://example.test?token=secret",
        "https://example.test/#fragment",
    ] {
        let error = Client::new(url, TOKEN, Transport::Rest).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Configuration);
        assert!(!error.to_string().contains("password"));
        assert!(!error.to_string().contains(TOKEN));
    }
    for token in ["", "token\nsecret", "two words", "token\rsecret"] {
        assert_eq!(
            Client::new("https://example.test", token, Transport::Rest)
                .unwrap_err()
                .kind(),
            ErrorKind::Configuration
        );
    }
}

#[tokio::test]
async fn does_not_follow_redirects_or_retry_failed_mutations() {
    let target = Server::new(vec![]).await;
    let server = Server::new(vec![Reply {
        status: 307,
        media: "text/plain",
        body: String::new(),
        location: Some(target.url.clone()),
    }])
    .await;
    let error = server
        .client(Transport::Rest)
        .create("sitemap_raw", &input())
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Http { status: 307, .. }));
    assert!(target.requests.lock().unwrap().is_empty());
    assert_eq!(server.requests.lock().unwrap().len(), 1);
    let server = Server::new(vec![Reply::json(
        503,
        "application/problem+json",
        problem(503),
    )])
    .await;
    assert_eq!(
        server
            .client(Transport::Rest)
            .create("sitemap_raw", &input())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::ServiceUnavailable
    );
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn preserves_large_bodies_in_both_directions() {
    for transport in [Transport::Rest, Transport::Graphql] {
        let mut input = input();
        input.payload = "雪".repeat(800_000);
        input.metadata = json!({"large":"m".repeat(2_200_000)});
        let mut stored = record();
        stored["payload"] = json!(input.payload);
        stored["metadata"] = input.metadata.clone();
        let (created, record, media) = match transport {
            Transport::Rest => (json!({"id":ID}), stored, "application/json"),
            Transport::Graphql => (
                json!({"data":{"createRecord":{"id":ID}}}),
                json!({"data":{"record":stored}}),
                "application/graphql-response+json",
            ),
        };
        let server = Server::new(vec![
            Reply::json(200, media, created),
            Reply::json(200, media, record),
        ])
        .await;
        let client = server.client(transport);
        let id = client.create("sitemap_raw", &input).await.unwrap();
        let stored = client.get("sitemap_raw", id).await.unwrap();
        assert_eq!(stored.payload, input.payload);
        assert_eq!(stored.metadata, input.metadata);
        assert!(server.requests.lock().unwrap()[0].body.len() > 4_000_000);
    }
}

#[tokio::test]
async fn connection_failures_are_transport_errors() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let error = Client::new(&url, TOKEN, Transport::Rest)
        .unwrap()
        .health()
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Transport);
    assert!(!format!("{error:?}").contains(TOKEN));
}
