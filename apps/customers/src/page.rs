use std::{fmt::Write, sync::Arc};

use crate::offers::{OffersClient, PAGE_SIZE, Page, Price};
use poem::{
    IntoResponse, Response, handler,
    http::StatusCode,
    web::{Data, Html, Query},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Parameters {
    #[serde(default)]
    offset: i32,
}

#[handler]
pub async fn index(
    parameters: poem::Result<Query<Parameters>>,
    Data(client): Data<&Arc<OffersClient>>,
) -> Response {
    let offset = match parameters {
        Ok(Query(parameters)) if parameters.offset >= 0 => parameters.offset,
        _ => {
            return response(
                StatusCode::BAD_REQUEST,
                error_page(
                    "Invalid page",
                    "This page address is not valid. Please return to the latest offers.",
                    "/",
                    "Latest offers",
                ),
            );
        }
    };
    match client.fetch(offset).await {
        Ok(page) => response(StatusCode::OK, render(&page)),
        Err(()) => response(
            StatusCode::SERVICE_UNAVAILABLE,
            error_page(
                "Offers are temporarily unavailable",
                "We couldn’t load the latest offers. Please try again in a moment.",
                &page_url(offset),
                "Try again",
            ),
        ),
    }
}

fn response(status: StatusCode, body: String) -> Response {
    let mut response = Html(body).into_response();
    response.set_status(status);
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
}

fn start() -> String {
    let mut html = String::from(
        r##"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="description" content="Browse the latest offers from retailers in one place."><title>Latest offers · Bodul</title><style>"##,
    );
    html.push_str(include_str!("style.css"));
    html.push_str(r##"</style></head><body><a class="skip-link" href="#main">Skip to offers</a><header class="site-header"><a class="brand" href="/" aria-label="Bodul home"><span class="brand-mark" aria-hidden="true">b.</span>Bodul</a><span class="header-note">A little discovery. A good find.</span></header><main id="main"><section class="intro"><p class="eyebrow">THE LATEST FINDS</p><h1>Good things, worth a look.</h1><p class="intro-copy">Explore offers from retailers, all in one place.<br> Find something you like and visit the store to make it yours.</p></section>"##);
    html
}

fn end(html: &mut String) {
    html.push_str("</main><footer><a class=\"footer-brand\" href=\"/\">Bodul</a><p>Prices are shown in the retailer’s currency. Check the store for current pricing and availability.</p></footer></body></html>");
}

pub fn render(page: &Page) -> String {
    let mut html = start();
    html.push_str(r#"<section aria-labelledby="offers-heading"><div class="section-heading"><h2 id="offers-heading">Latest offers</h2><span class="sort-label">Newest first</span></div>"#);
    if page.items.is_empty() {
        html.push_str(if page.offset == 0 {
            r#"<div class="empty-state"><span class="empty-mark" aria-hidden="true">↗</span><h3>More good finds are on the way.</h3><p>There are no offers here yet. Come back soon to see what’s new.</p></div>"#
        } else {
            r#"<div class="empty-state"><h3>You’ve reached the end.</h3><p>Head back to explore the latest offers.</p><a class="button" href="/">Latest offers</a></div>"#
        });
    } else {
        html.push_str(r#"<ul class="offers-grid">"#);
        for offer in &page.items {
            let _ = write!(
                html,
                r#"<li class="offer-card"><article><span class="retailer">{}</span><h3>{}</h3><div class="card-bottom"><p class="price">{}</p>"#,
                escape(&offer.retailer_code),
                escape(if offer.title.trim().is_empty() {
                    "Untitled offer"
                } else {
                    &offer.title
                }),
                escape(&format_price(&offer.price))
            );
            if safe_link(&offer.url) {
                let _ = write!(
                    html,
                    r#"<a class="offer-link" href="{}">View offer <span aria-hidden="true">↗</span><span class="sr-only"> for {}</span></a>"#,
                    escape(&offer.url),
                    escape(&offer.title)
                );
            } else {
                html.push_str(r#"<span class="link-unavailable">Store link unavailable</span>"#);
            }
            html.push_str("</div></article></li>");
        }
        html.push_str("</ul>");
    }
    let next = page.offset.checked_add(PAGE_SIZE).filter(|_| page.has_more);
    if page.offset > 0 || next.is_some() {
        html.push_str(r#"<nav class="pagination" aria-label="Offer pages">"#);
        if page.offset > 0 {
            let _ = write!(
                html,
                r#"<a class="button previous" rel="prev" href="{}">← Previous</a>"#,
                page_url(page.offset.saturating_sub(PAGE_SIZE).max(0))
            );
        }
        if let Some(offset) = next {
            let _ = write!(
                html,
                r#"<a class="button next" rel="next" href="{}">Next <span aria-hidden="true">→</span></a>"#,
                page_url(offset)
            );
        }
        html.push_str("</nav>");
    }
    html.push_str("</section>");
    end(&mut html);
    html
}

fn error_page(title: &str, message: &str, href: &str, label: &str) -> String {
    let mut html = start();
    let _ = write!(
        html,
        r#"<section class="empty-state" aria-labelledby="error-heading"><h2 id="error-heading">{}</h2><p>{}</p><a class="button" href="{}">{}</a></section>"#,
        escape(title),
        escape(message),
        escape(href),
        escape(label)
    );
    end(&mut html);
    html
}

fn page_url(offset: i32) -> String {
    if offset == 0 {
        "/".into()
    } else {
        format!("/?offset={offset}")
    }
}

fn safe_link(value: &str) -> bool {
    // Preserve the exact original URL in the attribute after validating it.
    value.trim() == value
        && !value.chars().any(char::is_control)
        && reqwest::Url::parse(value).is_ok_and(|url| {
            matches!(url.scheme(), "http" | "https")
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        })
}

fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn format_price(price: &Price) -> String {
    // The HTTP client has validated the canonical i64 representation.
    let amount = price.amount.parse::<i64>().expect("validated price amount");
    let magnitude = amount.unsigned_abs();
    let exponent = price.currency.exponent();
    let scale = 10_u64.pow(u32::from(exponent));
    let sign = if amount < 0 { "−" } else { "" };
    if exponent == 0 {
        format!("{sign}{magnitude} {}", price.currency.as_str())
    } else {
        format!(
            "{sign}{}.{:0width$} {}",
            magnitude / scale,
            magnitude % scale,
            price.currency.as_str(),
            width = usize::from(exponent)
        )
    }
}

#[cfg(test)]
mod tests;
