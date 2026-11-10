//! Request-scoped diagnostics. Never retain headers, payloads, or raw URLs in output.
use super::FetchError;
use crate::platform::logging::TIMING_TARGET;
use std::{error::Error, time::Instant};
use uuid::Uuid;

const MAX_CAUSES: usize = 8;
const MAX_MESSAGE: usize = 8 * 1024;
const TRUNCATED: &str = " [truncated]";

/// A diagnostic URL, not a URL to send: omit userinfo, query and fragment.
pub fn safe_url(value: &str) -> String {
    let Ok(mut url) = url::Url::parse(value) else {
        return "[invalid URL]".into();
    };
    if !matches!(url.scheme(), "http" | "https" | "socks5" | "socks5h") || url.host_str().is_none()
    {
        return "[invalid URL]".into();
    }
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}

/// Also sanitize URLs embedded in a library's error text, including redirect targets.
fn sanitize_urls(text: &str) -> String {
    let mut remaining = text;
    let mut output = String::new();
    while let Some(separator) = remaining.find("://") {
        let start = remaining[..separator]
            .rfind(|c: char| !c.is_ascii_alphanumeric() && !matches!(c, '+' | '-' | '.'))
            .map_or(0, |i| i + remaining[i..].chars().next().unwrap().len_utf8());
        let end = remaining[separator + 3..]
            .find(|c: char| {
                c.is_whitespace() || matches!(c, '\'' | '"' | '`' | '<' | '>' | ')' | '}')
            })
            .map_or(remaining.len(), |i| separator + 3 + i);
        output.push_str(&remaining[..start]);
        output.push_str(&safe_url(&remaining[start..end]));
        remaining = &remaining[end..];
    }
    output.push_str(remaining);
    output
}

fn bounded(mut text: String, limit: usize) -> String {
    if text.len() > limit {
        let mut end = limit - TRUNCATED.len();
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
        text.push_str(TRUNCATED);
    }
    text
}

pub(super) struct Request {
    id: Uuid,
    method: &'static str,
    endpoint: String,
    route: String,
    proxy_endpoint: Option<String>,
    started: Instant,
    secrets: Vec<String>,
}

impl Request {
    pub(super) fn new(url: &str, method: &'static str, route: &str) -> Self {
        let mut request = Self {
            id: Uuid::now_v7(),
            method,
            endpoint: safe_url(url),
            route: route.into(),
            proxy_endpoint: None,
            started: Instant::now(),
            secrets: Vec::new(),
        };
        request.redact_url_values(url);
        request
    }

    pub(super) fn redact(&mut self, value: &str) {
        if !value.is_empty() && !self.secrets.iter().any(|old| old == value) {
            self.secrets.push(value.into());
            self.secrets
                .sort_by_key(|value| std::cmp::Reverse(value.len()));
        }
    }

    pub(super) fn redact_url_values(&mut self, value: &str) {
        if let Ok(url) = url::Url::parse(value) {
            for value in [
                url.username(),
                url.password().unwrap_or(""),
                url.fragment().unwrap_or(""),
            ] {
                self.redact(value);
                self.redact(&percent_encoding::percent_decode_str(value).decode_utf8_lossy());
            }
            for (_, value) in url.query_pairs() {
                self.redact(&value);
            }
            if let Some(query) = url.query() {
                for pair in query.split('&') {
                    if let Some((_, value)) = pair.split_once('=') {
                        self.redact(value);
                    }
                }
            }
        }
    }

    pub(super) fn route(&mut self, route: &str) {
        self.route = route.into();
    }

    pub(super) fn proxy(&mut self, url: &str) {
        self.proxy_endpoint = Some(safe_url(url));
        self.redact_url_values(url);
    }

    fn sanitize(&self, text: &str) -> String {
        let text = sanitize_urls(text);
        let mut remaining = text.as_str();
        let mut redacted = String::new();
        // Match the original text only, so short secrets cannot rewrite the
        // redaction markers inserted for longer overlapping secrets.
        while let Some((position, secret)) = self
            .secrets
            .iter()
            .filter_map(|secret| remaining.find(secret).map(|position| (position, secret)))
            .min_by_key(|(position, secret)| (*position, std::cmp::Reverse(secret.len())))
        {
            redacted.push_str(&remaining[..position]);
            redacted.push_str("[redacted]");
            remaining = &remaining[position + secret.len()..];
        }
        redacted.push_str(remaining);
        redacted
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect()
    }

    pub(super) fn failure(
        &self,
        stage: &'static str,
        error: &(dyn Error + 'static),
        status: Option<u16>,
    ) -> FetchError {
        let reqwest = error.downcast_ref::<reqwest::Error>();
        let dns = reqwest.is_some_and(reqwest::Error::is_dns);
        let connect = reqwest.is_some_and(reqwest::Error::is_connect);
        let timeout = reqwest.is_some_and(reqwest::Error::is_timeout);
        let mut causes = Vec::new();
        let mut current = Some(error);
        while let Some(error) = current {
            if causes.len() == MAX_CAUSES {
                causes.push(TRUNCATED.trim().into());
                break;
            }
            causes.push(bounded(self.sanitize(&error.to_string()), 768));
            current = error.source();
        }
        let elapsed_ms = self.started.elapsed().as_millis() as u64;
        let endpoint = bounded(self.sanitize(&self.endpoint), 1024);
        let proxy = bounded(
            self.sanitize(self.proxy_endpoint.as_deref().unwrap_or("none-or-implicit")),
            512,
        );
        let route = bounded(self.sanitize(&self.route), 128);
        let causes = causes.join(" -> ");
        let message = bounded(
            format!(
                "HTTP failure request_id={} method={} endpoint={} route={} proxy_endpoint={} stage={} elapsed_ms={} http_status={} dns={} connect={} timeout={}: {}",
                self.id,
                self.method,
                endpoint,
                route,
                proxy,
                stage,
                elapsed_ms,
                status.map_or_else(|| "none".into(), |status| status.to_string()),
                dns,
                connect,
                timeout,
                causes
            ),
            MAX_MESSAGE,
        );
        tracing::error!(request_id=%self.id, method=self.method, endpoint, route, proxy_endpoint=proxy,
            stage, elapsed_ms, http_status=?status, dns, connect, timeout, error=%message, "retailer HTTP request failed");
        tracing::info!(target: TIMING_TARGET, kind="fetch", request_id=%self.id, method=self.method,
            url=endpoint, route, proxy_endpoint=proxy, stage, elapsed_ms, http_status=?status,
            dns, connect, timeout, status="error");
        FetchError { message }
    }

    pub(super) fn success(&self, status: u16) {
        tracing::info!(target: TIMING_TARGET, kind="fetch", request_id=%self.id, method=self.method,
            url=bounded(self.sanitize(&self.endpoint), 1024), route=bounded(self.sanitize(&self.route), 128),
            proxy_endpoint=self.proxy_endpoint.as_deref().map(|value| bounded(self.sanitize(value), 512)),
            stage="complete", elapsed_ms=self.started.elapsed().as_millis() as u64,
            http_status=status, status="ok");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::layer::SubscriberExt;

    #[derive(Debug)]
    struct Cause {
        message: String,
        source: Option<Box<Cause>>,
    }
    impl std::fmt::Display for Cause {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str(&self.message)
        }
    }
    impl Error for Cause {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.source.as_deref().map(|value| value as &dyn Error)
        }
    }

    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Capture {
        type Writer = Self;
        fn make_writer(&'a self) -> Self {
            self.clone()
        }
    }
    impl Capture {
        fn text(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    struct LogStreams {
        main: Capture,
        timing: Capture,
    }
    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogStreams {
        type Writer = Capture;
        fn make_writer(&'a self) -> Capture {
            self.main.clone()
        }
        fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Capture {
            if meta.target() == TIMING_TARGET {
                self.timing.clone()
            } else {
                self.main.clone()
            }
        }
    }

    #[test]
    fn errors_and_both_log_streams_share_sanitized_diagnostics() {
        let main = Capture::default();
        let timing = Capture::default();
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .without_time()
                .with_writer(LogStreams {
                    main: main.clone(),
                    timing: timing.clone(),
                }),
        );
        // Install once, like production: other tests can initialize the same
        // HTTP tracing callsites concurrently on threads without a subscriber.
        tracing::subscriber::set_global_default(subscriber).unwrap();
        let error = {
            let mut request = Request::new(
                "https://alice:secret@example.com/graphql?key=query-secret#fragment-secret",
                "POST",
                "proxy_1",
            );
            request.proxy("http://proxy-user:proxy-password@proxy.example:8888?key=proxy-query");
            request.redact("storefront-token");
            request.failure("send", &Cause {
                message: "send failed at https://alice:secret@example.com/graphql?key=query-secret#fragment-secret".into(),
                source: Some(Box::new(Cause {
                    message: "connection failed\nproxy-password storefront-token query-secret fragment-secret proxy-query redirect https://bob:redirect-password@redirect.example/path?secret=redirect-query".into(),
                    source: None,
                })),
            }, None)
        };
        let request_id = error
            .message
            .split("request_id=")
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap();
        assert_eq!(Uuid::parse_str(request_id).unwrap().get_version_num(), 7);
        let main = main
            .text()
            .lines()
            .filter(|line| line.contains(request_id))
            .collect::<Vec<_>>()
            .join("\n");
        let timing = timing
            .text()
            .lines()
            .filter(|line| line.contains(request_id))
            .collect::<Vec<_>>()
            .join("\n");
        for output in [&error.message, &main, &timing] {
            assert!(
                output.contains(request_id),
                "missing {request_id} in {output:?}"
            );
            assert!(output.contains("proxy.example:8888"));
            assert!(output.contains("example.com/graphql"));
            assert!(output.contains("proxy_1"));
            assert!(output.contains("send"));
            for secret in [
                "alice",
                "secret",
                "proxy-user",
                "proxy-password",
                "storefront-token",
                "query-secret",
                "fragment-secret",
                "proxy-query",
                "redirect-password",
                "redirect-query",
            ] {
                assert!(!output.contains(secret), "leaked {secret}: {output}");
            }
        }
        assert!(error.message.contains(" -> connection failed"));
        assert!(!error.message.contains('\n'));
        assert!(main.contains("retailer HTTP request failed"));
        assert!(timing.contains("kind=\"fetch\""));
    }

    #[test]
    fn source_chain_and_unicode_messages_are_bounded() {
        let mut error = Cause {
            message: "deepest omitted cause".into(),
            source: None,
        };
        for i in (0..12).rev() {
            error = Cause {
                message: format!("cause-{i}: {}", "🦀".repeat(500)),
                source: Some(Box::new(error)),
            };
        }
        let request = Request::new(
            &format!("https://example.com/{}", "a".repeat(20_000)),
            "GET",
            "direct",
        );
        let message = request.failure("decode", &error, Some(200)).message;
        assert!(message.len() <= MAX_MESSAGE);
        assert!(message.contains("[truncated]"));
        assert!(message.contains("cause-7"));
        assert!(!message.contains("cause-8"));
        assert!(!message.contains("deepest"));
        assert!(message.contains("http_status=200"));
        assert!(message.contains("dns=false connect=false timeout=false"));
    }

    #[test]
    fn overlapping_secrets_do_not_rewrite_redaction_markers() {
        let mut request = Request::new("https://example.com", "GET", "direct");
        request.redact("abc");
        request.redact("a");
        request.redact("redacted");
        assert_eq!(
            request.sanitize("abc a redacted"),
            "[redacted] [redacted] [redacted]"
        );
    }

    #[test]
    fn sanitizes_embedded_urls_and_encoded_credentials() {
        let request = Request::new(
            "https://user:pass%20word@example.com/?key=encoded%2Fsecret#private",
            "GET",
            "direct",
        );
        let value = request.sanitize("éhttps://other:password@redirect.example:8443/a?token=anything#fragment pass word encoded/secret encoded%2Fsecret");
        assert_eq!(
            value,
            "éhttps://redirect.example:8443/a [redacted] [redacted] [redacted]"
        );
        assert_eq!(safe_url("not a url"), "[invalid URL]");
        assert_eq!(safe_url("data:text/plain,hidden"), "[invalid URL]");
        assert_eq!(
            safe_url("https://user:password@[::1]:123/path?secret=yes#hidden"),
            "https://[::1]:123/path"
        );
    }
}
