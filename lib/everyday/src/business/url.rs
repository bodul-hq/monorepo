//! URL representation.
//!
//! Provides a value object for web resource URLs with basic validation.

use core::fmt;
use serde::{Deserialize, Serialize};

/// Errors that can occur when parsing a URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlError {
    /// `://` separator not present.
    MissingSchemeSeparator,
    /// Scheme is empty or contains non-letter characters.
    InvalidScheme,
    /// Scheme is not in the allowed set (http, https).
    DisallowedScheme,
    /// Nothing after `://` or empty host segment.
    MissingHost,
    /// Host contains whitespace, contains `/`, starts or ends with `.`.
    InvalidHost,
    /// URL contains whitespace characters.
    WhitespaceInUrl,
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlError::MissingSchemeSeparator => write!(f, "URL must contain '://' separator"),
            UrlError::InvalidScheme => {
                write!(f, "URL scheme must contain only ASCII letters")
            }
            UrlError::DisallowedScheme => {
                write!(f, "URL scheme must be 'http' or 'https'")
            }
            UrlError::MissingHost => write!(f, "URL must have a non-empty host"),
            UrlError::InvalidHost => write!(f, "URL host is invalid"),
            UrlError::WhitespaceInUrl => write!(f, "URL must not contain whitespace"),
        }
    }
}

impl std::error::Error for UrlError {}

/// Url represents a web resource URL.
///
/// The URL is stored in normalized form with lowercase scheme and host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Url {
    scheme: String,
    host: String,
    rest: String,
}

impl Url {
    /// Creates a new Url from a raw string.
    ///
    /// The input is trimmed. Scheme and host are converted to lowercase.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input does not contain `://` separator
    /// - Scheme is empty, contains non-letter characters, or is not allowed
    /// - Host is empty, contains whitespace or `/`, or starts/ends with `.`
    /// - URL contains whitespace
    pub fn new(url: impl Into<String>) -> Result<Self, UrlError> {
        let url = url.into();
        let trimmed = url.trim();

        let Some((scheme, remainder)) = trimmed.split_once("://") else {
            return Err(UrlError::MissingSchemeSeparator);
        };

        if scheme.is_empty() {
            return Err(UrlError::InvalidScheme);
        }
        if !scheme.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(UrlError::InvalidScheme);
        }

        let scheme_lower = scheme.to_lowercase();
        if scheme_lower != "http" && scheme_lower != "https" {
            return Err(UrlError::DisallowedScheme);
        }

        let (host, rest) = extract_host_and_rest(remainder);

        if host.is_empty() {
            return Err(UrlError::MissingHost);
        }
        if host.chars().any(|c| c.is_whitespace()) {
            return Err(UrlError::InvalidHost);
        }
        if host.contains('/') {
            return Err(UrlError::InvalidHost);
        }
        if host.starts_with('.') || host.ends_with('.') {
            return Err(UrlError::InvalidHost);
        }

        if rest.chars().any(|c| c.is_whitespace()) {
            return Err(UrlError::WhitespaceInUrl);
        }

        Ok(Url {
            scheme: scheme_lower,
            host: host.to_lowercase(),
            rest: rest.to_string(),
        })
    }

    /// Returns the full URL as a string.
    pub fn value(&self) -> String {
        if self.rest.is_empty() {
            format!("{}://{}", self.scheme, self.host)
        } else {
            format!("{}://{}{}", self.scheme, self.host, self.rest)
        }
    }

    /// Returns the scheme (protocol) part of the URL.
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// Returns the host part of the URL.
    pub fn host(&self) -> &str {
        &self.host
    }
}

/// Extracts host and rest from the remainder after `://`.
///
/// Host is the substring up to the first `/`, `?`, or `#`.
fn extract_host_and_rest(remainder: &str) -> (&str, &str) {
    if let Some(slash_pos) = remainder.find('/') {
        (&remainder[..slash_pos], &remainder[slash_pos..])
    } else if let Some(query_pos) = remainder.find('?') {
        (&remainder[..query_pos], &remainder[query_pos..])
    } else if let Some(frag_pos) = remainder.find('#') {
        (&remainder[..frag_pos], &remainder[frag_pos..])
    } else {
        (remainder, "")
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid URLs

    #[test]
    fn valid_simple_https() {
        let url = Url::new("https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.value(), "https://example.com");
    }

    #[test]
    fn valid_simple_http() {
        let url = Url::new("http://example.com").unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host(), "example.com");
    }

    #[test]
    fn valid_with_trimming_and_lowercase() {
        let url = Url::new(" HTTP://EXAMPLE.com/path?query=1#section ").unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.value(), "http://example.com/path?query=1#section");
    }

    #[test]
    fn valid_with_path() {
        let url = Url::new("https://api.example.com/v1/resource").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "api.example.com");
        assert_eq!(url.value(), "https://api.example.com/v1/resource");
    }

    #[test]
    fn valid_with_query() {
        let url = Url::new("https://example.com?foo=bar").unwrap();
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.value(), "https://example.com?foo=bar");
    }

    #[test]
    fn valid_with_fragment() {
        let url = Url::new("https://example.com#section").unwrap();
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.value(), "https://example.com#section");
    }

    #[test]
    fn valid_with_port() {
        let url = Url::new("https://example.com:8080/path").unwrap();
        assert_eq!(url.host(), "example.com:8080");
        assert_eq!(url.value(), "https://example.com:8080/path");
    }

    #[test]
    fn valid_uppercase_scheme_normalized() {
        let url = Url::new("HTTPS://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn valid_uppercase_host_normalized() {
        let url = Url::new("https://EXAMPLE.COM").unwrap();
        assert_eq!(url.host(), "example.com");
    }

    #[test]
    fn valid_subdomain() {
        let url = Url::new("https://www.example.com").unwrap();
        assert_eq!(url.host(), "www.example.com");
    }

    // Invalid URLs

    #[test]
    fn invalid_missing_scheme_separator() {
        let result = Url::new("example.com");
        assert_eq!(result.unwrap_err(), UrlError::MissingSchemeSeparator);
    }

    #[test]
    fn invalid_scheme_with_colon_only() {
        let result = Url::new("http:example.com");
        assert_eq!(result.unwrap_err(), UrlError::MissingSchemeSeparator);
    }

    #[test]
    fn invalid_empty_scheme() {
        let result = Url::new("://example.com");
        assert_eq!(result.unwrap_err(), UrlError::InvalidScheme);
    }

    #[test]
    fn invalid_scheme_with_numbers() {
        let result = Url::new("http2://example.com");
        assert_eq!(result.unwrap_err(), UrlError::InvalidScheme);
    }

    #[test]
    fn invalid_disallowed_scheme() {
        let result = Url::new("ftp://example.com");
        assert_eq!(result.unwrap_err(), UrlError::DisallowedScheme);
    }

    #[test]
    fn invalid_missing_host() {
        let result = Url::new("http://");
        assert_eq!(result.unwrap_err(), UrlError::MissingHost);
    }

    #[test]
    fn invalid_empty_host_with_path() {
        let result = Url::new("http:///path");
        assert_eq!(result.unwrap_err(), UrlError::MissingHost);
    }

    #[test]
    fn invalid_whitespace_in_host() {
        let result = Url::new("http:// example.com");
        assert_eq!(result.unwrap_err(), UrlError::InvalidHost);
    }

    #[test]
    fn invalid_host_starts_with_dot() {
        let result = Url::new("http://.example.com");
        assert_eq!(result.unwrap_err(), UrlError::InvalidHost);
    }

    #[test]
    fn invalid_host_ends_with_dot() {
        let result = Url::new("http://example.com.");
        assert_eq!(result.unwrap_err(), UrlError::InvalidHost);
    }

    #[test]
    fn invalid_whitespace_in_path() {
        let result = Url::new("http://example.com/path with spaces");
        assert_eq!(result.unwrap_err(), UrlError::WhitespaceInUrl);
    }

    // Equality

    #[test]
    fn equality_case_insensitive_scheme_and_host() {
        let url1 = Url::new("HTTP://EXAMPLE.com").unwrap();
        let url2 = Url::new("http://example.com").unwrap();
        assert_eq!(url1, url2);
    }

    #[test]
    fn equality_different_hosts() {
        let url1 = Url::new("https://example.com").unwrap();
        let url2 = Url::new("https://example.org").unwrap();
        assert_ne!(url1, url2);
    }

    #[test]
    fn equality_different_paths() {
        let url1 = Url::new("https://example.com/path1").unwrap();
        let url2 = Url::new("https://example.com/path2").unwrap();
        assert_ne!(url1, url2);
    }

    // Display

    #[test]
    fn display_formats_correctly() {
        let url = Url::new("HTTPS://EXAMPLE.COM/Path").unwrap();
        assert_eq!(format!("{}", url), "https://example.com/Path");
    }

    // Error display

    #[test]
    fn error_display_messages() {
        assert!(UrlError::MissingSchemeSeparator.to_string().contains("://"));
        assert!(UrlError::InvalidScheme.to_string().contains("scheme"));
        assert!(UrlError::DisallowedScheme.to_string().contains("http"));
        assert!(UrlError::MissingHost.to_string().contains("host"));
        assert!(UrlError::InvalidHost.to_string().contains("host"));
        assert!(UrlError::WhitespaceInUrl.to_string().contains("whitespace"));
    }

    // Host-only URLs (no path)

    #[test]
    fn valid_host_without_dot() {
        // Per spec: "MAY also be a valid IP address" or simpler hosts
        // localhost is a common case
        let url = Url::new("http://localhost");
        assert!(url.is_ok());
        assert_eq!(url.unwrap().host(), "localhost");
    }
}
