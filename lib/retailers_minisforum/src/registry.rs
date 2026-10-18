//! Looks up each retailer in this crate by its `RetailerCode`.

use crate::minisforumau;
use crate::minisforumca;
use crate::minisforumeu;
use crate::minisforumfr;
use crate::minisforumhk;
use crate::minisforumjp;
use crate::minisforumkr;
use crate::minisforumru;
use crate::minisforumuk;
use crate::minisforumus;
use std::time::Duration;

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use bodul_shared::prelude::{NormalizedData, RetailerCode};
use html_extractor::io::Architecture;
use retailer_sauce::structuralized::StructuralizedData;

pub fn architecture_for(retailer: RetailerCode) -> Option<Architecture> {
    match retailer {
        RetailerCode::MinisForumAu => Some(minisforumau::prelude::offer_detail_architecture_v1()),
        RetailerCode::MinisForumCa => Some(minisforumca::prelude::offer_detail_architecture_v1()),
        RetailerCode::MinisForumEu => Some(minisforumeu::prelude::offer_detail_architecture_v1()),
        RetailerCode::MinisForumFr => Some(minisforumfr::prelude::offer_detail_architecture_v1()),
        RetailerCode::MinisForumHk => Some(minisforumhk::prelude::offer_detail_architecture_v1()),
        RetailerCode::MinisForumJp => Some(minisforumjp::prelude::offer_detail_architecture_v1()),
        RetailerCode::MinisForumKr => Some(minisforumkr::prelude::offer_detail_architecture_v1()),
        RetailerCode::MinisForumRu => Some(minisforumru::prelude::offer_detail_architecture_v1()),
        RetailerCode::MinisForumUk => Some(minisforumuk::prelude::offer_detail_architecture_v1()),
        RetailerCode::MinisForumUs => Some(minisforumus::prelude::offer_detail_architecture_v1()),
        _ => None,
    }
}

pub fn normalize_for(retailer: RetailerCode, json: &str) -> Option<Result<NormalizedData, String>> {
    match retailer {
        RetailerCode::MinisForumAu => Some(minisforumau::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumCa => Some(minisforumca::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumEu => Some(minisforumeu::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumFr => Some(minisforumfr::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumHk => Some(minisforumhk::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumJp => Some(minisforumjp::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumKr => Some(minisforumkr::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumRu => Some(minisforumru::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumUk => Some(minisforumuk::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumUs => Some(minisforumus::prelude::offer_detail_normalize_v1(json)),
        _ => None,
    }
}

/// Only the storefronts with a structuralizer have an arm; every other code is `None`.
pub fn structuralize_for(
    retailer: RetailerCode,
    json: &str,
) -> Option<Result<StructuralizedData, String>> {
    match retailer {
        RetailerCode::MinisForumAu => {
            Some(minisforumau::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumEu => {
            Some(minisforumeu::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumFr => {
            Some(minisforumfr::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumHk => {
            Some(minisforumhk::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumJp => {
            Some(minisforumjp::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumCa => {
            Some(minisforumca::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumKr => {
            Some(minisforumkr::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumRu => {
            Some(minisforumru::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumUk => {
            Some(minisforumuk::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumUs => {
            Some(minisforumus::prelude::offer_detail_structuralize_v1(json))
        }
        _ => None,
    }
}

pub fn sitemap_config_for(retailer: &RetailerCode) -> Option<SitemapConfig> {
    match retailer {
        RetailerCode::MinisForumAu => Some(minisforumau::prelude::sitemap_config_v1()),
        RetailerCode::MinisForumCa => Some(minisforumca::prelude::sitemap_config_v1()),
        RetailerCode::MinisForumEu => Some(minisforumeu::prelude::sitemap_config_v1()),
        RetailerCode::MinisForumFr => Some(minisforumfr::prelude::sitemap_config_v1()),
        RetailerCode::MinisForumHk => Some(minisforumhk::prelude::sitemap_config_v1()),
        RetailerCode::MinisForumJp => Some(minisforumjp::prelude::sitemap_config_v1()),
        RetailerCode::MinisForumKr => Some(minisforumkr::prelude::sitemap_config_v1()),
        RetailerCode::MinisForumRu => Some(minisforumru::prelude::sitemap_config_v1()),
        RetailerCode::MinisForumUk => Some(minisforumuk::prelude::sitemap_config_v1()),
        RetailerCode::MinisForumUs => Some(minisforumus::prelude::sitemap_config_v1()),
        _ => None,
    }
}

pub fn sitemap_pause_for(retailer: RetailerCode) -> Option<Duration> {
    match retailer {
        RetailerCode::MinisForumAu => Some(minisforumau::prelude::sitemap_pause_v1()),
        RetailerCode::MinisForumCa => Some(minisforumca::prelude::sitemap_pause_v1()),
        RetailerCode::MinisForumEu => Some(minisforumeu::prelude::sitemap_pause_v1()),
        RetailerCode::MinisForumFr => Some(minisforumfr::prelude::sitemap_pause_v1()),
        RetailerCode::MinisForumHk => Some(minisforumhk::prelude::sitemap_pause_v1()),
        RetailerCode::MinisForumJp => Some(minisforumjp::prelude::sitemap_pause_v1()),
        RetailerCode::MinisForumKr => Some(minisforumkr::prelude::sitemap_pause_v1()),
        RetailerCode::MinisForumRu => Some(minisforumru::prelude::sitemap_pause_v1()),
        RetailerCode::MinisForumUk => Some(minisforumuk::prelude::sitemap_pause_v1()),
        RetailerCode::MinisForumUs => Some(minisforumus::prelude::sitemap_pause_v1()),
        _ => None,
    }
}

pub fn sitemap_greylisted_for(retailer: RetailerCode, url: &str) -> Option<bool> {
    match retailer {
        RetailerCode::MinisForumAu => Some(minisforumau::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumCa => Some(minisforumca::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumEu => Some(minisforumeu::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumFr => Some(minisforumfr::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumHk => Some(minisforumhk::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumJp => Some(minisforumjp::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumKr => Some(minisforumkr::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumRu => Some(minisforumru::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumUk => Some(minisforumuk::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumUs => Some(minisforumus::prelude::sitemap_greylisted_v1(url)),
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
        RetailerCode::MinisForumAu => Some(minisforumau::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumCa => Some(minisforumca::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumEu => Some(minisforumeu::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumFr => Some(minisforumfr::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumHk => Some(minisforumhk::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumJp => Some(minisforumjp::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumKr => Some(minisforumkr::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumRu => Some(minisforumru::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumUk => Some(minisforumuk::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumUs => Some(minisforumus::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        _ => None,
    }
}
