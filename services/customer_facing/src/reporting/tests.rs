use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use poem::{http::StatusCode, test::TestClient};
use serde_json::{Value, json};

use super::{
    DatabaseTables, EntryStatuses, ReportError, Reporting, Repository, store::RowCountKind,
};

struct Fixture {
    error: Option<ReportError>,
    panic: bool,
    calls: AtomicUsize,
}

impl Fixture {
    fn check(&self) -> Result<(), ReportError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(!self.panic, "private query diagnostics");
        self.error.map_or(Ok(()), Err)
    }
}

impl Repository for Fixture {
    fn database_tables(&self) -> Result<DatabaseTables, ReportError> {
        self.check()?;
        Ok(DatabaseTables {
            schema_name: "test".into(),
            generated_at: "2026-10-25T12:00:00Z".into(),
            row_count_kind: RowCountKind::Estimated,
            tables: vec![],
        })
    }

    fn entry_statuses(&self) -> Result<EntryStatuses, ReportError> {
        self.check()?;
        Ok(EntryStatuses {
            generated_at: "2026-10-25T12:00:00Z".into(),
            command_entries: vec![],
            event_entries: vec![],
        })
    }
}

pub(crate) fn empty() -> Reporting {
    Reporting::new(Arc::new(Fixture {
        error: None,
        panic: false,
        calls: AtomicUsize::new(0),
    }))
}

const PATHS: [&str; 2] = [
    "/api/reporting/database-tables",
    "/api/reporting/entry-statuses",
];

fn client(repository: Arc<Fixture>, token: &str) -> TestClient<poem::Route> {
    let mut state = crate::queue_tests::state();
    state.reporting = Reporting::new(repository);
    TestClient::new(crate::app_with_reporting_token(state, token.into()))
}

#[tokio::test]
async fn auth_precedes_database_access_and_reports_are_never_cached() {
    for token in ["test-token", ""] {
        let repository = Arc::new(Fixture {
            error: None,
            panic: true,
            calls: AtomicUsize::new(0),
        });
        let client = client(repository.clone(), token);
        for path in PATHS {
            for header in [
                None,
                Some("Bearer wrong"),
                Some("Basic private"),
                Some("Bearer "),
            ] {
                let request = client.get(path);
                let request = if let Some(header) = header {
                    request.header("authorization", header)
                } else {
                    request
                };
                let response = request.send().await;
                response.assert_status(StatusCode::UNAUTHORIZED);
                assert_eq!(response.0.headers()["cache-control"], "no-store");
            }
        }
        assert_eq!(repository.calls.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn empty_reports_and_openapi_match_existing_contracts() {
    let client = client(
        Arc::new(Fixture {
            error: None,
            panic: false,
            calls: AtomicUsize::new(0),
        }),
        "test-token",
    );
    for (path, expected) in [
        (
            PATHS[0],
            json!({"schema_name":"test","generated_at":"2026-10-25T12:00:00Z","row_count_kind":"estimated","tables":[]}),
        ),
        (
            PATHS[1],
            json!({"generated_at":"2026-10-25T12:00:00Z","command_entries":[],"event_entries":[]}),
        ),
    ] {
        let response = client
            .get(path)
            .header("authorization", "Bearer test-token")
            .send()
            .await;
        response.assert_status_is_ok();
        assert_eq!(response.0.headers()["cache-control"], "no-store");
        response.assert_json(expected).await;
    }
    let response = client.get("/api/openapi.json").send().await;
    let spec: Value = response.json().await.value().deserialize();
    for path in PATHS {
        let operation = &spec["paths"][path.strip_prefix("/api").unwrap()]["get"];
        assert!(!operation["security"].as_array().unwrap().is_empty());
        assert!(
            operation["responses"]["200"]["content"]
                .as_object()
                .unwrap()
                .iter()
                .any(
                    |(media_type, media)| media_type.starts_with("application/json")
                        && media["schema"].is_object()
                )
        );
    }
}

#[tokio::test]
async fn database_and_task_failures_are_sanitized() {
    for (error, panic, status, message) in [
        (
            Some(ReportError::Unavailable),
            false,
            StatusCode::SERVICE_UNAVAILABLE,
            "Database is unavailable.",
        ),
        (
            Some(ReportError::Query),
            false,
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to generate the database report.",
        ),
        (
            None,
            true,
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to generate the database report.",
        ),
    ] {
        let client = client(
            Arc::new(Fixture {
                error,
                panic,
                calls: AtomicUsize::new(0),
            }),
            "test-token",
        );
        for path in PATHS {
            let response = client
                .get(path)
                .header("authorization", "Bearer test-token")
                .send()
                .await;
            response.assert_status(status);
            assert_eq!(response.0.headers()["cache-control"], "no-store");
            response.assert_json(json!({"error":message})).await;
        }
    }
}
