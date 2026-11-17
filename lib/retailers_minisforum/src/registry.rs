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
use minisforumau::prelude::classify_link_v1 as prelude_classify_link_v1;
use minisforumau::prelude::offer_detail_architecture_v1 as prelude_offer_detail_architecture_v1;
use minisforumau::prelude::offer_detail_normalize_v1 as prelude_offer_detail_normalize_v1;
use minisforumau::prelude::offer_detail_structuralize_v1 as prelude_offer_detail_structuralize_v1;
use minisforumau::prelude::sitemap_config_v1 as prelude_sitemap_config_v1;
use minisforumau::prelude::sitemap_greylisted_v1 as prelude_sitemap_greylisted_v1;
use minisforumau::prelude::sitemap_pause_v1 as prelude_sitemap_pause_v1;
use minisforumca::prelude::classify_link_v1 as minisforumca_prelude_classify_link_v1;
use minisforumca::prelude::offer_detail_architecture_v1 as minisforumca_prelude_offer_detail_architecture_v1;
use minisforumca::prelude::offer_detail_normalize_v1 as minisforumca_prelude_offer_detail_normalize_v1;
use minisforumca::prelude::offer_detail_structuralize_v1 as minisforumca_prelude_offer_detail_structuralize_v1;
use minisforumca::prelude::sitemap_config_v1 as minisforumca_prelude_sitemap_config_v1;
use minisforumca::prelude::sitemap_greylisted_v1 as minisforumca_prelude_sitemap_greylisted_v1;
use minisforumca::prelude::sitemap_pause_v1 as minisforumca_prelude_sitemap_pause_v1;
use minisforumeu::prelude::classify_link_v1 as minisforumeu_prelude_classify_link_v1;
use minisforumeu::prelude::offer_detail_architecture_v1 as minisforumeu_prelude_offer_detail_architecture_v1;
use minisforumeu::prelude::offer_detail_normalize_v1 as minisforumeu_prelude_offer_detail_normalize_v1;
use minisforumeu::prelude::offer_detail_structuralize_v1 as minisforumeu_prelude_offer_detail_structuralize_v1;
use minisforumeu::prelude::sitemap_config_v1 as minisforumeu_prelude_sitemap_config_v1;
use minisforumeu::prelude::sitemap_greylisted_v1 as minisforumeu_prelude_sitemap_greylisted_v1;
use minisforumeu::prelude::sitemap_pause_v1 as minisforumeu_prelude_sitemap_pause_v1;
use minisforumfr::prelude::classify_link_v1 as minisforumfr_prelude_classify_link_v1;
use minisforumfr::prelude::offer_detail_architecture_v1 as minisforumfr_prelude_offer_detail_architecture_v1;
use minisforumfr::prelude::offer_detail_normalize_v1 as minisforumfr_prelude_offer_detail_normalize_v1;
use minisforumfr::prelude::offer_detail_structuralize_v1 as minisforumfr_prelude_offer_detail_structuralize_v1;
use minisforumfr::prelude::sitemap_config_v1 as minisforumfr_prelude_sitemap_config_v1;
use minisforumfr::prelude::sitemap_greylisted_v1 as minisforumfr_prelude_sitemap_greylisted_v1;
use minisforumfr::prelude::sitemap_pause_v1 as minisforumfr_prelude_sitemap_pause_v1;
use minisforumhk::prelude::classify_link_v1 as minisforumhk_prelude_classify_link_v1;
use minisforumhk::prelude::offer_detail_architecture_v1 as minisforumhk_prelude_offer_detail_architecture_v1;
use minisforumhk::prelude::offer_detail_normalize_v1 as minisforumhk_prelude_offer_detail_normalize_v1;
use minisforumhk::prelude::offer_detail_structuralize_v1 as minisforumhk_prelude_offer_detail_structuralize_v1;
use minisforumhk::prelude::sitemap_config_v1 as minisforumhk_prelude_sitemap_config_v1;
use minisforumhk::prelude::sitemap_greylisted_v1 as minisforumhk_prelude_sitemap_greylisted_v1;
use minisforumhk::prelude::sitemap_pause_v1 as minisforumhk_prelude_sitemap_pause_v1;
use minisforumjp::prelude::classify_link_v1 as minisforumjp_prelude_classify_link_v1;
use minisforumjp::prelude::offer_detail_architecture_v1 as minisforumjp_prelude_offer_detail_architecture_v1;
use minisforumjp::prelude::offer_detail_normalize_v1 as minisforumjp_prelude_offer_detail_normalize_v1;
use minisforumjp::prelude::offer_detail_structuralize_v1 as minisforumjp_prelude_offer_detail_structuralize_v1;
use minisforumjp::prelude::sitemap_config_v1 as minisforumjp_prelude_sitemap_config_v1;
use minisforumjp::prelude::sitemap_greylisted_v1 as minisforumjp_prelude_sitemap_greylisted_v1;
use minisforumjp::prelude::sitemap_pause_v1 as minisforumjp_prelude_sitemap_pause_v1;
use minisforumkr::prelude::classify_link_v1 as minisforumkr_prelude_classify_link_v1;
use minisforumkr::prelude::offer_detail_architecture_v1 as minisforumkr_prelude_offer_detail_architecture_v1;
use minisforumkr::prelude::offer_detail_normalize_v1 as minisforumkr_prelude_offer_detail_normalize_v1;
use minisforumkr::prelude::offer_detail_structuralize_v1 as minisforumkr_prelude_offer_detail_structuralize_v1;
use minisforumkr::prelude::sitemap_config_v1 as minisforumkr_prelude_sitemap_config_v1;
use minisforumkr::prelude::sitemap_greylisted_v1 as minisforumkr_prelude_sitemap_greylisted_v1;
use minisforumkr::prelude::sitemap_pause_v1 as minisforumkr_prelude_sitemap_pause_v1;
use minisforumru::prelude::classify_link_v1 as minisforumru_prelude_classify_link_v1;
use minisforumru::prelude::offer_detail_architecture_v1 as minisforumru_prelude_offer_detail_architecture_v1;
use minisforumru::prelude::offer_detail_normalize_v1 as minisforumru_prelude_offer_detail_normalize_v1;
use minisforumru::prelude::offer_detail_structuralize_v1 as minisforumru_prelude_offer_detail_structuralize_v1;
use minisforumru::prelude::sitemap_config_v1 as minisforumru_prelude_sitemap_config_v1;
use minisforumru::prelude::sitemap_greylisted_v1 as minisforumru_prelude_sitemap_greylisted_v1;
use minisforumru::prelude::sitemap_pause_v1 as minisforumru_prelude_sitemap_pause_v1;
use minisforumuk::prelude::classify_link_v1 as minisforumuk_prelude_classify_link_v1;
use minisforumuk::prelude::offer_detail_architecture_v1 as minisforumuk_prelude_offer_detail_architecture_v1;
use minisforumuk::prelude::offer_detail_normalize_v1 as minisforumuk_prelude_offer_detail_normalize_v1;
use minisforumuk::prelude::offer_detail_structuralize_v1 as minisforumuk_prelude_offer_detail_structuralize_v1;
use minisforumuk::prelude::sitemap_config_v1 as minisforumuk_prelude_sitemap_config_v1;
use minisforumuk::prelude::sitemap_greylisted_v1 as minisforumuk_prelude_sitemap_greylisted_v1;
use minisforumuk::prelude::sitemap_pause_v1 as minisforumuk_prelude_sitemap_pause_v1;
use minisforumus::prelude::classify_link_v1 as minisforumus_prelude_classify_link_v1;
use minisforumus::prelude::offer_detail_architecture_v1 as minisforumus_prelude_offer_detail_architecture_v1;
use minisforumus::prelude::offer_detail_normalize_v1 as minisforumus_prelude_offer_detail_normalize_v1;
use minisforumus::prelude::offer_detail_structuralize_v1 as minisforumus_prelude_offer_detail_structuralize_v1;
use minisforumus::prelude::sitemap_config_v1 as minisforumus_prelude_sitemap_config_v1;
use minisforumus::prelude::sitemap_greylisted_v1 as minisforumus_prelude_sitemap_greylisted_v1;
use minisforumus::prelude::sitemap_pause_v1 as minisforumus_prelude_sitemap_pause_v1;
use std::time::Duration;

use html_extractor::io::Architecture;
use retailer_sauce::structuralized::StructuralizedData;
use shared::SitemapConfig;
use shared::link::LinkKind;
use shared::prelude::{NormalizedData, RetailerCode};

pub fn architecture_for(retailer: RetailerCode) -> Option<Architecture> {
    match retailer {
        RetailerCode::MinisForumAu => Some(prelude_offer_detail_architecture_v1()),
        RetailerCode::MinisForumCa => Some(minisforumca_prelude_offer_detail_architecture_v1()),
        RetailerCode::MinisForumEu => Some(minisforumeu_prelude_offer_detail_architecture_v1()),
        RetailerCode::MinisForumFr => Some(minisforumfr_prelude_offer_detail_architecture_v1()),
        RetailerCode::MinisForumHk => Some(minisforumhk_prelude_offer_detail_architecture_v1()),
        RetailerCode::MinisForumJp => Some(minisforumjp_prelude_offer_detail_architecture_v1()),
        RetailerCode::MinisForumKr => Some(minisforumkr_prelude_offer_detail_architecture_v1()),
        RetailerCode::MinisForumRu => Some(minisforumru_prelude_offer_detail_architecture_v1()),
        RetailerCode::MinisForumUk => Some(minisforumuk_prelude_offer_detail_architecture_v1()),
        RetailerCode::MinisForumUs => Some(minisforumus_prelude_offer_detail_architecture_v1()),
        _ => None,
    }
}

pub fn normalize_for(retailer: RetailerCode, json: &str) -> Option<Result<NormalizedData, String>> {
    match retailer {
        RetailerCode::MinisForumAu => Some(prelude_offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumCa => Some(minisforumca_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumEu => Some(minisforumeu_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumFr => Some(minisforumfr_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumHk => Some(minisforumhk_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumJp => Some(minisforumjp_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumKr => Some(minisforumkr_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumRu => Some(minisforumru_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumUk => Some(minisforumuk_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::MinisForumUs => Some(minisforumus_prelude_offer_detail_normalize_v1(json)),
        _ => None,
    }
}

/// Only the storefronts with a structuralizer have an arm; every other code is `None`.
pub fn structuralize_for(
    retailer: RetailerCode,
    json: &str,
) -> Option<Result<StructuralizedData, String>> {
    match retailer {
        RetailerCode::MinisForumAu => Some(prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::MinisForumEu => {
            Some(minisforumeu_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumFr => {
            Some(minisforumfr_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumHk => {
            Some(minisforumhk_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumJp => {
            Some(minisforumjp_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumCa => {
            Some(minisforumca_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumKr => {
            Some(minisforumkr_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumRu => {
            Some(minisforumru_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumUk => {
            Some(minisforumuk_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::MinisForumUs => {
            Some(minisforumus_prelude_offer_detail_structuralize_v1(json))
        }
        _ => None,
    }
}

pub fn sitemap_config_for(retailer: &RetailerCode) -> Option<SitemapConfig> {
    match retailer {
        RetailerCode::MinisForumAu => Some(prelude_sitemap_config_v1()),
        RetailerCode::MinisForumCa => Some(minisforumca_prelude_sitemap_config_v1()),
        RetailerCode::MinisForumEu => Some(minisforumeu_prelude_sitemap_config_v1()),
        RetailerCode::MinisForumFr => Some(minisforumfr_prelude_sitemap_config_v1()),
        RetailerCode::MinisForumHk => Some(minisforumhk_prelude_sitemap_config_v1()),
        RetailerCode::MinisForumJp => Some(minisforumjp_prelude_sitemap_config_v1()),
        RetailerCode::MinisForumKr => Some(minisforumkr_prelude_sitemap_config_v1()),
        RetailerCode::MinisForumRu => Some(minisforumru_prelude_sitemap_config_v1()),
        RetailerCode::MinisForumUk => Some(minisforumuk_prelude_sitemap_config_v1()),
        RetailerCode::MinisForumUs => Some(minisforumus_prelude_sitemap_config_v1()),
        _ => None,
    }
}

pub fn sitemap_pause_for(retailer: RetailerCode) -> Option<Duration> {
    match retailer {
        RetailerCode::MinisForumAu => Some(prelude_sitemap_pause_v1()),
        RetailerCode::MinisForumCa => Some(minisforumca_prelude_sitemap_pause_v1()),
        RetailerCode::MinisForumEu => Some(minisforumeu_prelude_sitemap_pause_v1()),
        RetailerCode::MinisForumFr => Some(minisforumfr_prelude_sitemap_pause_v1()),
        RetailerCode::MinisForumHk => Some(minisforumhk_prelude_sitemap_pause_v1()),
        RetailerCode::MinisForumJp => Some(minisforumjp_prelude_sitemap_pause_v1()),
        RetailerCode::MinisForumKr => Some(minisforumkr_prelude_sitemap_pause_v1()),
        RetailerCode::MinisForumRu => Some(minisforumru_prelude_sitemap_pause_v1()),
        RetailerCode::MinisForumUk => Some(minisforumuk_prelude_sitemap_pause_v1()),
        RetailerCode::MinisForumUs => Some(minisforumus_prelude_sitemap_pause_v1()),
        _ => None,
    }
}

pub fn sitemap_greylisted_for(retailer: RetailerCode, url: &str) -> Option<bool> {
    match retailer {
        RetailerCode::MinisForumAu => Some(prelude_sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumCa => Some(minisforumca_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumEu => Some(minisforumeu_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumFr => Some(minisforumfr_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumHk => Some(minisforumhk_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumJp => Some(minisforumjp_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumKr => Some(minisforumkr_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumRu => Some(minisforumru_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumUk => Some(minisforumuk_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::MinisForumUs => Some(minisforumus_prelude_sitemap_greylisted_v1(url)),
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
        RetailerCode::MinisForumAu => Some(prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::MinisForumCa => Some(minisforumca_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumEu => Some(minisforumeu_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumFr => Some(minisforumfr_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumHk => Some(minisforumhk_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumJp => Some(minisforumjp_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumKr => Some(minisforumkr_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumRu => Some(minisforumru_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumUk => Some(minisforumuk_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::MinisForumUs => Some(minisforumus_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        _ => None,
    }
}
