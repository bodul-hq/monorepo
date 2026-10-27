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
