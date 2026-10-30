use super::*;
use client::TableStats;

// Existing two-service cases also verify that unavailable downstream services are isolated.
fn render(
    sourcing: Result<Report, ReportError>,
    storage: Result<Report, ReportError>,
    entries: Result<EntryStatuses, ReportError>,
) -> String {
    super::render(
        sourcing,
        storage,
        entries,
        Err(ReportError::Unavailable),
        Err(ReportError::Unavailable),
        Err(ReportError::Unavailable),
        Err(ReportError::Unavailable),
    )
}

#[test]
fn offers_contributes_to_totals_and_has_its_own_status_panel() {
    let html = super::render(
        Ok(report()),
        Ok(report()),
        Err(ReportError::Unavailable),
        Ok(report()),
        Ok(EntryStatuses {
            window_start: Some("2026-10-29T00:00:00Z".into()),
            window_end: Some("2026-10-30T00:00:00Z".into()),
            inbox_entries: Some(vec![]),
            outbox_entries: Some(vec![]),
            generated_at: "2026-10-19T12:00:00Z".into(),
            command_entries: vec![],
            event_entries: vec![client::StatusCount {
                status: 5,
                status_text: "completed".into(),
                count: 42,
            }],
        }),
        Err(ReportError::Unavailable),
        Err(ReportError::Unavailable),
    );
    assert!(html.contains("3<small> / 4</small>"));
    assert!(html.contains("<h2>Offers</h2>"));
    assert!(html.contains("Offers queue status"));
    assert!(html.contains("Retailer Sourcing queue status"));
    assert!(html.contains("<dt>Completed</dt><dd>42</dd>"));
    assert!(html.contains("<strong>6</strong>"));
    assert!(html.contains("Summary totals include available services only"));
}

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
    let html = render(Ok(report()), Ok(report()), Err(ReportError::Unavailable));
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
    assert!(
        html.contains(r#"<a class="active" href="/dashboard" aria-current="page">Dashboard</a>"#)
    );
    assert!(html.contains(r#"data-theme-choice="system""#));
    assert!(html.ends_with("</body></html>"));
    assert!(html.contains("ROW COUNTS ARE ESTIMATES"));
    // Only the theme scripts from the shared layout; reports render without JavaScript.
    assert_eq!(html.matches("<script").count(), 2);
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
    let html = render(Ok(bad), Ok(empty), Err(ReportError::Unavailable));
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("&lt;script&gt;alert"));
    assert!(!html.contains("<img src=x onerror=alert(1)>"));
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
        let html = render(Ok(report()), Err(error), Err(ReportError::Unavailable));
        assert!(html.contains(title));
        assert!(html.contains("<code>zeta_large"));
        assert!(html.contains("available services only"));
    }
    assert!(
        render(
            Err(ReportError::Unavailable),
            Err(ReportError::Unavailable),
            Err(ReportError::Unavailable)
        )
        .contains("0 B")
    );
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

#[test]
fn renders_exact_status_counts_and_empty_queues() {
    let entries = EntryStatuses {
        window_start: Some("2026-10-29T00:00:00Z".into()),
        window_end: Some("2026-10-30T00:00:00Z".into()),
        inbox_entries: Some(vec![]),
        outbox_entries: Some(vec![]),
        generated_at: "2026-10-18T12:00:00Z".into(),
        command_entries: vec![client::StatusCount {
            status: 4,
            status_text: "failed <test>".into(),
            count: 1234,
        }],
        event_entries: vec![],
    };
    let html = render(Ok(report()), Ok(report()), Ok(entries));
    assert!(html.contains("Command entries"));
    assert!(html.contains("Event entries"));
    assert!(html.contains("failed &lt;test&gt;"));
    assert!(html.contains("1,234"));
    assert_queue(&html, "Retailer Sourcing", "Inbox entries", true);
    assert_queue(&html, "Retailer Sourcing", "Command entries", false);
    assert_queue(&html, "Retailer Sourcing", "Event entries", true);
    assert_queue(&html, "Retailer Sourcing", "Outbox entries", true);
}

#[test]
fn customer_facing_contributes_to_totals_with_independent_reports() {
    let empty_entries = || EntryStatuses {
        window_start: Some("2026-10-29T00:00:00Z".into()),
        window_end: Some("2026-10-30T00:00:00Z".into()),
        inbox_entries: Some(vec![]),
        outbox_entries: Some(vec![]),
        generated_at: "2026-10-25T12:00:00Z".into(),
        command_entries: vec![],
        event_entries: vec![],
    };
    let html = super::render(
        Ok(report()),
        Ok(report()),
        Ok(empty_entries()),
        Ok(report()),
        Ok(empty_entries()),
        Ok(report()),
        Ok(empty_entries()),
    );
    assert!(html.contains("4<small> / 4</small>"));
    assert!(html.contains("<h2>Customer Facing</h2>"));
    // Each service has separate command and event boxes, including zero counts.
    assert_eq!(html.matches(r#"class="command-boxes""#).count(), 12);
    assert_eq!(
        html.matches(r#"<dt>New</dt><dd><span class="sr-only">0</span></dd>"#)
            .count(),
        12
    );
    assert_eq!(
        html.matches(r#"<dt>Archive</dt><dd><span class="sr-only">0</span></dd>"#)
            .count(),
        12
    );
    assert!(html.contains("Customer Facing queue status"));
    assert!(html.contains("<strong>8</strong>"));
    assert!(html.contains("4.0 MiB"));
    assert!(!html.contains("Summary totals include available services only"));
    for service in ["Retailer Sourcing", "Offers", "Customer Facing"] {
        for queue in [
            "Inbox entries",
            "Command entries",
            "Event entries",
            "Outbox entries",
        ] {
            assert_queue(&html, service, queue, true);
        }
    }

    for error in [
        ReportError::Unavailable,
        ReportError::Authentication,
        ReportError::Malformed,
        ReportError::Configuration,
    ] {
        let html = super::render(
            Ok(report()),
            Ok(report()),
            Ok(empty_entries()),
            Ok(report()),
            Ok(empty_entries()),
            Err(error),
            Ok(empty_entries()),
        );
        assert!(html.contains("3<small> / 4</small>"));
        assert!(html.contains("<h2>Offers</h2>"));
        assert!(html.contains("<strong>6</strong>"));
        assert!(html.contains("Summary totals include available services only"));
        assert_queue(&html, "Customer Facing", "Command entries", true);

        let mut customer = report();
        customer.tables[0].table_name = "<customer-table>".into();
        let html = super::render(
            Ok(report()),
            Ok(report()),
            Ok(empty_entries()),
            Ok(report()),
            Ok(empty_entries()),
            Ok(customer),
            Err(error),
        );
        assert!(html.contains("4<small> / 4</small>"));
        assert!(html.contains("&lt;customer-table&gt;"));
        assert!(!html.contains("<customer-table>"));
        assert!(!html.contains("customer-facing-development-token"));
        assert!(!html.contains("172.28.100.42"));
        let panel = service_panel(&html, "Customer Facing");
        assert!(panel.contains("error-state"));
        assert!(!panel.contains("queue-row-empty"));
        assert!(!panel.contains("command-boxes"));
    }
}

#[test]
fn inbox_outbox_counts_and_older_service_reports() {
    for (service, image) in [
        ("Retailer Sourcing", "retailer-sourcing"),
        ("Offers", "offers"),
        ("Customer Facing", "customer-facing"),
    ] {
        let report = serde_json::from_value(serde_json::json!({
            "generated_at": "2026-10-29T12:00:00Z",
            "window_start": "2026-10-29T00:00:00Z",
            "window_end": "2026-10-30T00:00:00Z",
            "inbox_entries": [{"status": 0, "status_text": "received", "count": 1234}],
            "command_entries": [], "event_entries": [],
            "outbox_entries": [{"status": 8, "status_text": "archive", "count": 77},
                {"status": 99, "status_text": "<unknown>", "count": 9}]
        }))
        .unwrap();
        let mut html = String::new();
        render_entries(&mut html, service, image, Ok(report));
        assert!(html.contains("<dt>New</dt><dd>1,234</dd>"));
        assert!(html.contains("<dt>Archive</dt><dd>77</dd>"));
        assert!(html.contains("&lt;unknown&gt; (status 99)</dt><dd>9</dd>"));
        assert!(html.contains(&format!("src=\"/assets/{image}.svg\"")));
        assert!(html.contains(&format!("alt=\"{service} queue status\"")));
        assert_queue(&html, service, "Inbox entries", false);
        assert_queue(&html, service, "Command entries", true);
        assert_queue(&html, service, "Event entries", true);
        assert_queue(&html, service, "Outbox entries", false);
        let inbox = html.find("aria-label=\"Inbox entries\"").unwrap();
        let command = html.find("aria-label=\"Command entries\"").unwrap();
        let event = html.find("aria-label=\"Event entries\"").unwrap();
        let outbox = html.find("aria-label=\"Outbox entries\"").unwrap();
        assert!(inbox < command && command < event && event < outbox);
        for icon in ["inbox", "commands", "events", "outbox"] {
            assert!(html.contains(&format!("src=\"/assets/{icon}.svg\"")));
        }
        assert!(!html.contains("<h3>Inbox entries</h3>"));
        assert!(html[inbox..command].contains("1,234"));
        assert!(!html[inbox..command].contains("command-empty"));
        assert_eq!(html[command..outbox].matches("command-empty").count(), 12);
        assert!(!html[outbox..].contains("command-empty"));
        assert!(!html[command..outbox].contains("1,234"));
        assert!(html[outbox..].contains("77"));

        let old = serde_json::from_value(serde_json::json!({
            "generated_at": "2026-10-29T12:00:00Z",
            "command_entries": [], "event_entries": []
        }))
        .unwrap();
        let mut html = String::new();
        render_entries(&mut html, service, image, Ok(old));
        assert_eq!(html.matches("Upgrade this service").count(), 1);
        assert!(!html.contains("Report UTC date"));
        assert_eq!(html.matches("class=\"command-boxes\"").count(), 0);
    }
}

#[test]
fn compact_boxes_require_the_whole_queue_to_be_empty() {
    for status in [0, 2, 4, 5, 7, 8, 99] {
        for count in [0, 1] {
            let entries = vec![client::StatusCount {
                status,
                status_text: "test".into(),
                count,
            }];
            let mut html = String::new();
            render_entry_boxes(&mut html, "Inbox entries", "inbox", &entries);
            assert_eq!(html.contains("queue-row-empty"), count == 0);
            assert!(!html.contains("<dd>0</dd>"));
            if count > 0 {
                assert!(html.contains("<dd>1</dd>"));
            }
            let boxes = if status == 99 { 7 } else { 6 };
            assert_eq!(
                html.matches("command-empty").count(),
                if count == 0 { boxes } else { 0 }
            );
        }
    }
}

#[test]
fn daily_counts_require_an_explicit_valid_window() {
    for (service, image) in [
        ("Retailer Sourcing", "retailer-sourcing"),
        ("Offers", "offers"),
        ("Customer Facing", "customer-facing"),
    ] {
        let legacy = serde_json::json!({
            "generated_at": "2026-10-30T01:00:00+02:00",
            "command_entries": [{"status":5,"status_text":"completed","count":120000}],
            "event_entries": []
        });
        for with_queues in [false, true] {
            let mut value = legacy.clone();
            if with_queues {
                value["inbox_entries"] = serde_json::json!([]);
                value["outbox_entries"] = serde_json::json!([]);
            }
            let mut html = String::new();
            render_entries(
                &mut html,
                service,
                image,
                Ok(serde_json::from_value(value).unwrap()),
            );
            assert!(html.contains("Upgrade this service"));
            assert!(!html.contains("120,000"));
            assert!(!html.contains("Report UTC date"));
        }
        let mut value = legacy;
        // Local dates differ from the UTC date; generation time is not the window.
        value["window_start"] = "2026-10-28T19:00:00-05:00".into();
        value["window_end"] = "2026-10-29T19:00:00-05:00".into();
        let mut html = String::new();
        render_entries(
            &mut html,
            service,
            image,
            Ok(serde_json::from_value(value.clone()).unwrap()),
        );
        assert!(html.contains("Report UTC date: 2026-10-29"));
        assert!(html.contains("120,000"));
        for end in [
            "bad",
            "2026-10-29T00:00:00Z",
            "2026-10-31T00:00:00Z",
            "2026-10-30T01:00:00Z",
        ] {
            value["window_end"] = end.into();
            let mut html = String::new();
            render_entries(
                &mut html,
                service,
                image,
                Ok(serde_json::from_value(value.clone()).unwrap()),
            );
            assert!(html.contains("Invalid reporting window"));
            assert!(!html.contains("120,000"));
            assert!(!html.contains("Report UTC date"));
        }
        value.as_object_mut().unwrap().remove("window_end");
        let report: EntryStatuses = serde_json::from_value(value).unwrap();
        assert!(report.window_date().is_err());
    }
}

#[test]
fn present_window_bounds_must_be_strings() {
    for field in ["window_start", "window_end"] {
        for invalid in [
            serde_json::Value::Null,
            serde_json::json!(42),
            serde_json::json!([]),
        ] {
            let mut value = serde_json::json!({
                "generated_at": "2026-10-29T12:00:00Z",
                "command_entries": [], "event_entries": []
            });
            value[field] = invalid;
            assert!(serde_json::from_value::<EntryStatuses>(value).is_err());
        }
    }
}

#[test]
fn display_labels_do_not_select_icons() {
    let mut html = String::new();
    render_entries(
        &mut html,
        "Renamed service",
        "offers",
        Err(ReportError::Unavailable),
    );
    assert!(html.contains(r#"src="/assets/offers.svg" alt="Renamed service queue status""#));
    html.clear();
    render_entry_boxes(&mut html, "Renamed queue", "inbox", &[]);
    assert!(html.contains(r#"src="/assets/inbox.svg" alt="Renamed queue""#));
}

fn service_panel<'a>(html: &'a str, service: &str) -> &'a str {
    let start = html
        .find(&format!(r#"aria-label="{service} queue status""#))
        .unwrap();
    let rest = &html[start..];
    &rest[..rest
        .find(r#"<section class="service "#)
        .unwrap_or(rest.len())]
}

fn assert_queue(html: &str, service: &str, queue: &str, empty: bool) {
    let panel = service_panel(html, service);
    let label = panel.find(&format!(r#"aria-label="{queue}""#)).unwrap();
    let start = panel[..label].rfind("<section").unwrap();
    let row = &panel[start..];
    let row = &row[..row.find("</section>").unwrap()];
    assert_eq!(row.contains("queue-row-empty"), empty, "{service}: {queue}");
    assert_eq!(
        row.matches("command-empty").count(),
        if empty { 6 } else { 0 }
    );
    if empty {
        for label in [
            "New",
            "Processing",
            "Failed",
            "Completed",
            "Dead",
            "Archive",
        ] {
            assert!(row.contains(&format!(
                r#"<dt>{label}</dt><dd><span class="sr-only">0</span></dd>"#
            )));
        }
    }
}

#[test]
fn missing_queues_are_distinct_from_empty_and_unknown_only_queues() {
    for (service, image) in [
        ("Offers", "offers"),
        ("Customer Facing", "customer-facing"),
        ("Retailer Sourcing", "retailer-sourcing"),
    ] {
        let report = serde_json::from_value(serde_json::json!({
            "generated_at": "2026-12-31T12:00:00Z",
            "window_start": "2026-12-31T00:00:00Z",
            "window_end": "2027-01-01T00:00:00Z",
            "command_entries": [{"status":99,"status_text":"unknown","count":1}],
            "event_entries": []
        }))
        .unwrap();
        let mut html = String::new();
        render_entries(&mut html, service, image, Ok(report));
        assert_queue(&html, service, "Command entries", false);
        assert_queue(&html, service, "Event entries", true);
        assert_eq!(html.matches("queue-row-empty").count(), 1);
        assert_eq!(html.matches("Upgrade this service").count(), 2);
        assert!(html.contains("unknown (status 99)</dt><dd>1</dd>"));
    }
}
