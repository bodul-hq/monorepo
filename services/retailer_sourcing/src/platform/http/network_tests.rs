//! Local HTTP fixtures; no public network or real retailer credentials required.
use super::*;
use std::{io::Write, net::TcpListener, time::Duration};

fn server(response: &'static [u8], delay: Duration) -> (String, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/graphql?key=query-secret",
        listener.local_addr().unwrap()
    );
    let thread = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut request = Vec::new();
        let mut byte = [0];
        while !request.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        let headers = String::from_utf8(request).unwrap();
        let length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().unwrap())
            })
            .unwrap_or(0);
        let mut body = vec![0; length];
        stream.read_exact(&mut body).unwrap();
        std::thread::sleep(delay);
        let _ = stream.write_all(response);
        headers
    });
    (url, thread)
}

#[test]
fn status_body_and_decode_failures_keep_their_stage_and_status() {
    for (response, stage, status) in [
        (b"HTTP/1.1 503 Unavailable\r\nContent-Length: 15\r\nConnection: close\r\n\r\nresponse-secret".as_slice(), "status", 503),
        (b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\nConnection: close\r\n\r\nshort".as_slice(), "read_body", 200),
        (b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\n\x1f\x8b\x00".as_slice(), "decode", 200),
    ] {
        let (url, server) = server(response, Duration::ZERO);
        let error = Client.get_with_builder(&url, reqwest::blocking::Client::builder().no_proxy(), "test").unwrap_err();
        server.join().unwrap();
        assert!(error.message.contains(&format!("stage={stage}")), "{error}");
        assert!(error.message.contains(&format!("http_status={status}")), "{error}");
        assert!(!error.message.contains("query-secret"));
        assert!(!error.message.contains("response-secret"));
    }
}

#[test]
fn timeout_is_classified_and_preserves_the_source_chain() {
    let (url, server) = server(b"", Duration::from_millis(300));
    let error = Client
        .get_with_builder(
            &url,
            reqwest::blocking::Client::builder()
                .no_proxy()
                .timeout(Duration::from_millis(100)),
            "test",
        )
        .unwrap_err();
    server.join().unwrap();
    assert!(error.message.contains("stage=send"), "{error}");
    assert!(error.message.contains("timeout=true"), "{error}");
    assert!(error.message.contains(" -> "), "{error}");
}

#[test]
fn failed_proxy_post_identifies_route_without_leaking_token_or_body() {
    let (proxy, server) = server(
        b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        Duration::ZERO,
    );
    let mut proxy = url::Url::parse(&proxy).unwrap();
    proxy.set_username("proxy-user").unwrap();
    proxy.set_password(Some("proxy-password")).unwrap();
    proxy.set_query(None);
    let client = OfferClient::from_urls(&[proxy.as_str()]).unwrap();
    assert_eq!(client.next_route(), 0); // Select the proxy on the tested call.
    let error = client
        .post_storefront_json(
            "http://storefront.invalid/graphql",
            r#"{"secret":"request-body-secret"}"#,
            "storefront-token",
        )
        .unwrap_err();
    server.join().unwrap();
    assert!(error.message.contains("route=proxy_1"), "{error}");
    assert!(
        error.message.contains("proxy_endpoint=http://127.0.0.1:"),
        "{error}"
    );
    assert!(error.message.contains("method=POST"));
    assert!(error.message.contains("http_status=502"));
    for secret in [
        "proxy-user",
        "proxy-password",
        "storefront-token",
        "request-body-secret",
    ] {
        assert!(!error.message.contains(secret), "{error}");
    }
}

#[test]
fn refused_connection_has_connect_flag_and_cause() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let error = Client
        .get_with_builder(
            &url,
            reqwest::blocking::Client::builder().no_proxy(),
            "direct",
        )
        .unwrap_err();
    assert!(error.message.contains("connect=true"), "{error}");
    assert!(error.message.contains("stage=send"), "{error}");
    assert!(error.message.contains(" -> "), "{error}");
}

#[test]
fn public_storefront_post_has_no_authentication_header() {
    let (url, server) = server(
        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
        Duration::ZERO,
    );
    assert_eq!(
        OfferClient::default()
            .post_storefront_json_with_access(&url, "{}", None)
            .unwrap(),
        "{}"
    );
    let headers = server.join().unwrap().to_ascii_lowercase();
    assert!(headers.starts_with("post "));
    assert!(headers.contains("content-type: application/json"));
    assert!(!headers.contains("access-token"));
    assert!(!headers.contains("authorization:"));
}
