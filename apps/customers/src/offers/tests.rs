use super::*;
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

pub(crate) fn page(count: usize, offset: i32, more: bool) -> Value {
    json!({"items":(0..count).map(|i| json!({
        "id":format!("00000000-0000-0000-0000-{i:012}"),"retailer_code":"ankerau",
        "title":format!("USB charger {i}"),"price":{"amount":"1999","currency":"AUD"},
        "url":"https://shop.example/Charger?a=1&variant=2#details",
        "created_at":"2026-10-24T12:00:00.123456Z"
    })).collect::<Vec<_>>(),"limit":30,"offset":offset,"has_more":more})
}

pub(crate) fn fixture(
    status: u16,
    body: String,
    expected_path: &str,
    delay: Duration,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let path = expected_path.to_string();
    let task = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut request = Vec::new();
        while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            let mut buffer = [0; 1024];
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0);
            request.extend_from_slice(&buffer[..count]);
        }
        let request = String::from_utf8(request).unwrap();
        assert!(
            request.starts_with(&format!("GET {path} HTTP/1.1\r\n")),
            "{request}"
        );
        thread::sleep(delay);
        let _ = write!(
            stream,
            "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nLocation: http://127.0.0.1:1/private\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
    });
    (base, task)
}

#[tokio::test]
async fn exact_request_path_prefix_and_decimal_amounts() {
    let mut data = page(1, 30, false);
    data["items"][0]["price"]["amount"] = json!(i64::MIN.to_string());
    let (base, task) = fixture(
        200,
        data.to_string(),
        "/prefix/api/offers?limit=30&offset=30",
        Duration::ZERO,
    );
    let client = OffersClient::new(&format!("{base}/prefix/"), Duration::from_secs(1)).unwrap();
    let result = client.fetch(30).await.unwrap();
    assert_eq!(result.items[0].price.amount, i64::MIN.to_string());
    assert_eq!(
        result.items[0].url,
        "https://shop.example/Charger?a=1&variant=2#details"
    );
    task.join().unwrap();
}

#[tokio::test]
async fn errors_redirects_timeouts_and_malformed_contracts_fail() {
    let valid = page(1, 0, false);
    let mut malformed = vec![
        json!({}),
        page(31, 0, false),
        page(1, 30, false),
        page(1, 0, true),
    ];
    for (field, value) in [
        ("amount", json!(1999)),
        ("amount", json!("9223372036854775808")),
        ("amount", json!("01")),
        ("amount", json!("1.5")),
        ("currency", json!("UNKNOWN")),
    ] {
        let mut data = valid.clone();
        data["items"][0]["price"][field] = value;
        malformed.push(data);
    }
    for data in malformed {
        let (base, task) = fixture(
            200,
            data.to_string(),
            "/api/offers?limit=30&offset=0",
            Duration::ZERO,
        );
        assert!(
            OffersClient::new(&base, Duration::from_secs(1))
                .unwrap()
                .fetch(0)
                .await
                .is_err()
        );
        task.join().unwrap();
    }
    for (status, body, delay) in [
        (500, "private diagnostic", Duration::ZERO),
        (302, "", Duration::ZERO),
        (200, "not json", Duration::ZERO),
        (200, "{}", Duration::from_millis(100)),
    ] {
        let (base, task) = fixture(status, body.into(), "/api/offers?limit=30&offset=0", delay);
        assert!(
            OffersClient::new(&base, Duration::from_millis(30))
                .unwrap()
                .fetch(0)
                .await
                .is_err()
        );
        task.join().unwrap();
    }
}

#[test]
fn configuration_defaults_and_invalid_urls() {
    assert_eq!(
        configuration(Err(VarError::NotPresent)).unwrap(),
        "http://localhost:3003"
    );
    for value in [
        "",
        " ",
        "file:///tmp/offers",
        "http://user:password@example.com",
        "http://example.com?token=private",
        "http://example.com#fragment",
        "relative",
        " http://example.com",
    ] {
        match configuration(Ok(value.into())) {
            Ok(base) => assert!(
                OffersClient::new(&base, Duration::from_secs(1)).is_err(),
                "{value}"
            ),
            Err(error) => assert_eq!(error.kind(), io::ErrorKind::InvalidInput),
        }
    }
    #[cfg(unix)]
    {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};
        assert!(configuration(Err(VarError::NotUnicode(OsString::from_vec(vec![255])))).is_err());
    }
}

#[test]
fn extended_facts_are_data_only_strict_and_compatible_with_old_responses() {
    let mut value = page(1, 0, false)["items"][0].clone();
    let legacy: Offer = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(legacy.identifiers, Default::default());
    assert_eq!(legacy.localization, Default::default());
    assert!(legacy.was_price.is_none());
    assert!(legacy.availability.is_none());
    assert!(legacy.item_condition.is_none());
    assert_eq!(legacy.content, Default::default());
    value["gtin"] = json!("0012345678905");
    value["sku"] = json!("A1-BLACK");
    value["external_identifiers"] = json!({"shopify_product_gid":"gid://shopify/Product/1","shopify_product_variation_gid":"gid://shopify/ProductVariant/2","shopify_handle":"usb-charger"});
    value["localization"] =
        json!({"target_market":{"kind":"country","code":"CA","label":null},"language":"fr"});
    let offer: Offer = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(offer.identifiers.gtin.unwrap().value, "0012345678905");
    assert_eq!(offer.localization.language.as_deref(), Some("fr"));
    for (field, invalid) in [
        ("gtin", json!(123)),
        ("sku", json!("")),
        ("external_identifiers", json!(null)),
        ("localization", json!(null)),
        ("localization", json!({"language":"bad_tag"})),
    ] {
        let mut bad = value.clone();
        bad[field] = invalid;
        assert!(serde_json::from_value::<Offer>(bad).is_err());
    }
}

#[test]
fn pricing_and_typed_facts_preserve_values_without_guessing() {
    let mut value = page(1, 0, false)["items"][0].clone();
    let raw = " https://example.com/PreOrder ž ";
    value["was_price"] = json!({"amount":"2999","currency":"AUD"});
    value["availability"] = json!({"kind":"other","raw":raw});
    value["item_condition"] = json!({"kind":"new","raw":null});
    value["content"] = json!({"description":"<p>Raw &amp; description</p>"});
    let offer: Offer = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(offer.comparable_was_price().unwrap().amount, "2999");
    assert_eq!(
        offer.availability,
        Some(retailer_sauce::availability::Availability::Other(
            raw.into()
        ))
    );
    assert_eq!(
        offer.item_condition,
        Some(retailer_sauce::item_condition::ItemCondition::New)
    );
    assert_eq!(
        offer.content.description.as_deref(),
        Some("<p>Raw &amp; description</p>")
    );
    for (amount, currency) in [("1999", "AUD"), ("1998", "AUD"), ("2999", "EUR")] {
        let mut other = value.clone();
        other["was_price"] = json!({"amount":amount,"currency":currency});
        assert!(
            serde_json::from_value::<Offer>(other)
                .unwrap()
                .comparable_was_price()
                .is_none()
        );
    }
    for kind in [
        "available",
        "unavailable",
        "in_stock",
        "out_of_stock",
        "sold_out",
    ] {
        let mut other = value.clone();
        other["availability"] = json!({"kind":kind,"raw":null});
        assert!(serde_json::from_value::<Offer>(other).is_ok());
    }
    for (field, bad) in [
        ("availability", json!("in_stock")),
        ("availability", json!(["in_stock", null])),
        ("availability", json!({"kind":"unknown","raw":null})),
        (
            "availability",
            json!({"kind":"in_stock","raw":"unexpected"}),
        ),
        ("availability", json!({"kind":"other","raw":null})),
        ("availability", json!({"kind":"other","raw":" \t"})),
        ("item_condition", json!({"kind":"other","raw":"\0"})),
        ("content", json!({})),
        ("content", json!([null])),
        ("content", json!(null)),
        ("content", json!({"description":12})),
        ("content", json!({"description":"\0"})),
    ] {
        let mut other = value.clone();
        other[field] = bad;
        assert!(serde_json::from_value::<Offer>(other).is_err(), "{field}");
    }
}

#[test]
fn search_query_boundaries() {
    assert_eq!(normalize_query(" \tČaj\u{2003}USB\n ").unwrap(), "Čaj USB");
    assert_eq!(
        normalize_query("'100%_USB+充電器'").unwrap(),
        "'100%_USB+充電器'"
    );
    assert!(normalize_query(&"é".repeat(200)).is_ok());
    for invalid in ["", " \n", "!?%_", "a\0b", "a\u{7f}", &"é".repeat(201)] {
        assert!(normalize_query(invalid).is_err(), "{invalid:?}");
    }
}

#[tokio::test]
async fn search_encodes_query_and_preserves_base_prefix() {
    let (base, task) = fixture(
        200,
        page(1, 30, false).to_string(),
        "/prefix/api/offers/search?q=%C4%8Caj+%26%2B%22&limit=30&offset=30",
        Duration::ZERO,
    );
    let client = OffersClient::new(&format!("{base}/prefix/"), Duration::from_secs(1)).unwrap();
    let result = client.search("Čaj &+\"", 30).await.unwrap();
    assert_eq!(result.items.len(), 1);
    assert!(result.items[0].images.is_empty());
    assert_eq!(result.items[0].localization, Default::default());
    assert_eq!(result.items[0].identifiers, Default::default());
    task.join().unwrap();
}

#[test]
fn new_and_historical_localization_observations_are_not_reinterpreted() {
    for localization in [
        json!({"target_market":null,"language":"it-IT"}),
        json!({"target_market":{"kind":"country","code":"IT"},"language":"it"}),
    ] {
        let mut value = page(1, 0, false)["items"][0].clone();
        value["localization"] = localization.clone();
        let offer: Offer = serde_json::from_value(value).unwrap();
        assert_eq!(
            serde_json::to_value(offer.localization).unwrap(),
            localization
        );
    }
}
