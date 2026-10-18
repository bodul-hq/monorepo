use super::*;
use client::TableStats;
fn report() -> Report {
    Report {
        schema_name: "public".into(),
        generated_at: "2026-10-17T10:30:00Z".into(),
        row_count_kind: RowCountKind::Estimated,
        tables: vec![
            TableStats {
                table_name: "alpha_small".into(),
                row_count: 7,
                table_bytes: 512,
                index_bytes: 512,
                total_bytes: 1024,
            },
            TableStats {
                table_name: "zeta_large".into(),
                row_count: 12345,
                table_bytes: 1048576,
                index_bytes: 4096,
                total_bytes: 1052672,
            },
        ],
    }
}
#[test]
fn renders_both_panels_sorting_sizes_and_manual_refresh() {
    let html = render(Ok(report()), Ok(report()));
    assert!(html.contains("Retailer Sourcing"));
    assert!(html.contains("Object Storage"));
    assert!(html.contains("12,345"));
    assert!(html.contains("1.0 MiB"));
    assert!(html.contains("1,052,672 bytes"));
    assert!(html.find("<code>alpha_small").unwrap() < html.find("<code>zeta_large").unwrap());
    assert!(html.contains("aria-sort=\"ascending\""));
    assert!(html.contains("<i style=\"width:100.00%\"></i>"));
    assert!(html.contains("<i style=\"width:0.10%\"></i>"));
    assert!(html.contains("Refresh reports"));
    assert!(html.contains("ROW COUNTS ARE ESTIMATES"));
    assert!(!html.contains("<script"));
    assert!(!html.contains("http-equiv"));
}
#[test]
fn escapes_names_and_timestamps_and_displays_empty_schemas() {
    let mut bad = report();
    bad.schema_name = "<script>alert('x')</script>".into();
    bad.generated_at = "\" onmouseover=\"x".into();
    bad.tables[0].table_name = "<img src=x onerror=alert(1)>".into();
    let mut empty = report();
    empty.tables.clear();
    let html = render(Ok(bad), Ok(empty));
    assert!(!html.contains("<script"));
    assert!(!html.contains("<img"));
    assert!(html.contains("&lt;img"));
    assert!(html.contains("&quot; onmouseover=&quot;x"));
    assert!(html.contains("No tables yet"));
}
#[test]
fn failures_leave_other_panel_visible_and_totals_labelled() {
    for (error, title) in [
        (ReportError::Unavailable, "Unavailable"),
        (ReportError::Authentication, "Authentication failed"),
        (ReportError::Malformed, "Invalid response"),
        (ReportError::Configuration, "Configuration needed"),
    ] {
        let html = render(Ok(report()), Err(error));
        assert!(html.contains(title));
        assert!(html.contains("<code>zeta_large"));
        assert!(html.contains("available services only"));
    }
    assert!(render(Err(ReportError::Unavailable), Err(ReportError::Unavailable)).contains("0 B"));
}
#[test]
fn formats_zero_boundaries_and_large_values() {
    assert_eq!(bytes(0), "0 B");
    assert_eq!(bytes(1023), "1023 B");
    assert_eq!(bytes(1024), "1.0 KiB");
    assert_eq!(number(1234567), "1,234,567");
    assert_eq!(number(0), "0");
    assert!(bytes(u64::MAX as u128).contains("EiB"));
}

#[tokio::test]
async fn dashboard_route_renders_when_services_are_unconfigured() {
    let clients = ReportingClients::new(
        "invalid",
        "",
        "invalid",
        "",
        std::time::Duration::from_secs(1),
    )
    .unwrap();
    let app = crate::app(Arc::new(Dashboard { clients }));
    let response = poem::test::TestClient::new(app)
        .get("/dashboard")
        .send()
        .await;
    response.assert_status_is_ok();
    assert_eq!(response.0.headers()["cache-control"], "no-store");
    let html = response.0.into_body().into_string().await.unwrap();
    assert!(html.contains("Configuration needed"));
    assert!(html.contains("Database overview"));
}
