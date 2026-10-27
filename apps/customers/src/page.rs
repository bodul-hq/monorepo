use std::{fmt::Write, sync::Arc};

use crate::offers::{OffersClient, PAGE_SIZE, Page};
use poem::{
    IntoResponse, Response, handler,
    http::StatusCode,
    web::{Data, Html, Query},
};
use serde::Deserialize;

mod price;
mod title;

use price::format_price;
use title::display_title;

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

#[derive(Deserialize, Default)]
struct SearchParameters {
    #[serde(default)]
    q: String,
    offset: Option<String>,
}

#[handler]
pub async fn search(
    parameters: poem::Result<Query<SearchParameters>>,
    Data(client): Data<&Arc<OffersClient>>,
) -> Response {
    let parameters = match parameters {
        Ok(Query(parameters)) => parameters,
        Err(_) => {
            return response(
                StatusCode::BAD_REQUEST,
                search_message(
                    "",
                    "Invalid search",
                    "Check the search address and try again.",
                    "/search",
                ),
            );
        }
    };
    let offset = match parameters.offset.as_deref().unwrap_or("0").parse::<i32>() {
        Ok(offset) if offset >= 0 => offset,
        _ => {
            return response(
                StatusCode::BAD_REQUEST,
                search_message(
                    &parameters.q,
                    "Invalid page",
                    "Offset must be between 0 and 2147483647.",
                    "/search",
                ),
            );
        }
    };
    if parameters.q.trim().is_empty() {
        return response(
            StatusCode::OK,
            search_message(
                &parameters.q,
                "Search offers",
                "Enter a title or a few words in the search field.",
                "/",
            ),
        );
    }
    let query = match crate::offers::normalize_query(&parameters.q) {
        Ok(query) => query,
        Err(message) => {
            return response(
                StatusCode::BAD_REQUEST,
                search_message(&parameters.q, "Invalid search", message, "/search"),
            );
        }
    };
    match client.search(&query, offset).await {
        Ok(page) => response(StatusCode::OK, render_page(&page, Some(&query))),
        Err(()) => response(
            StatusCode::SERVICE_UNAVAILABLE,
            search_message(
                &query,
                "Search is temporarily unavailable",
                "Please try again in a moment.",
                &search_url(&query, offset),
            ),
        ),
    }
}

fn search_message(query: &str, title: &str, message: &str, href: &str) -> String {
    error_page_with_query(
        title,
        message,
        href,
        if href == "/" {
            "Latest offers"
        } else {
            "Try again"
        },
        Some(query),
    )
}

fn search_url(query: &str, offset: i32) -> String {
    let mut url = reqwest::Url::parse("http://localhost/search").unwrap();
    url.query_pairs_mut()
        .append_pair("q", query)
        .append_pair("offset", &offset.to_string());
    format!("{}?{}", url.path(), url.query().unwrap())
}

fn result_url(query: Option<&str>, offset: i32) -> String {
    query.map_or_else(|| page_url(offset), |query| search_url(query, offset))
}

fn response(status: StatusCode, body: String) -> Response {
    let mut response = Html(body).into_response();
    response.set_status(status);
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
}

fn start(query: Option<&str>) -> String {
    let mut html = String::from(
        r##"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="description" content="Browse the latest offers from retailers in one place."><title>Latest offers · Bodul</title><style>"##,
    );
    if query.is_some() {
        html = html.replace("Latest offers · Bodul", "Search offers · Bodul");
    }
    html.push_str(include_str!("style.css"));
    html.push_str(r##"</style></head><body><a class="skip-link" href="#main">Skip to offers</a><header class="site-header"><a class="brand" href="/" aria-label="Bodul home"><span class="brand-mark" aria-hidden="true">b.</span>Bodul</a>"##);
    let _ = write!(
        html,
        r#"<form class="search-form" role="search" action="/search" method="get"><label class="sr-only" for="offer-query">Search offer titles</label><input id="offer-query" type="search" name="q" value="{}" placeholder="Search offers" required><button class="button" type="submit">Search</button></form>"#,
        escape(query.unwrap_or(""))
    );
    html.push_str(r#"<span class="header-note">A little discovery. A good find.</span></header><main id="main">"#);
    if let Some(query) = query {
        let _ = write!(
            html,
            r#"<section class="intro"><p class="eyebrow">FIND SOMETHING GOOD</p><h1>Search offers</h1><p class="intro-copy">Results for “{}”</p></section>"#,
            escape(query)
        );
    } else {
        html.push_str(r#"<section class="intro"><p class="eyebrow">THE LATEST FINDS</p><h1>Good things, worth a look.</h1><p class="intro-copy">Explore offers from retailers, all in one place.<br> Find something you like and visit the store to make it yours.</p></section>"#);
    }
    html
}

fn end(html: &mut String) {
    html.push_str("</main><footer><a class=\"footer-brand\" href=\"/\">Bodul</a><p>Prices are shown in the retailer’s currency. Check the store for current pricing and availability.</p></footer></body></html>");
}

pub fn render(page: &Page) -> String {
    render_page(page, None)
}

fn render_page(page: &Page, query: Option<&str>) -> String {
    let mut html = start(query);
    let (heading, order) = if query.is_some() {
        ("Search results", "Most relevant first")
    } else {
        ("Latest offers", "Newest first")
    };
    let _ = write!(
        html,
        r#"<section aria-labelledby="offers-heading"><div class="section-heading"><h2 id="offers-heading">{heading}</h2><span class="sort-label">{order}</span></div>"#
    );
    if page.items.is_empty() {
        if let Some(query) = query {
            if page.offset == 0 {
                html.push_str(r#"<div class="empty-state"><h3>No matching offers.</h3><p>Try another title or fewer words.</p></div>"#);
            } else {
                let _ = write!(
                    html,
                    r#"<div class="empty-state"><h3>You’ve reached the end.</h3><a class="button" href="{}">First results</a></div>"#,
                    escape(&search_url(query, 0))
                );
            }
        } else {
            html.push_str(if page.offset == 0 {
            r#"<div class="empty-state"><span class="empty-mark" aria-hidden="true">↗</span><h3>More good finds are on the way.</h3><p>There are no offers here yet. Come back soon to see what’s new.</p></div>"#
        } else {
            r#"<div class="empty-state"><h3>You’ve reached the end.</h3><p>Head back to explore the latest offers.</p><a class="button" href="/">Latest offers</a></div>"#
        });
        }
    } else {
        html.push_str(r#"<ul class="offers-grid">"#);
        for offer in &page.items {
            let title = display_title(&offer.title);
            let price = format_price(&offer.price);
            let _ = write!(
                html,
                r#"<li class="offer-card"><article>{}<h3>{}</h3><div class="card-bottom"><p class="price"><span aria-hidden="true">{}</span><span class="sr-only">{}</span></p>"#,
                image_html(offer, &title),
                escape(&title),
                escape(&price.visible),
                escape(&price.accessible)
            );
            if safe_link(&offer.url) {
                let _ = write!(
                    html,
                    r#"<a class="offer-link" href="{}" target="_blank" rel="noopener">View offer <span aria-hidden="true">↗</span><span class="sr-only"> for {} (opens in a new tab)</span></a>"#,
                    escape(&offer.url),
                    escape(&title)
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
                escape(&result_url(
                    query,
                    page.offset.saturating_sub(PAGE_SIZE).max(0)
                ))
            );
        }
        if let Some(offset) = next {
            let _ = write!(
                html,
                r#"<a class="button next" rel="next" href="{}">Next <span aria-hidden="true">→</span></a>"#,
                escape(&result_url(query, offset))
            );
        }
        html.push_str("</nav>");
    }
    html.push_str("</section>");
    end(&mut html);
    html
}

fn error_page(title: &str, message: &str, href: &str, label: &str) -> String {
    error_page_with_query(title, message, href, label, None)
}

fn error_page_with_query(
    title: &str,
    message: &str,
    href: &str,
    label: &str,
    query: Option<&str>,
) -> String {
    let mut html = start(query);
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

fn image_html(offer: &crate::offers::Offer, title: &str) -> String {
    let retailer = format!(
        r#"<span class="retailer"><span>{}</span></span>"#,
        escape(&offer.retailer_code)
    );
    match offer
        .images
        .first()
        .filter(|url| url.len() <= 2048 && safe_link(url))
    {
        Some(url) => format!(
            r#"<div class="offer-image"><img src="{}" alt="{}" loading="lazy" width="480" height="360">{retailer}</div>"#,
            escape(url),
            escape(title)
        ),
        None => format!(r#"<div class="offer-image empty-image">{retailer}</div>"#),
    }
}

fn safe_link(value: &str) -> bool {
    // Preserve the exact original URL in the attribute after validating it.
    !value.split_once("://").is_some_and(|(_, rest)| {
        rest.split(['/', '?', '#'])
            .next()
            .unwrap_or("")
            .contains('@')
    }) && value.trim() == value
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

#[cfg(test)]
mod tests;
