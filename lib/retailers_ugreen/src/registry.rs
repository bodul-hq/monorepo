//! Looks up each retailer in this crate by its `RetailerCode`.

use crate::ugreenca;
use crate::ugreencom;
use crate::ugreende;
use crate::ugreeneu;
use crate::ugreenfr;
use crate::ugreenin;
use crate::ugreenjp;
use crate::ugreenkr;
use crate::ugreennas;
use crate::ugreennasau;
use crate::ugreennasca;
use crate::ugreennasde;
use crate::ugreennases;
use crate::ugreennaseu;
use crate::ugreennasfr;
use crate::ugreennasit;
use crate::ugreennasjp;
use crate::ugreennasuk;
use crate::ugreennl;
use crate::ugreenuk;
use crate::ugreenus;
use std::time::Duration;

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use bodul_shared::prelude::{NormalizedData, RetailerCode};
use html_extractor::io::Architecture;
use retailer_sauce::structuralized::StructuralizedData;

pub fn architecture_for(retailer: RetailerCode) -> Option<Architecture> {
    match retailer {
        RetailerCode::UgreenCa => Some(ugreenca::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenCom => Some(ugreencom::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenDe => Some(ugreende::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenEu => Some(ugreeneu::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenFr => Some(ugreenfr::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenIn => Some(ugreenin::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenJp => Some(ugreenjp::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenKr => Some(ugreenkr::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNas => Some(ugreennas::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNasAu => Some(ugreennasau::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNasCa => Some(ugreennasca::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNasDe => Some(ugreennasde::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNasEs => Some(ugreennases::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNasEu => Some(ugreennaseu::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNasFr => Some(ugreennasfr::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNasIt => Some(ugreennasit::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNasJp => Some(ugreennasjp::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNasUk => Some(ugreennasuk::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenNl => Some(ugreennl::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenUk => Some(ugreenuk::prelude::offer_detail_architecture_v1()),
        RetailerCode::UgreenUs => Some(ugreenus::prelude::offer_detail_architecture_v1()),
        _ => None,
    }
}

pub fn normalize_for(retailer: RetailerCode, json: &str) -> Option<Result<NormalizedData, String>> {
    match retailer {
        RetailerCode::UgreenCa => Some(ugreenca::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenCom => Some(ugreencom::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenDe => Some(ugreende::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenEu => Some(ugreeneu::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenFr => Some(ugreenfr::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenIn => Some(ugreenin::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenJp => Some(ugreenjp::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenKr => Some(ugreenkr::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNas => Some(ugreennas::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasAu => Some(ugreennasau::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasCa => Some(ugreennasca::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasDe => Some(ugreennasde::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasEs => Some(ugreennases::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasEu => Some(ugreennaseu::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasFr => Some(ugreennasfr::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasIt => Some(ugreennasit::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasJp => Some(ugreennasjp::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasUk => Some(ugreennasuk::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNl => Some(ugreennl::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenUk => Some(ugreenuk::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::UgreenUs => Some(ugreenus::prelude::offer_detail_normalize_v1(json)),
        _ => None,
    }
}

/// Only the storefronts with a structuralizer have an arm; every other code is `None`.
pub fn structuralize_for(
    retailer: RetailerCode,
    json: &str,
) -> Option<Result<StructuralizedData, String>> {
    match retailer {
        RetailerCode::UgreenCom => Some(ugreencom::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenJp => Some(ugreenjp::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNas => Some(ugreennas::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasEu => {
            Some(ugreennaseu::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::UgreenNasFr => {
            Some(ugreennasfr::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::UgreenNasIt => {
            Some(ugreennasit::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::UgreenNasUk => {
            Some(ugreennasuk::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::UgreenEu => Some(ugreeneu::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenFr => Some(ugreenfr::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenIn => Some(ugreenin::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasAu => {
            Some(ugreennasau::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::UgreenNasCa => {
            Some(ugreennasca::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::UgreenNasDe => {
            Some(ugreennasde::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::UgreenNasEs => {
            Some(ugreennases::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::UgreenCa => Some(ugreenca::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenDe => Some(ugreende::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenKr => Some(ugreenkr::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNl => Some(ugreennl::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenUk => Some(ugreenuk::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenUs => Some(ugreenus::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasJp => {
            Some(ugreennasjp::prelude::offer_detail_structuralize_v1(json))
        }
        _ => None,
    }
}

pub fn sitemap_config_for(retailer: &RetailerCode) -> Option<SitemapConfig> {
    match retailer {
        RetailerCode::UgreenCa => Some(ugreenca::prelude::sitemap_config_v1()),
        RetailerCode::UgreenCom => Some(ugreencom::prelude::sitemap_config_v1()),
        RetailerCode::UgreenDe => Some(ugreende::prelude::sitemap_config_v1()),
        RetailerCode::UgreenEu => Some(ugreeneu::prelude::sitemap_config_v1()),
        RetailerCode::UgreenFr => Some(ugreenfr::prelude::sitemap_config_v1()),
        RetailerCode::UgreenIn => Some(ugreenin::prelude::sitemap_config_v1()),
        RetailerCode::UgreenJp => Some(ugreenjp::prelude::sitemap_config_v1()),
        RetailerCode::UgreenKr => Some(ugreenkr::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNas => Some(ugreennas::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNasAu => Some(ugreennasau::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNasCa => Some(ugreennasca::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNasDe => Some(ugreennasde::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNasEs => Some(ugreennases::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNasEu => Some(ugreennaseu::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNasFr => Some(ugreennasfr::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNasIt => Some(ugreennasit::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNasJp => Some(ugreennasjp::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNasUk => Some(ugreennasuk::prelude::sitemap_config_v1()),
        RetailerCode::UgreenNl => Some(ugreennl::prelude::sitemap_config_v1()),
        RetailerCode::UgreenUk => Some(ugreenuk::prelude::sitemap_config_v1()),
        RetailerCode::UgreenUs => Some(ugreenus::prelude::sitemap_config_v1()),
        _ => None,
    }
}

pub fn sitemap_pause_for(retailer: RetailerCode) -> Option<Duration> {
    match retailer {
        RetailerCode::UgreenCa => Some(ugreenca::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenCom => Some(ugreencom::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenDe => Some(ugreende::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenEu => Some(ugreeneu::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenFr => Some(ugreenfr::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenIn => Some(ugreenin::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenJp => Some(ugreenjp::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenKr => Some(ugreenkr::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNas => Some(ugreennas::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNasAu => Some(ugreennasau::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNasCa => Some(ugreennasca::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNasDe => Some(ugreennasde::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNasEs => Some(ugreennases::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNasEu => Some(ugreennaseu::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNasFr => Some(ugreennasfr::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNasIt => Some(ugreennasit::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNasJp => Some(ugreennasjp::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNasUk => Some(ugreennasuk::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenNl => Some(ugreennl::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenUk => Some(ugreenuk::prelude::sitemap_pause_v1()),
        RetailerCode::UgreenUs => Some(ugreenus::prelude::sitemap_pause_v1()),
        _ => None,
    }
}

pub fn sitemap_greylisted_for(retailer: RetailerCode, url: &str) -> Option<bool> {
    match retailer {
        RetailerCode::UgreenCa => Some(ugreenca::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenCom => Some(ugreencom::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenDe => Some(ugreende::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenEu => Some(ugreeneu::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenFr => Some(ugreenfr::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenIn => Some(ugreenin::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenJp => Some(ugreenjp::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenKr => Some(ugreenkr::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNas => Some(ugreennas::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasAu => Some(ugreennasau::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasCa => Some(ugreennasca::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasDe => Some(ugreennasde::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasEs => Some(ugreennases::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasEu => Some(ugreennaseu::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasFr => Some(ugreennasfr::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasIt => Some(ugreennasit::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasJp => Some(ugreennasjp::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasUk => Some(ugreennasuk::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNl => Some(ugreennl::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenUk => Some(ugreenuk::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::UgreenUs => Some(ugreenus::prelude::sitemap_greylisted_v1(url)),
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
        RetailerCode::UgreenCa => Some(ugreenca::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenCom => Some(ugreencom::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenDe => Some(ugreende::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenEu => Some(ugreeneu::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenFr => Some(ugreenfr::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenIn => Some(ugreenin::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenJp => Some(ugreenjp::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenKr => Some(ugreenkr::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNas => Some(ugreennas::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasAu => Some(ugreennasau::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasCa => Some(ugreennasca::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasDe => Some(ugreennasde::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasEs => Some(ugreennases::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasEu => Some(ugreennaseu::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasFr => Some(ugreennasfr::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasIt => Some(ugreennasit::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasJp => Some(ugreennasjp::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasUk => Some(ugreennasuk::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNl => Some(ugreennl::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenUk => Some(ugreenuk::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenUs => Some(ugreenus::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        _ => None,
    }
}
