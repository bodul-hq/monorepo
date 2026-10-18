use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

fn fixture(
    status: u16,
    body: &str,
    token: &str,
    delay: Duration,
) -> (String, thread::JoinHandle<()>) {
    fixture_path(status, body, token, delay, PATH)
}
fn fixture_path(
    status: u16,
    body: &str,
    token: &str,
    delay: Duration,
    path: &str,
) -> (String, thread::JoinHandle<()>) {
    let path = path.to_owned();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let body = body.to_owned();
    let token = token.to_owned();
    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut input = Vec::new();
        while !input.windows(4).any(|w| w == b"\r\n\r\n") {
            let mut chunk = [0; 1024];
            let n = socket.read(&mut chunk).unwrap();
            assert!(n > 0);
            input.extend_from_slice(&chunk[..n]);
        }
        let request = String::from_utf8(input).unwrap();
        assert!(request.starts_with(&format!("GET {path} HTTP/1.1")));
        assert!(
            request
                .to_ascii_lowercase()
                .contains(&format!("authorization: bearer {token}"))
        );
        thread::sleep(delay);
        let _ = write!(
            socket,
            "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nLocation: http://127.0.0.1:1/private\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
    });
    (url, handle)
}
const REPORT: &str = r#"{"schema_name":"public","generated_at":"2026-10-17T10:30:00Z","row_count_kind":"estimated","tables":[{"table_name":"records","row_count":12,"table_bytes":8192,"index_bytes":4096,"total_bytes":12288}]}"#;

#[tokio::test]
async fn authenticates_both_services_and_keeps_partial_results() {
    let (a, ah) = fixture(200, REPORT, "sourcing-token", Duration::ZERO);
    let (b, bh) = fixture(401, "private diagnostics", "storage-token", Duration::ZERO);
    let clients = ReportingClients::new(
        &a,
        "sourcing-token",
        &b,
        "storage-token",
        Duration::from_secs(1),
    )
    .unwrap();
    let (a, b) = clients.reports().await;
    assert_eq!(a.unwrap().tables[0].row_count, 12);
    assert_eq!(b.unwrap_err(), ReportError::Authentication);
    ah.join().unwrap();
    bh.join().unwrap();
}
#[tokio::test]
async fn classifies_failures_without_exposing_upstream_content() {
    for (status, body, error) in [
        (503, "private diagnostics", ReportError::Unavailable),
        (403, "secret", ReportError::Authentication),
        (200, "not json", ReportError::Malformed),
        (200, "{}", ReportError::Malformed),
        (302, "", ReportError::Unavailable),
        (200, &REPORT.replace("12288", "0"), ReportError::Malformed),
    ] {
        let (url, handle) = fixture(status, body, "token", Duration::ZERO);
        let clients =
            ReportingClients::new(&url, "token", "invalid", "token", Duration::from_secs(1))
                .unwrap();
        let (a, b) = clients.reports().await;
        assert_eq!(a.unwrap_err(), error);
        assert_eq!(b.unwrap_err(), ReportError::Configuration);
        handle.join().unwrap();
    }
}
#[tokio::test]
async fn times_out_and_rejects_invalid_configuration() {
    let (url, handle) = fixture(200, REPORT, "token", Duration::from_millis(200));
    let clients =
        ReportingClients::new(&url, "token", &url, "", Duration::from_millis(30)).unwrap();
    let (a, b) = clients.reports().await;
    assert_eq!(a.unwrap_err(), ReportError::Unavailable);
    assert_eq!(b.unwrap_err(), ReportError::Configuration);
    handle.join().unwrap();
    for url in [
        "file:///tmp/report",
        "http://user:pass@localhost",
        "http://localhost?secret=token",
        "http://localhost#fragment",
    ] {
        assert!(endpoint(url, "token").is_err());
    }
}

#[tokio::test]
async fn fetches_authenticated_status_counts_and_handles_errors() {
    for (status, body, expected) in [
        (
            200,
            r#"{"generated_at":"2026-10-18T12:00:00Z","command_entries":[{"status":0,"status_text":"received","count":12}],"event_entries":[]}"#,
            None,
        ),
        (200, "{}", Some(ReportError::Malformed)),
        (401, "private", Some(ReportError::Authentication)),
        (503, "private", Some(ReportError::Unavailable)),
    ] {
        let (url, handle) = fixture_path(
            status,
            body,
            "token",
            Duration::ZERO,
            "/prefix/api/reporting/entry-statuses",
        );
        let clients = ReportingClients::new(
            &format!("{url}/prefix"),
            "token",
            "invalid",
            "",
            Duration::from_secs(1),
        )
        .unwrap();
        let result = clients.entry_statuses().await;
        if let Some(error) = expected {
            assert_eq!(result.unwrap_err(), error);
        } else {
            assert_eq!(result.unwrap().command_entries[0].count, 12);
        }
        handle.join().unwrap();
    }
}
