//! Blocking HTTP client for fetching remote documents.

use std::fmt;
use std::io::Read;

use flate2::read::GzDecoder;

use super::logging::TIMING_TARGET;

/// A browser-like User-Agent. Many storefronts reject the default `reqwest/x.y`
/// agent with `403 Forbidden`; sending a common one is enough for basic fetching.
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

/// The first two bytes of every gzip stream (RFC 1952).
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// An error fetching a remote resource.
#[derive(Debug, Clone)]
pub struct FetchError {
    pub message: String,
}

impl fmt::Display for FetchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for FetchError {}

/// A minimal blocking HTTP client.
///
/// Each call builds its own `reqwest` client. A blocking client can't be built
/// on an async runtime thread, and calls run on Mulac's blocking workers.
#[derive(Debug, Clone, Default)]
pub struct Client;

impl Client {
    /// Fetches `url` and returns the response body as text.
    ///
    /// Bodies that are gzip-compressed at the content level (e.g. `*.xml.gz`)
    /// are decompressed here, so callers always receive plain text. Detection is
    /// by gzip magic bytes, so it works regardless of the URL suffix or a
    /// mislabelled `Content-Type`.
    pub fn get(&self, url: &str) -> Result<String, FetchError> {
        self.get_with_builder(url, reqwest::blocking::Client::builder(), "default")
    }

    fn get_with_builder(
        &self,
        url: &str,
        builder: reqwest::blocking::ClientBuilder,
        route: &str,
    ) -> Result<String, FetchError> {
        self.request_with_builder(url, None, builder, route)
    }

    fn request_with_builder(
        &self,
        url: &str,
        json_body: Option<&str>,
        builder: reqwest::blocking::ClientBuilder,
        route: &str,
    ) -> Result<String, FetchError> {
        let builder = if json_body.is_some() {
            builder.timeout(std::time::Duration::from_secs(60))
        } else {
            builder
        };
        let client = builder
            .user_agent(USER_AGENT)
            .build()
            .map_err(|error| FetchError {
                message: error.to_string(),
            })?;

        // Time just the network fetch (request -> response bytes), excluding decoding.
        let start = std::time::Instant::now();
        let request = match json_body {
            Some(body) => client
                .post(url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.to_owned()),
            None => client.get(url),
        };
        let fetched = request
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.bytes());
        let elapsed_ms = start.elapsed().as_millis() as u64;
        let status = if fetched.is_ok() { "ok" } else { "error" };
        tracing::info!(target: TIMING_TARGET, kind = "fetch", route, url, elapsed_ms, status);

        let bytes = fetched.map_err(|error| FetchError {
            message: error.to_string(),
        })?;

        decode_body(&bytes)
    }
}

/// Decodes a response body to text, gunzipping it first when it is a gzip stream.
fn decode_body(bytes: &[u8]) -> Result<String, FetchError> {
    if bytes.starts_with(&GZIP_MAGIC) {
        let mut text = String::new();
        GzDecoder::new(bytes)
            .read_to_string(&mut text)
            .map_err(|error| FetchError {
                message: format!("gzip decode failed: {error}"),
            })?;
        Ok(text)
    } else {
        // Lossy decoding keeps a stray byte from failing the whole fetch.
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    #[test]
    fn source_post_sends_json_and_preserves_the_response_body() {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/graphql", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                stream.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            let headers = String::from_utf8(request).unwrap().to_ascii_lowercase();
            assert!(headers.starts_with("post /graphql http/1.1\r\n"));
            assert!(headers.contains("content-type: application/json"));
            let mut body = [0; 2];
            stream.read_exact(&mut body).unwrap();
            assert_eq!(&body, b"{}");
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\n{\"data\":1}").unwrap();
        });
        let body = Client
            .request_with_builder(
                &url,
                Some("{}"),
                reqwest::blocking::Client::builder().no_proxy(),
                "test",
            )
            .unwrap();
        server.join().unwrap();
        assert_eq!(body, "{\"data\":1}");
    }

    fn gzip(text: &str) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(text.as_bytes()).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn decodes_gzipped_body() {
        let xml = "<urlset><url><loc>https://example.com/a</loc></url></urlset>";
        assert_eq!(decode_body(&gzip(xml)).unwrap(), xml);
    }

    #[test]
    fn passes_plain_body_through() {
        let xml = "<urlset></urlset>";
        assert_eq!(decode_body(xml.as_bytes()).unwrap(), xml);
    }
}

/// Process-wide offer routing, shared by cloned handlers.
#[derive(Clone, Default)]
pub struct OfferClient {
    proxies: std::sync::Arc<Vec<reqwest::Proxy>>,
    cursor: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    nordic_storefront_token: Option<reqwest::header::HeaderValue>,
}

const OFFER_PROXIES: &[&str] = &["http://10.1.0.102:8888"];

impl OfferClient {
    pub fn configured() -> Result<Self, std::io::Error> {
        let mut client = Self::from_urls(OFFER_PROXIES)?;
        if let Some(value) = std::env::var_os("ANKER_NORDICS_STOREFRONT_TOKEN") {
            let invalid = || {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid ANKER_NORDICS_STOREFRONT_TOKEN",
                )
            };
            let value = value
                .to_str()
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(invalid)?;
            let mut token = reqwest::header::HeaderValue::from_str(value).map_err(|_| invalid())?;
            token.set_sensitive(true);
            client.nordic_storefront_token = Some(token);
        }
        Ok(client)
    }

    fn from_urls(urls: &[&str]) -> Result<Self, std::io::Error> {
        let invalid = || {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid hardcoded offer proxy URL",
            )
        };
        let mut proxies = Vec::new();
        {
            for entry in urls {
                let url = url::Url::parse(entry).map_err(|_| invalid())?;
                if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
                    return Err(invalid());
                }
                proxies.push(reqwest::Proxy::all(url.as_str()).map_err(|_| invalid())?);
            }
        }
        Ok(Self {
            proxies: std::sync::Arc::new(proxies),
            ..Self::default()
        })
    }

    fn next_route(&self) -> usize {
        self.cursor
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |current| Some((current + 1) % (self.proxies.len() + 1)),
            )
            .unwrap()
    }

    pub fn get(&self, url: &str) -> Result<String, FetchError> {
        self.request(url, None)
    }

    pub fn post_json(&self, url: &str, body: &str) -> Result<String, FetchError> {
        self.request(url, Some(body))
    }

    fn request(&self, url: &str, json_body: Option<&str>) -> Result<String, FetchError> {
        let index = self.next_route();
        let mut builder = reqwest::blocking::Client::builder().no_proxy();
        if json_body.is_some() {
            if url != "https://ankerneu.myshopify.com/api/2026-10/graphql.json" {
                return Err(FetchError {
                    message: "unsupported offer POST endpoint".into(),
                });
            }
            let token = self
                .nordic_storefront_token
                .clone()
                .ok_or_else(|| FetchError {
                    message:
                        "ANKER_NORDICS_STOREFRONT_TOKEN is required for Nordic market downloads"
                            .into(),
                })?;
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("X-Shopify-Storefront-Access-Token", token);
            builder = builder
                .default_headers(headers)
                .redirect(reqwest::redirect::Policy::none());
        }
        let route = if index == 0 {
            "direct".to_owned()
        } else {
            builder = builder.proxy(self.proxies[index - 1].clone());
            format!("proxy_{index}")
        };
        Client.request_with_builder(url, json_body, builder, &route)
    }
}

#[cfg(test)]
mod routing_tests {
    use super::*;
    #[test]
    fn nordic_posts_require_a_token_and_cannot_send_it_to_another_endpoint() {
        let client = OfferClient::default();
        assert!(
            client
                .post_json(
                    "https://ankerneu.myshopify.com/api/2026-10/graphql.json",
                    "{}"
                )
                .unwrap_err()
                .message
                .contains("ANKER_NORDICS_STOREFRONT_TOKEN")
        );
        let client = OfferClient {
            nordic_storefront_token: Some(reqwest::header::HeaderValue::from_static(
                "test-public-token",
            )),
            ..Default::default()
        };
        assert_eq!(
            client
                .post_json("https://other.example/graphql", "{}")
                .unwrap_err()
                .message,
            "unsupported offer POST endpoint"
        );
    }
    #[test]
    fn rotation_is_shared_and_supports_multiple_proxies() {
        for count in 0..=2 {
            let value = vec!["http://127.0.0.1:8888"; count];
            let client = OfferClient::from_urls(&value).unwrap();
            let clone = client.clone();
            for i in 0..12 {
                assert_eq!(
                    if i % 2 == 0 {
                        client.next_route()
                    } else {
                        clone.next_route()
                    },
                    i % (count + 1)
                );
            }
        }
    }
    #[test]
    fn configuration_validation() {
        assert!(OfferClient::from_urls(&[]).is_ok());
        for value in ["bad", "socks5://localhost:8888", ""] {
            assert!(OfferClient::from_urls(&[value]).is_err());
        }
    }
}
