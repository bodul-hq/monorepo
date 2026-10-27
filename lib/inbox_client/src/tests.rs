use super::*;
use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{any, post},
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

struct TestServer {
    url: String,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve(app: Router) -> TestServer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    TestServer { url, task }
}

fn submission(message_id: Uuid) -> Value {
    json!({"payload":{"title":"USB charger"},"routing_key":"Example","message_id":message_id})
}

async fn submit(client: InboxClient, expected: Uuid) -> Result<(), SubmitError> {
    tokio::task::spawn_blocking(move || client.submit(&submission(expected), expected))
        .await
        .unwrap()
}

#[test]
fn configuration_accepts_only_origins() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    for origin in [
        "http://localhost",
        "https://offers.example/",
        "http://[::1]:3100",
    ] {
        let url = inbox_url(origin).unwrap();
        assert_eq!(url.path(), "/api/inbox");
    }
    for origin in [
        "",
        "not a URL",
        "ftp://example.com",
        "http://user:secret@example.com",
        "http://example.com/api",
        "http://example.com/a/..",
        "http://@example.com",
        "http:example.com",
        "http://example.com/\\",
        "http://example.com/?token=secret",
        "http://example.com/#secret",
        " http://example.com",
        "http://example.com\n",
    ] {
        assert_eq!(inbox_url(origin).unwrap_err(), ConfigError::InvalidOrigin);
    }
    let handle = || runtime.handle().clone();
    assert!(
        InboxClient::from_env_value(Err(VarError::NotPresent), handle())
            .unwrap()
            .is_none()
    );
    assert_eq!(
        InboxClient::from_env_value(Ok(String::new()), handle()).unwrap_err(),
        ConfigError::InvalidOrigin
    );
    assert!(
        InboxClient::from_env_value(Ok("http://localhost:1".into()), handle())
            .unwrap()
            .is_some()
    );
    #[cfg(unix)]
    {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};
        let invalid = VarError::NotUnicode(OsString::from_vec(vec![0xff]));
        assert_eq!(
            InboxClient::from_env_value(Err(invalid), handle()).unwrap_err(),
            ConfigError::InvalidOrigin
        );
    }
    assert!(!ConfigError::InvalidOrigin.to_string().contains("secret"));
}

#[tokio::test(flavor = "multi_thread")]
async fn posts_the_submission_as_json_and_confirms_the_matching_uuid() {
    let seen = Arc::new(Mutex::new(Vec::<(Option<String>, Value)>::new()));
    let captured = seen.clone();
    let app = Router::new().route(
        "/api/inbox",
        post(
            move |headers: axum::http::HeaderMap, Json(body): Json<Value>| {
                let captured = captured.clone();
                async move {
                    let content_type = headers
                        .get("content-type")
                        .map(|value| value.to_str().unwrap().to_owned());
                    captured.lock().unwrap().push((content_type, body.clone()));
                    (
                        axum::http::StatusCode::ACCEPTED,
                        Json(json!({"message_id":body["message_id"]})),
                    )
                }
            },
        ),
    );
    let server = serve(app).await;
    let client = InboxClient::new(&server.url, Handle::current()).unwrap();
    let expected = Uuid::now_v7();

    submit(client.clone(), expected).await.unwrap();
    submit(client, expected).await.unwrap();

    let requests = seen.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0], requests[1]);
    assert_eq!(
        requests[0],
        (Some("application/json".into()), submission(expected))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn rejects_non_acceptance_without_redirects_or_automatic_retries() {
    for status in [200, 201, 204, 302, 400, 429, 500] {
        let calls = Arc::new(Mutex::new(0));
        let captured = calls.clone();
        let redirects = Arc::new(Mutex::new(0));
        let redirected = redirects.clone();
        let app = Router::new()
            .route(
                "/api/inbox",
                post(move || {
                    let captured = captured.clone();
                    async move {
                        *captured.lock().unwrap() += 1;
                        (
                            axum::http::StatusCode::from_u16(status).unwrap(),
                            [("location", "/must-not-follow")],
                            "private response body",
                        )
                    }
                }),
            )
            .route(
                "/must-not-follow",
                any(move || {
                    let redirected = redirected.clone();
                    async move {
                        *redirected.lock().unwrap() += 1;
                        axum::http::StatusCode::OK
                    }
                }),
            );
        let server = serve(app).await;
        let client = InboxClient::new(&server.url, Handle::current()).unwrap();
        let error = submit(client, Uuid::now_v7()).await.unwrap_err();
        assert_eq!(error, SubmitError::Rejected, "{status}");
        assert!(!error.to_string().contains("private"));
        assert_eq!(*calls.lock().unwrap(), 1);
        assert_eq!(*redirects.lock().unwrap(), 0);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn rejects_invalid_oversized_and_mismatched_acknowledgements() {
    for body in [
        "{".to_owned(),
        "{}".into(),
        "{\"message_id\":\"wrong\"}".into(),
        json!({"message_id":Uuid::now_v7()}).to_string(),
        " ".repeat(MAX_ACK_BYTES + 1),
    ] {
        let app = Router::new().route(
            "/api/inbox",
            post(move || {
                let body = body.clone();
                async move { (axum::http::StatusCode::ACCEPTED, body) }
            }),
        );
        let server = serve(app).await;
        let client = InboxClient::new(&server.url, Handle::current()).unwrap();
        assert_eq!(
            submit(client, Uuid::now_v7()).await.unwrap_err(),
            SubmitError::Acknowledgement
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn bounds_total_time_and_reports_connection_failure() {
    let server = serve(Router::new().route(
        "/api/inbox",
        post(|| async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            axum::http::StatusCode::ACCEPTED.into_response()
        }),
    ))
    .await;
    let client =
        InboxClient::with_timeout(&server.url, Handle::current(), Duration::from_millis(30))
            .unwrap();
    let outcome = tokio::time::timeout(Duration::from_millis(500), submit(client, Uuid::now_v7()))
        .await
        .expect("client deadline must fire before the delayed response");
    assert!(outcome.is_err());
    drop(server);
    // Release an ephemeral listener so the subsequent connection is refused.
    let unavailable = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", unavailable.local_addr().unwrap());
    drop(unavailable);
    let client = InboxClient::new(&address, Handle::current()).unwrap();
    assert_eq!(
        submit(client, Uuid::now_v7()).await.unwrap_err(),
        SubmitError::Transport
    );
}

/// Accepts one connection and answers `202` with a chunked body.
async fn chunked_response(chunks: Vec<(Duration, Vec<u8>)>) -> TestServer {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            let size = stream.read(&mut buffer).await.unwrap();
            if size == 0 {
                return;
            }
            request.extend_from_slice(&buffer[..size]);
        }
        if stream.write_all(b"HTTP/1.1 202 Accepted\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").await.is_err() { return; }
        for (delay, bytes) in chunks {
            tokio::time::sleep(delay).await;
            let mut chunk = format!("{:X}\r\n", bytes.len()).into_bytes();
            chunk.extend_from_slice(&bytes);
            chunk.extend_from_slice(b"\r\n");
            if stream.write_all(&chunk).await.is_err() {
                return;
            }
        }
        let _ = stream.write_all(b"0\r\n\r\n").await;
    });
    TestServer { url, task }
}

#[tokio::test(flavor = "multi_thread")]
async fn bounds_chunked_acknowledgements_without_content_length() {
    let expected = Uuid::now_v7();
    let mut acknowledgement = json!({"message_id":expected}).to_string().into_bytes();
    acknowledgement.resize(MAX_ACK_BYTES + 1, b' ');
    let server = chunked_response(vec![(Duration::ZERO, acknowledgement)]).await;
    let client = InboxClient::new(&server.url, Handle::current()).unwrap();
    assert_eq!(
        submit(client, expected).await.unwrap_err(),
        SubmitError::Acknowledgement
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn deadline_covers_a_slow_acknowledgement_body() {
    let expected = Uuid::now_v7();
    let final_chunk = format!("\"{expected}\"}}").into_bytes();
    let server = chunked_response(vec![
        (Duration::ZERO, b"{\"message_id\":".to_vec()),
        (Duration::from_secs(1), final_chunk),
    ])
    .await;
    let client =
        InboxClient::with_timeout(&server.url, Handle::current(), Duration::from_millis(30))
            .unwrap();
    let outcome = tokio::time::timeout(Duration::from_millis(500), submit(client, expected))
        .await
        .expect("the deadline must cover reading a streamed response body");
    assert!(outcome.is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_disconnect_after_the_headers_is_a_transport_failure() {
    let server = chunked_response(vec![(Duration::ZERO, b"{\"message_id\"".to_vec())]).await;
    // The server closes after one chunk without the terminating chunk.
    let client = InboxClient::new(&server.url, Handle::current()).unwrap();
    let error = submit(client, Uuid::now_v7()).await.unwrap_err();
    assert!(
        matches!(error, SubmitError::Transport | SubmitError::Acknowledgement),
        "{error:?}"
    );
}
