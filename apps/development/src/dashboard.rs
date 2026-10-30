//! Server-rendered reporting dashboard. Service credentials never reach the browser.
mod client;
use crate::layout::{self, Page};
use client::{EntryStatuses, Report, ReportError, ReportingClients, RowCountKind};
use poem::{
    IntoResponse, Response, handler,
    web::{Data, Html},
};
use std::{fmt::Write, sync::Arc};

pub(crate) struct Dashboard {
    clients: ReportingClients,
}
impl Dashboard {
    pub(crate) fn from_env() -> Result<Self, reqwest::Error> {
        Ok(Self {
            clients: ReportingClients::from_env()?,
        })
    }
}

#[handler]
pub(crate) async fn page(Data(state): Data<&Arc<Dashboard>>) -> Response {
    let (
        (sourcing, storage),
        entries,
        offers,
        offers_entries,
        customer_facing,
        customer_facing_entries,
    ) = tokio::join!(
        state.clients.reports(),
        state.clients.entry_statuses(),
        state.clients.offers_report(),
        state.clients.offers_entry_statuses(),
        state.clients.customer_facing_report(),
        state.clients.customer_facing_entry_statuses(),
    );
    let mut response = Html(render(
        sourcing,
        storage,
        entries,
        offers,
        offers_entries,
        customer_facing,
        customer_facing_entries,
    ))
    .into_response();
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
}

fn render(
    sourcing: Result<Report, ReportError>,
    storage: Result<Report, ReportError>,
    entries: Result<EntryStatuses, ReportError>,
    offers: Result<Report, ReportError>,
    offers_entries: Result<EntryStatuses, ReportError>,
    customer_facing: Result<Report, ReportError>,
    customer_facing_entries: Result<EntryStatuses, ReportError>,
) -> String {
    let reports = [
        (&sourcing, "Retailer Sourcing", "Pipeline metadata", "01"),
        (&storage, "Object Storage", "Stored artifacts", "02"),
        (&offers, "Offers", "Offer events and metadata", "03"),
        (
            &customer_facing,
            "Customer Facing",
            "Customer offer projections",
            "04",
        ),
    ];
    let service_count = reports.len();
    let available = reports.iter().filter(|(r, _, _, _)| r.is_ok()).count();
    let total_bytes: u128 = reports
        .iter()
        .filter_map(|(r, _, _, _)| r.as_ref().ok())
        .flat_map(|r| &r.tables)
        .map(|t| t.total_bytes as u128)
        .sum();
    let total_tables: usize = reports
        .iter()
        .filter_map(|(r, _, _, _)| r.as_ref().ok())
        .map(|r| r.tables.len())
        .sum();
    let mut html = layout::start("Database overview · Development", Page::Dashboard);
    html.push_str(r#"<main><div class="hero"><div><p class="eyebrow">SYSTEM VISIBILITY</p><h1>Database overview<span class="title-dot">.</span></h1><p class="lede">A clear view of the tables behind your pipeline.</p></div><a class="refresh" href="/dashboard"><span aria-hidden="true">↻</span> Refresh reports</a></div>"#);
    let _ = write!(
        html,
        r#"<section class="overview" aria-label="Overview"><div><span class="metric-label">Services available</span><strong>{available}<small> / {service_count}</small></strong></div><div><span class="metric-label">Tables reported</span><strong>{total_tables}</strong></div><div><span class="metric-label">Storage reported</span><strong>{}</strong></div><div class="refresh-note"><span class="mini-dot" aria-hidden="true"></span><div><b>Refresh on demand</b><span>Reports update when you load this page.</span></div></div></section>"#,
        bytes(total_bytes)
    );
    if available < service_count {
        html.push_str(
            r#"<p class="partial-note">Summary totals include available services only.</p>"#,
        );
    }
    render_entries(&mut html, "Retailer Sourcing", entries);
    render_entries(&mut html, "Offers", offers_entries);
    render_entries(&mut html, "Customer Facing", customer_facing_entries);
    html.push_str(r#"<div class="services">"#);
    for (report, title, description, number) in reports {
        let _ = write!(
            html,
            r#"<section class="service"><header class="service-header"><div class="service-title"><span class="service-number">{number}</span><div><h2>{title}</h2><p>{description}</p></div></div>"#
        );
        match report {
            Ok(report) => {
                html.push_str(r#"<span class="status healthy"><i aria-hidden="true"></i>Available</span></header>"#);
                render_report(&mut html, report);
            }
            Err(error) => {
                let (label, description) = match error {
                    ReportError::Unavailable => (
                        "Unavailable",
                        "The service could not be reached or could not generate its report. Try refreshing shortly.",
                    ),
                    ReportError::Authentication => (
                        "Authentication failed",
                        "The service rejected the dashboard credentials. Check this service’s token configuration.",
                    ),
                    ReportError::Malformed => (
                        "Invalid response",
                        "The service returned a report the dashboard could not read. Check that the service is up to date.",
                    ),
                    ReportError::Configuration => (
                        "Configuration needed",
                        "The service address or token is invalid. Check the development app’s configuration.",
                    ),
                };
                let _ = write!(
                    html,
                    r#"<span class="status warning">Needs attention</span></header><div class="error-state"><span class="error-symbol" aria-hidden="true">!</span><h3>{label}</h3><p>{description}</p><a href="/dashboard">Try again <span aria-hidden="true">↗</span></a></div>"#
                );
            }
        }
        html.push_str("</section>");
    }
    html.push_str(r#"</div><footer class="footnote"><span>DATABASE TABLE ROW COUNTS ARE ESTIMATES</span><p>PostgreSQL statistics may lag behind writes. Table storage includes TOAST and auxiliary storage; totals include indexes. Reports cover each service’s active schema.</p></footer></main>"#);
    layout::end(&mut html);
    html
}
fn render_entries(html: &mut String, service: &str, report: Result<EntryStatuses, ReportError>) {
    let image = match service {
        "Retailer Sourcing" => "retailer-sourcing",
        "Offers" => "offers",
        "Customer Facing" => "customer-facing",
        _ => unreachable!("queue sections use the three reporting services"),
    };
    let _ = write!(
        html,
        r#"<section class="service queue-service" aria-label="{service} queue status"><header class="queue-service-heading"><h2><img class="queue-service-image" src="/assets/{image}.svg" alt="{service} queue status" width="128" height="128"></h2></header><div class="queue-service-content">"#
    );
    match report {
        Ok(report) => {
            let date = match report.window_date() {
                Ok(Some(date)) => date,
                Ok(None) => {
                    html.push_str("<p>Upgrade this service to report daily counts. This report does not identify its counting window.</p></div></section>");
                    return;
                }
                Err(_) => {
                    html.push_str("<p>Invalid reporting window. Daily counts unavailable.</p></div></section>");
                    return;
                }
            };
            let _ = write!(
                html,
                r#"<div class="report-meta">Report UTC date: {}. Updated <time datetime="{}">{}</time></div>"#,
                date,
                escape(&report.generated_at),
                escape(&report.generated_at)
            );
            for (title, entries) in [
                ("Inbox entries", report.inbox_entries),
                ("Command entries", Some(report.command_entries)),
                ("Event entries", Some(report.event_entries)),
                ("Outbox entries", report.outbox_entries),
            ] {
                if let Some(entries) = entries {
                    render_entry_boxes(html, title, &entries);
                } else {
                    render_queue_start(html, title, false);
                    html.push_str("<p>Upgrade this service to report these counts.</p></section>");
                }
            }
        }
        Err(error) => {
            let message = match error {
                ReportError::Unavailable => "Status counts unavailable. Try refreshing shortly.",
                ReportError::Authentication => {
                    "Authentication failed for status counts. Check this service’s token."
                }
                ReportError::Malformed => {
                    "Invalid response for status counts. Check that this service is up to date."
                }
                ReportError::Configuration => {
                    "Configuration needed for status counts. Check this service’s address and token."
                }
            };
            let _ = write!(html, r#"<div class="error-state"><p>{message}</p></div>"#);
        }
    }
    html.push_str("</div></section>");
}

fn render_queue_start(html: &mut String, title: &str, empty: bool) {
    let row_class = if empty { " queue-row-empty" } else { "" };
    let image = match title {
        "Inbox entries" => "inbox",
        "Command entries" => "commands",
        "Event entries" => "events",
        "Outbox entries" => "outbox",
        _ => unreachable!("queue titles are defined by the dashboard"),
    };
    let _ = write!(
        html,
        r#"<section class="command-summary queue-row{row_class}" aria-label="{title}"><h3><img class="queue-icon" src="/assets/{image}.svg" alt="{title}" width="64" height="64"></h3>"#
    );
}

fn render_entry_boxes(html: &mut String, title: &str, entries: &[client::StatusCount]) {
    let statuses = [
        (0, "New", "new"),
        (2, "Processing", "processing"),
        (4, "Failed", "failed"),
        (5, "Completed", "completed"),
        (7, "Dead", "dead"),
        (8, "Archive", "archive"),
    ];
    let empty = entries.iter().all(|entry| entry.count == 0);
    render_queue_start(html, title, empty);
    let empty_class = if empty { " command-empty" } else { "" };
    html.push_str(r#"<dl class="command-boxes">"#);
    for (code, label, class) in statuses {
        let entry = entries.iter().find(|entry| entry.status == code);
        let count = entry.map_or(0, |entry| entry.count);
        let source_label = entry.map_or(label, |entry| entry.status_text.as_str());
        let _ = write!(
            html,
            r#"<div class="command-box command-{class}{empty_class}" title="{} (status {code})"><dt>{label}</dt><dd>{}</dd></div>"#,
            escape(source_label),
            queue_count(count)
        );
    }
    for entry in entries
        .iter()
        .filter(|entry| !statuses.iter().any(|(code, _, _)| *code == entry.status))
    {
        let _ = write!(
            html,
            r#"<div class="command-box command-unknown{empty_class}"><dt>{} (status {})</dt><dd>{}</dd></div>"#,
            escape(&entry.status_text),
            entry.status,
            queue_count(entry.count)
        );
    }
    html.push_str("</dl></section>");
}

fn queue_count(count: u64) -> String {
    if count == 0 {
        r#"<span class="sr-only">0</span>"#.into()
    } else {
        number(count as u128)
    }
}

fn render_report(html: &mut String, report: &Report) {
    let RowCountKind::Estimated = report.row_count_kind;
    let rows: u128 = report.tables.iter().map(|t| t.row_count as u128).sum();
    let table_bytes: u128 = report.tables.iter().map(|t| t.table_bytes as u128).sum();
    let index_bytes: u128 = report.tables.iter().map(|t| t.index_bytes as u128).sum();
    let total = table_bytes + index_bytes;
    let percent = if total == 0 {
        0.0
    } else {
        table_bytes as f64 / total as f64 * 100.0
    };
    let _ = write!(
        html,
        r#"<div class="report-meta"><span>Schema <code>{}</code></span><span>Updated <time datetime="{}">{}</time></span></div><div class="service-metrics"><div><span>Tables</span><strong>{}</strong></div><div><span>Rows <abbr title="Estimated live rows">est.</abbr></span><strong>{}</strong></div><div><span>Total storage</span><strong>{}</strong></div></div><div class="storage-breakdown"><div class="storage-track" role="img" aria-label="Table storage {}, index storage {}"><span style="width:{percent:.2}%"></span></div><div class="legend"><span><i class="data-key"></i>Tables {}</span><span><i class="index-key"></i>Indexes {}</span></div></div>"#,
        escape(&report.schema_name),
        escape(&report.generated_at),
        escape(&report.generated_at.replace('T', " ")),
        report.tables.len(),
        number(rows),
        size(total),
        bytes(table_bytes),
        bytes(index_bytes),
        size(table_bytes),
        size(index_bytes)
    );
    if report.tables.is_empty() {
        html.push_str(r#"<div class="empty-state"><h3>No tables yet</h3><p>This schema has no ordinary tables to report.</p></div>"#);
        return;
    }
    html.push_str(r#"<div class="table-scroll" tabindex="0" role="region" aria-label="Table statistics"><table><caption class="sr-only">Storage and estimated rows, ordered alphabetically by table name</caption><thead><tr><th scope="col" aria-sort="ascending">Table <span aria-hidden="true">↑</span></th><th scope="col">Rows <abbr title="Estimated">est.</abbr></th><th scope="col">Table size</th><th scope="col">Indexes</th><th scope="col">Total</th></tr></thead><tbody>"#);
    let mut tables: Vec<_> = report.tables.iter().collect();
    tables.sort_by(|a, b| a.table_name.cmp(&b.table_name));
    let max = tables
        .iter()
        .map(|table| table.total_bytes)
        .max()
        .unwrap_or(0);
    for table in tables {
        let percent = if max == 0 {
            0.0
        } else {
            table.total_bytes as f64 / max as f64 * 100.0
        };
        let _ = write!(
            html,
            r#"<tr><th scope="row"><code>{}</code><span class="row-track" aria-hidden="true"><i style="width:{percent:.2}%"></i></span></th><td>{}</td><td>{}</td><td>{}</td><td class="total">{}</td></tr>"#,
            escape(&table.table_name),
            number(table.row_count as u128),
            size(table.table_bytes as u128),
            size(table.index_bytes as u128),
            size(table.total_bytes as u128)
        );
    }
    html.push_str("</tbody></table></div>");
}
fn size(value: u128) -> String {
    format!(
        r#"<span title="{} bytes" aria-label="{} bytes">{}</span>"#,
        number(value),
        number(value),
        bytes(value)
    )
}
fn bytes(value: u128) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let mut amount = value as f64;
    let mut unit = 0;
    while amount >= 1024.0 && unit < units.len() - 1 {
        amount /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{value} B")
    } else {
        format!("{amount:.1} {}", units[unit])
    }
}
fn number(value: u128) -> String {
    let digits = value.to_string();
    let mut output = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            output.push(',');
        }
        output.push(c);
    }
    output
}
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
#[cfg(test)]
mod tests;
