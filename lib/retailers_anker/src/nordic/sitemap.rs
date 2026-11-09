//! All five markets share the Nordic catalog. Country isolation happens in the
//! offer request and response; there are no country-prefixed product URLs.
use ::shared::{SitemapConfig, link::LinkKind};
use std::time::Duration;

pub fn sitemap_config() -> SitemapConfig {
    SitemapConfig {
        sitemap_url: vec!["https://www.ankernordics.com/sitemap.xml".into()],
    }
}
pub fn sitemap_greylisted(url: &str) -> bool {
    url.to_ascii_lowercase()
        .starts_with("https://www.ankernordics.com/")
}
pub fn sitemap_pause() -> Duration {
    Duration::from_secs(1)
}
pub fn classify_link(url: &str, source: &str, _images: usize) -> LinkKind {
    let url = url.to_ascii_lowercase();
    let Some(path) = url.strip_prefix("https://www.ankernordics.com/") else {
        return LinkKind::NotInterested;
    };
    // Reject foreign and invented country prefixes even for a known source.
    if !(path.starts_with("products/")
        || path.starts_with("collections/")
        || path.starts_with("blogs/")
        || path.starts_with("pages/"))
    {
        return LinkKind::NotInterested;
    }
    match source.to_ascii_lowercase().as_str() {
        "https://www.ankernordics.com/sitemap-0.xml"
        | "https://www.ankernordics.com/server-sitemap-index-pages.xml" => LinkKind::NotInterested,
        _ => crate::anker_from_location(&url),
    }
}
