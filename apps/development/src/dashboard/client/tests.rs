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

#[tokio::test]
async fn fetches_offers_reports_with_its_own_token_and_base_path() {
    let (url, handle) = fixture_path(
        200,
        REPORT,
        "offers-token",
        Duration::ZERO,
        "/prefix/api/reporting/database-tables",
    );
    let clients = ReportingClients::new("invalid", "", "invalid", "", Duration::from_secs(1))
        .unwrap()
        .with_offers(&format!("{url}/prefix"), "offers-token");
    assert_eq!(
        clients.offers_report().await.unwrap().tables[0].row_count,
        12
    );
    handle.join().unwrap();
    for (status, body, expected) in [
        (
            200,
            r#"{"generated_at":"2026-10-19T12:00:00Z","command_entries":[],"event_entries":[]}"#,
            None,
        ),
        (401, "private", Some(ReportError::Authentication)),
        (503, "private", Some(ReportError::Unavailable)),
        (200, "{}", Some(ReportError::Malformed)),
    ] {
        let (url, handle) = fixture_path(
            status,
            body,
            "offers-token",
            Duration::ZERO,
            "/prefix/api/reporting/entry-statuses",
        );
        let clients = ReportingClients::new("invalid", "", "invalid", "", Duration::from_secs(1))
            .unwrap()
            .with_offers(&format!("{url}/prefix"), "offers-token");
        let result = clients.offers_entry_statuses().await;
        if let Some(error) = expected {
            assert_eq!(result.unwrap_err(), error);
        } else {
            assert!(result.unwrap().event_entries.is_empty());
        }
        handle.join().unwrap();
    }
}

#[tokio::test]
async fn customer_facing_reports_use_their_own_token_and_prefix() {
    for statuses in [false, true] {
        let path = if statuses {
            "/prefix/api/reporting/entry-statuses"
        } else {
            "/prefix/api/reporting/database-tables"
        };
        let body = if statuses {
            r#"{"generated_at":"2026-10-25T12:00:00Z","command_entries":[{"status":5,"status_text":"completed","count":3}],"event_entries":[]}"#
        } else {
            REPORT
        };
        let (url, handle) = fixture_path(200, body, "customer-token", Duration::ZERO, path);
        let clients = ReportingClients::new("invalid", "", "invalid", "", Duration::from_secs(1))
            .unwrap()
            .with_offers("invalid", "offers-token")
            .with_customer_facing(&format!("{url}/prefix/"), "customer-token");
        if statuses {
            assert_eq!(
                clients
                    .customer_facing_entry_statuses()
                    .await
                    .unwrap()
                    .command_entries[0]
                    .count,
                3
            );
        } else {
            assert_eq!(
                clients.customer_facing_report().await.unwrap().tables[0].row_count,
                12
            );
        }
        handle.join().unwrap();
    }
}

#[tokio::test]
async fn customer_facing_failures_and_timeout_do_not_affect_other_reports() {
    for statuses in [false, true] {
        let path = if statuses {
            "/api/reporting/entry-statuses"
        } else {
            PATH
        };
        for (status, body, delay, expected) in [
            (
                401,
                "private upstream diagnostics",
                Duration::ZERO,
                ReportError::Authentication,
            ),
            (
                503,
                "private upstream diagnostics",
                Duration::ZERO,
                ReportError::Unavailable,
            ),
            (302, "", Duration::ZERO, ReportError::Unavailable),
            (200, "{}", Duration::ZERO, ReportError::Malformed),
            (200, "not json", Duration::ZERO, ReportError::Malformed),
            (
                200,
                REPORT,
                Duration::from_millis(200),
                ReportError::Unavailable,
            ),
        ] {
            let (url, handle) = fixture_path(status, body, "customer-token", delay, path);
            let (sourcing, sourcing_handle) =
                fixture(200, REPORT, "sourcing-token", Duration::ZERO);
            let clients = ReportingClients::new(
                &sourcing,
                "sourcing-token",
                "invalid",
                "",
                Duration::from_millis(50),
            )
            .unwrap()
            .with_customer_facing(&url, "customer-token");
            let customer = async {
                if statuses {
                    clients.customer_facing_entry_statuses().await.map(|_| ())
                } else {
                    clients.customer_facing_report().await.map(|_| ())
                }
            };
            let ((sourcing, _), customer) = tokio::join!(clients.reports(), customer);
            assert_eq!(sourcing.unwrap().tables[0].row_count, 12);
            assert_eq!(customer.unwrap_err(), expected);
            handle.join().unwrap();
            sourcing_handle.join().unwrap();
        }
    }
    let clients = ReportingClients::new("invalid", "", "invalid", "", Duration::from_secs(1))
        .unwrap()
        .with_customer_facing("http://example.test", "");
    assert_eq!(
        clients.customer_facing_report().await.unwrap_err(),
        ReportError::Configuration
    );
    assert_eq!(
        clients.customer_facing_entry_statuses().await.unwrap_err(),
        ReportError::Configuration
    );
}

#[test]
fn new_queue_fields_distinguish_missing_empty_and_malformed() {
    let base = serde_json::json!({"generated_at":"2026-10-29T12:00:00Z",
        "command_entries":[], "event_entries":[]});
    let old: EntryStatuses = serde_json::from_value(base.clone()).unwrap();
    assert!(old.inbox_entries.is_none());
    assert!(old.outbox_entries.is_none());
    for field in ["inbox_entries", "outbox_entries"] {
        for bad in [
            serde_json::Value::Null,
            serde_json::json!({}),
            serde_json::json!([{"status":0,"status_text":"received","count":-1}]),
        ] {
            let mut input = base.clone();
            input[field] = bad;
            assert!(serde_json::from_value::<EntryStatuses>(input).is_err());
        }
    }
    let mut empty = base;
    empty["inbox_entries"] = serde_json::json!([]);
    empty["outbox_entries"] = serde_json::json!([]);
    let empty: EntryStatuses = serde_json::from_value(empty).unwrap();
    assert!(empty.inbox_entries.unwrap().is_empty());
    assert!(empty.outbox_entries.unwrap().is_empty());
}
