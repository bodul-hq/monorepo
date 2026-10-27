use super::*;
use crate::offers::tests::{fixture, page};
use money::Currency;
use poem::test::TestClient;
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
            assert!(html.contains("19.99 AUD"));
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
    assert!(html.contains("&lt;script&gt;alert(&quot;x&quot;)"));
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
fn price_formatting_preserves_all_minor_units() {
    for (amount, currency, expected) in [
        (1999, Currency::AUD, "19.99 AUD"),
        (0, Currency::EUR, "0.00 EUR"),
        (-1, Currency::USD, "−0.01 USD"),
        (123, Currency::JPY, "123 JPY"),
        (123, Currency::KRW, "123 KRW"),
        (123, Currency::VND, "123 VND"),
        (199999, Currency::INR, "1999.99 INR"),
        (i64::MIN, Currency::EUR, "−92233720368547758.08 EUR"),
        (i64::MAX, Currency::USD, "92233720368547758.07 USD"),
        (i64::MIN, Currency::JPY, "−9223372036854775808 JPY"),
    ] {
        assert_eq!(
            format_price(&Price {
                amount: amount.to_string(),
                currency
            }),
            expected
        );
    }
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
