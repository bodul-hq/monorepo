//! Looks up each retailer in this crate by its `RetailerCode`.

use std::time::Duration;

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use bodul_shared::prelude::{NormalizedData, RetailerCode};
use html_extractor::io::Architecture;

pub fn architecture_for(_retailer: RetailerCode) -> Option<Architecture> {
    None
}

pub fn normalize_for(
    _retailer: RetailerCode,
    _json: &str,
) -> Option<Result<NormalizedData, String>> {
    None
}

pub fn sitemap_config_for(_retailer: &RetailerCode) -> Option<SitemapConfig> {
    None
}

pub fn sitemap_pause_for(_retailer: RetailerCode) -> Option<Duration> {
    None
}

pub fn sitemap_greylisted_for(_retailer: RetailerCode, _url: &str) -> Option<bool> {
    None
}

pub fn classify_link_for(
    _retailer: RetailerCode,
    _url: &str,
    _source: &str,
    _image_count: usize,
) -> Option<LinkKind> {
    None
}
