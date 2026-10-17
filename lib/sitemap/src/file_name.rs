//! Names a sitemap file on disk after the URL it was fetched from.

use percent_encoding::percent_decode_str;
use url::Url;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum FileNameError {
    #[error("{url} isn't a valid URL: {source}")]
    InvalidUrl {
        url: String,
        #[source]
        source: url::ParseError,
    },

    #[error("{0} has no path")]
    NoPath(String),

    #[error("{0} makes a file name longer than 255 bytes")]
    TooLong(String),
}

/// The decoded URL path with `/` replaced by `-`, followed by the query with
/// `&` replaced by `_`, since Shopify pages its sitemaps by query:
/// `https://www.ankerjapan.com/sitemap_products_1.xml?from=1&to=2` becomes
/// `sitemap_products_1_from=1_to=2.xml`. The name never contains a `/`.
///
/// Two URLs can share a name, such as `/a/b` and `/a-b`, so callers that
/// write several files to one folder must check for that.
pub fn file_name(url: &str) -> Result<String, FileNameError> {
    let parsed = Url::parse(url).map_err(|source| FileNameError::InvalidUrl {
        url: url.to_string(),
        source,
    })?;
    // Decoded, since percent-encoded non-Latin paths exceed the 255-byte limit
    // on file names.
    let path = percent_decode_str(parsed.path()).decode_utf8_lossy();
    let path = path.trim_matches('/').replace('/', "-");
    if path.is_empty() {
        return Err(FileNameError::NoPath(url.to_string()));
    }

    let stem = path.strip_suffix(".xml").unwrap_or(&path);
    let name = match parsed.query().filter(|query| !query.is_empty()) {
        Some(query) => format!("{stem}_{}.xml", query.replace('&', "_").replace('/', "-")),
        None => format!("{stem}.xml"),
    };
    if name.len() > 255 {
        return Err(FileNameError::TooLong(url.to_string()));
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_a_file_after_the_url_path() {
        assert_eq!(
            file_name("https://www.anker.com/server-sitemap-index-products.xml").unwrap(),
            "server-sitemap-index-products.xml"
        );
        assert_eq!(
            file_name("https://www.anker.com/au/sitemap/products.xml").unwrap(),
            "au-sitemap-products.xml"
        );
    }

    #[test]
    fn keeps_the_query_that_pages_a_shopify_sitemap() {
        assert_eq!(
            file_name("https://www.ankerjapan.com/sitemap_products_1.xml?from=1&to=2").unwrap(),
            "sitemap_products_1_from=1_to=2.xml"
        );
    }

    #[test]
    fn gives_a_file_without_an_xml_extension_one() {
        assert_eq!(
            file_name("https://example.com/sitemap").unwrap(),
            "sitemap.xml"
        );
    }

    #[test]
    fn decodes_the_path() {
        assert_eq!(
            file_name("https://example.com/%E3%82%B5%E3%82%A4%E3%83%88.xml").unwrap(),
            "サイト.xml"
        );
    }

    #[test]
    fn never_names_a_file_with_a_slash() {
        assert_eq!(
            file_name("https://example.com/a%2Fb.xml?next=/c").unwrap(),
            "a-b_next=-c.xml"
        );
    }

    #[test]
    fn rejects_a_url_without_a_path() {
        assert_eq!(
            file_name("https://www.anker.com/"),
            Err(FileNameError::NoPath("https://www.anker.com/".to_string()))
        );
    }

    #[test]
    fn rejects_a_name_too_long_for_the_file_system() {
        let url = format!("https://example.com/{}.xml", "a".repeat(252));
        assert_eq!(file_name(&url), Err(FileNameError::TooLong(url.clone())));
    }
}
