//! Server-rendered reporting dashboard. Service credentials never reach the browser.
mod client;
use client::{Report, ReportError, ReportingClients, RowCountKind};
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
    let (sourcing, storage) = state.clients.reports().await;
    let mut response = Html(render(sourcing, storage)).into_response();
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
}

fn render(sourcing: Result<Report, ReportError>, storage: Result<Report, ReportError>) -> String {
    let reports = [
        (&sourcing, "Retailer Sourcing", "Pipeline metadata", "01"),
        (&storage, "Object Storage", "Stored artifacts", "02"),
    ];
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
    let mut html = String::from(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Database overview · Development</title><style>"#,
    );
    html.push_str(include_str!("dashboard/style.css"));
    html.push_str(r#"</style></head><body><header class="topbar"><a class="brand" href="/"><span class="brand-mark" aria-hidden="true">b.</span> Development <span class="workspace">WORKSPACE</span></a><nav aria-label="Main"><a href="/">Home</a><a class="active" href="/dashboard" aria-current="page">Dashboard</a></nav></header><main><div class="hero"><div><p class="eyebrow">SYSTEM VISIBILITY</p><h1>Database overview<span class="title-dot">.</span></h1><p class="lede">A clear view of the tables behind your pipeline.</p></div><a class="refresh" href="/dashboard"><span aria-hidden="true">↻</span> Refresh reports</a></div>"#);
    let _ = write!(
        html,
        r#"<section class="overview" aria-label="Overview"><div><span class="metric-label">Services available</span><strong>{available}<small> / 2</small></strong></div><div><span class="metric-label">Tables reported</span><strong>{total_tables}</strong></div><div><span class="metric-label">Storage reported</span><strong>{}</strong></div><div class="refresh-note"><span class="mini-dot" aria-hidden="true"></span><div><b>Refresh on demand</b><span>Reports update when you load this page.</span></div></div></section>"#,
        bytes(total_bytes)
    );
    if available < 2 {
        html.push_str(
            r#"<p class="partial-note">Summary totals include available services only.</p>"#,
        );
    }
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
    html.push_str(r#"</div><footer class="footnote"><span>ROW COUNTS ARE ESTIMATES</span><p>PostgreSQL statistics may lag behind writes. Table storage includes TOAST and auxiliary storage; totals include indexes. Reports cover each service’s active schema.</p></footer></main></body></html>"#);
    html
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
