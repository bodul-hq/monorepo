use super::*;
use crate::offers::tests::{fixture, page};
use poem::test::TestClient;
use scraper::{Html as ParsedHtml, Selector};
use std::time::Duration;

#[tokio::test]
async fn homepage_fetches_and_renders_thirty_then_one_without_scripts() {
    for (offset, count, more) in [(0, 30, true), (30, 1, false), (60, 0, false)] {
        let (base, task) = fixture(
            200,
            page(count, offset, more).to_string(),
            &format!("/api/offers?limit=30&offset={offset}"),
            Duration::ZERO,
        );
        let client = TestClient::new(crate::app(Arc::new(
            OffersClient::new(&base, Duration::from_secs(1)).unwrap(),
        )));
        let response = client.get(page_url(offset)).send().await;
        response.assert_status_is_ok();
        assert_eq!(response.0.headers()["cache-control"], "no-store");
        assert!(
            response.0.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/html")
        );
        let html = response.0.into_body().into_string().await.unwrap();
        assert_eq!(html.matches("class=\"offer-card\"").count(), count);
        assert!(!html.contains("<script"));
        assert!(html.contains("Latest offers"));
        assert_eq!(html.contains("rel=\"next\""), more);
        assert_eq!(html.contains("rel=\"prev\""), offset > 0);
        if count > 0 {
            assert!(html.contains("<span aria-hidden=\"true\">A$19.99</span>"));
            assert!(html.contains("19.99 AUD"));
        }
        for anchor in html.split("<a ").skip(1) {
            let attributes = anchor.split('>').next().unwrap();
            let outbound = attributes.contains("class=\"offer-link\"");
            assert_eq!(attributes.contains("target=\"_blank\""), outbound);
            assert_eq!(attributes.contains("rel=\"noopener\""), outbound);
            if outbound {
                assert!(
                    anchor
                        .split("</a>")
                        .next()
                        .unwrap()
                        .contains("opens in a new tab")
                );
            }
        }
        if offset == 0 {
            assert!(html.contains("href=\"/?offset=30\""));
        }
        if offset == 30 {
            assert!(html.contains("rel=\"prev\" href=\"/\""));
        }
        task.join().unwrap();
    }
}

#[tokio::test]
async fn invalid_queries_are_html_errors_without_upstream_requests() {
    let client = TestClient::new(crate::app(Arc::new(
        OffersClient::new("http://127.0.0.1:1", Duration::from_millis(20)).unwrap(),
    )));
    for query in [
        "offset=-1",
        "offset=2147483648",
        "offset=1.5",
        "offset=abc",
        "offset=",
        "offset=0&offset=30",
    ] {
        let response = client.get(format!("/?{query}")).send().await;
        response.assert_status(StatusCode::BAD_REQUEST);
        assert!(
            response
                .0
                .into_body()
                .into_string()
                .await
                .unwrap()
                .contains("Invalid page")
        );
    }
}

#[tokio::test]
async fn unavailable_response_is_styled_sanitized_and_retryable() {
    let (base, task) = fixture(
        500,
        "private database diagnostics".into(),
        "/api/offers?limit=30&offset=30",
        Duration::ZERO,
    );
    let client = TestClient::new(crate::app(Arc::new(
        OffersClient::new(&base, Duration::from_secs(1)).unwrap(),
    )));
    let response = client.get("/?offset=30").send().await;
    response.assert_status(StatusCode::SERVICE_UNAVAILABLE);
    let html = response.0.into_body().into_string().await.unwrap();
    assert!(html.contains("temporarily unavailable"));
    assert!(html.contains("href=\"/?offset=30\">Try again"));
    assert!(!html.contains("private"));
    task.join().unwrap();
}

#[test]
fn escapes_values_and_only_links_safe_absolute_urls() {
    let mut data: Page = serde_json::from_value(page(1, 0, false)).unwrap();
    let offer = &mut data.items[0];
    offer.title = "<script>alert(\"x\")</script> & 'quoted'".into();
    offer.retailer_code = "<img src=x onerror=alert(1)>".into();
    let html = render(&data);
    assert!(!html.contains("<script"));
    assert!(!html.contains("<img"));
    assert!(html.contains("<h3>&amp; &#39;quoted&#39;</h3>"));
    assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
    assert!(!html.contains("alert(&quot;x&quot;)"));
    assert!(html.contains("&amp;variant=2#details"));
    for url in [
        "javascript:alert(1)",
        "data:text/html,bad",
        "//example.com",
        "/relative",
        "https://user:pass@example.com",
        " https://example.com",
        "https://example.com\n",
    ] {
        data.items[0].url = url.into();
        let html = render(&data);
        assert!(!html.contains("class=\"offer-link\""), "{url}");
        assert!(html.contains("Store link unavailable"));
    }
    assert!(safe_link("http://example.com"));
    assert!(safe_link("https://example.com/a?b=2#details"));
}

#[test]
fn empty_states_and_navigation_handle_boundaries() {
    let empty: Page = serde_json::from_value(page(0, 0, false)).unwrap();
    assert!(render(&empty).contains("no offers here yet"));
    assert!(!render(&empty).contains("class=\"pagination\""));
    let mut data: Page = serde_json::from_value(page(30, 1, true)).unwrap();
    assert!(render(&data).contains("rel=\"prev\" href=\"/\""));
    data.offset = i32::MAX;
    assert!(!render(&data).contains("rel=\"next\""));
}

#[test]
fn first_image_only_with_escaping_and_safe_empty_states() {
    let mut data: Page = serde_json::from_value(page(1, 0, false)).unwrap();
    assert!(render(&data).contains("empty-image"));
    data.items[0].title = "A <charger> & \"test\"".into();
    data.items[0].images = vec![
        "https://shop.example/front?a=1&b=2".into(),
        "https://shop.example/back".into(),
    ];
    let html = render(&data);
    assert!(html.contains("src=\"https://shop.example/front?a=1&amp;b=2\""));
    assert!(html.contains("alt=\"A &amp; &quot;test&quot;\""));
    assert!(html.contains("loading=\"lazy\""));
    assert!(!html.contains("shop.example/back"));
    for url in [
        "javascript:alert(1)",
        "//shop.example/front",
        "https://user@shop.example/front",
        "https://@shop.example/front",
        " https://shop.example/front",
        "https://shop.example/front\n",
    ] {
        data.items[0].images[0] = url.into();
        let html = render(&data);
        assert!(!html.contains("<img "));
        assert!(html.contains("empty-image"));
        assert!(html.contains("View offer"));
    }
}

#[test]
fn retailer_overlay_is_escaped_and_accessible_with_or_without_an_image() {
    let mut data: Page = serde_json::from_value(page(1, 0, false)).unwrap();
    data.items[0].retailer_code = "shop<&\"example".into();
    for images in [
        vec!["https://shop.example/front.jpg".into()],
        Vec::new(),
        vec!["javascript:alert(1)".into()],
    ] {
        data.items[0].images = images;
        let html = render(&data);
        let image_area = html
            .split("class=\"offer-image")
            .nth(1)
            .unwrap()
            .split("</div>")
            .next()
            .unwrap();
        assert!(
            image_area.contains(
                "<span class=\"retailer\"><span>shop&lt;&amp;&quot;example</span></span>"
            )
        );
        assert!(!image_area.contains("aria-hidden=\"true\""));
        assert_eq!(html.matches("class=\"retailer\"").count(), 1);
        assert!(!html.contains("shop<&"));
    }
}

#[test]
fn cleaned_title_is_shared_by_heading_image_and_link_without_mutating_data() {
    for (raw, expected) in [
        (
            "<b>USB</b><br>charger &amp; \"cable\"",
            "USB charger & \"cable\"",
        ),
        (
            "<style>hidden</style><script>bad</script><template>hidden</template>",
            "Untitled offer",
        ),
        (
            "&lt;img src=x onerror=alert(1)&gt; &quot;",
            "<img src=x onerror=alert(1)> \"",
        ),
    ] {
        for has_image in [false, true] {
            for safe_url in [false, true] {
                let mut data: Page = serde_json::from_value(page(1, 0, false)).unwrap();
                data.items[0].title = raw.into();
                if has_image {
                    data.items[0].images = vec!["https://shop.example/front.jpg".into()];
                }
                if !safe_url {
                    data.items[0].url = "javascript:bad".into();
                }
                let html = render(&data);
                let document = ParsedHtml::parse_document(&html);
                let heading = document
                    .select(&Selector::parse(".offer-card h3").unwrap())
                    .next()
                    .unwrap();
                assert_eq!(heading.text().collect::<String>(), expected);
                assert_eq!(heading.child_elements().count(), 0);
                let image_selector = Selector::parse(".offer-card img").unwrap();
                let images: Vec<_> = document.select(&image_selector).collect();
                assert_eq!(images.len(), usize::from(has_image));
                if has_image {
                    assert_eq!(images[0].attr("alt"), Some(expected));
                    assert_eq!(images[0].attr("onerror"), None);
                }
                let link_selector = Selector::parse(".offer-link .sr-only").unwrap();
                let links: Vec<_> = document.select(&link_selector).collect();
                assert_eq!(links.len(), usize::from(safe_url));
                if safe_url {
                    assert_eq!(
                        links[0].text().collect::<String>(),
                        format!(" for {expected} (opens in a new tab)")
                    );
                }
                assert_eq!(document.select(&Selector::parse(".offer-card script, .offer-card style, .offer-card template, [onerror]").unwrap()).count(), 0);
                assert_eq!(data.items[0].title, raw);
            }
        }
    }
}

#[test]
fn mixed_currency_prices_expose_one_accessible_amount_with_iso_currency() {
    let mut data: Page = serde_json::from_value(page(3, 0, false)).unwrap();
    let cases = [
        (money::Currency::USD, "$1,234.56", "1234.56 USD"),
        (money::Currency::EUR, "1.234,56\u{a0}€", "1234.56 EUR"),
        (money::Currency::JPY, "¥123,456", "123456 JPY"),
    ];
    for (offer, (currency, _, _)) in data.items.iter_mut().zip(cases) {
        offer.price.currency = currency;
        offer.price.amount = "123456".into();
    }
    let document = ParsedHtml::parse_document(&render(&data));
    let prices = Selector::parse(".price").unwrap();
    for (price, (_, visible, accessible)) in document.select(&prices).zip(cases) {
        let children: Vec<_> = price.child_elements().collect();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].attr("aria-hidden"), Some("true"));
        assert_eq!(children[0].text().collect::<String>(), visible);
        assert_eq!(children[1].attr("class"), Some("sr-only"));
        assert_eq!(children[1].attr("aria-hidden"), None);
        assert_eq!(children[1].text().collect::<String>(), accessible);
    }
}
