//! Looks up each retailer in this crate by its `RetailerCode`.

use crate::micom;
use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use bodul_shared::prelude::{NormalizedData, RetailerCode};
use html_extractor::io::Architecture;
use std::time::Duration;

pub fn architecture_for(retailer: RetailerCode) -> Option<Architecture> {
    match retailer {
        RetailerCode::MiCom => Some(micom::prelude::offer_detail_architecture_v1()),
        _ => None,
    }
}

pub fn normalize_for(retailer: RetailerCode, json: &str) -> Option<Result<NormalizedData, String>> {
    match retailer {
        RetailerCode::MiCom => Some(micom::prelude::offer_detail_normalize_v1(json)),
        _ => None,
    }
}

pub fn sitemap_config_for(retailer: &RetailerCode) -> Option<SitemapConfig> {
    match retailer {
        RetailerCode::MiCom => Some(micom::prelude::sitemap_config_v1()),
        _ => None,
    }
}

pub fn sitemap_pause_for(retailer: RetailerCode) -> Option<Duration> {
    match retailer {
        RetailerCode::MiCom => Some(micom::prelude::sitemap_pause_v1()),
        _ => None,
    }
}

pub fn sitemap_greylisted_for(retailer: RetailerCode, url: &str) -> Option<bool> {
    match retailer {
        RetailerCode::MiCom => Some(micom::prelude::sitemap_greylisted_v1(url)),
        _ => None,
    }
}

pub fn classify_link_for(
    retailer: RetailerCode,
    url: &str,
    source: &str,
    image_count: usize,
) -> Option<LinkKind> {
    match retailer {
        RetailerCode::MiCom => Some(micom::prelude::classify_link_v1(url, source, image_count)),
        _ => None,
    }
}
