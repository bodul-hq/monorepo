//! Looks up each retailer in this crate by its `RetailerCode`.

use crate::ankerau;
use crate::ankerca;
use crate::ankercom;
use crate::ankerde;
use crate::ankereu;
use crate::ankerfr;
use crate::ankeritalycom;
use crate::ankerjapancom;
use crate::ankerkr;
use crate::ankernordicscom;
use crate::ankernz;
use crate::ankerpl;
use crate::ankeruk;
use crate::ankervn;
use std::time::Duration;

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use bodul_shared::prelude::{NormalizedData, RetailerCode};
use html_extractor::io::Architecture;
use retailer_sauce::structuralized::StructuralizedData;

pub fn architecture_for(retailer: RetailerCode) -> Option<Architecture> {
    match retailer {
        RetailerCode::AnkerAu => Some(ankerau::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerCa => Some(ankerca::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerCom => Some(ankercom::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerDe => Some(ankerde::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerEu => Some(ankereu::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerFr => Some(ankerfr::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerKr => Some(ankerkr::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerNordicsCom => {
            Some(ankernordicscom::prelude::offer_detail_architecture_v1())
        }
        RetailerCode::AnkerNz => Some(ankernz::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerPl => Some(ankerpl::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerUk => Some(ankeruk::prelude::offer_detail_architecture_v1()),
        RetailerCode::AnkerVn => Some(ankervn::prelude::offer_detail_architecture_v1()),
        _ => None,
    }
}

pub fn normalize_for(retailer: RetailerCode, json: &str) -> Option<Result<NormalizedData, String>> {
    match retailer {
        RetailerCode::AnkerAu => Some(ankerau::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::AnkerCa => Some(ankerca::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::AnkerCom => Some(ankercom::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::AnkerDe => Some(ankerde::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::AnkerEu => Some(ankereu::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::AnkerFr => Some(ankerfr::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::AnkerItalyCom => {
            Some(ankeritalycom::prelude::offer_detail_normalize_v1(json))
        }
        RetailerCode::AnkerJapanCom => {
            Some(ankerjapancom::prelude::offer_detail_normalize_v1(json))
        }
        RetailerCode::AnkerKr => Some(ankerkr::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::AnkerNordicsCom => {
            Some(ankernordicscom::prelude::offer_detail_normalize_v1(json))
        }
        RetailerCode::AnkerNz => Some(ankernz::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::AnkerPl => Some(ankerpl::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::AnkerUk => Some(ankeruk::prelude::offer_detail_normalize_v1(json)),
        RetailerCode::AnkerVn => Some(ankervn::prelude::offer_detail_normalize_v1(json)),
        _ => None,
    }
}

/// `ankerkr` runs on Cafe24 and has no data island to read yet, so it hands over no document
/// rather than failing.
pub fn structuralize_for(
    retailer: RetailerCode,
    json: &str,
) -> Option<Result<StructuralizedData, String>> {
    match retailer {
        RetailerCode::AnkerAu => Some(ankerau::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerCa => Some(ankerca::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerCom => Some(ankercom::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerDe => Some(ankerde::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerEu => Some(ankereu::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerFr => Some(ankerfr::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerItalyCom => {
            Some(ankeritalycom::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::AnkerJapanCom => {
            Some(ankerjapancom::prelude::offer_detail_structuralize_v1(json))
        }
        RetailerCode::AnkerNordicsCom => Some(
            ankernordicscom::prelude::offer_detail_structuralize_v1(json),
        ),
        RetailerCode::AnkerNz => Some(ankernz::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerPl => Some(ankerpl::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerUk => Some(ankeruk::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerVn => Some(ankervn::prelude::offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerKr => Some(Ok(StructuralizedData::just_document_json(None))),
        _ => None,
    }
}

pub fn sitemap_config_for(retailer: &RetailerCode) -> Option<SitemapConfig> {
    match retailer {
        RetailerCode::AnkerAu => Some(ankerau::prelude::sitemap_config_v1()),
        RetailerCode::AnkerCa => Some(ankerca::prelude::sitemap_config_v1()),
        RetailerCode::AnkerCom => Some(ankercom::prelude::sitemap_config_v1()),
        RetailerCode::AnkerDe => Some(ankerde::prelude::sitemap_config_v1()),
        RetailerCode::AnkerEu => Some(ankereu::prelude::sitemap_config_v1()),
        RetailerCode::AnkerFr => Some(ankerfr::prelude::sitemap_config_v1()),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom::prelude::sitemap_config_v1()),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom::prelude::sitemap_config_v1()),
        RetailerCode::AnkerKr => Some(ankerkr::prelude::sitemap_config_v1()),
        RetailerCode::AnkerNordicsCom => Some(ankernordicscom::prelude::sitemap_config_v1()),
        RetailerCode::AnkerNz => Some(ankernz::prelude::sitemap_config_v1()),
        RetailerCode::AnkerPl => Some(ankerpl::prelude::sitemap_config_v1()),
        RetailerCode::AnkerUk => Some(ankeruk::prelude::sitemap_config_v1()),
        RetailerCode::AnkerVn => Some(ankervn::prelude::sitemap_config_v1()),
        _ => None,
    }
}

pub fn sitemap_pause_for(retailer: RetailerCode) -> Option<Duration> {
    match retailer {
        RetailerCode::AnkerAu => Some(ankerau::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerCa => Some(ankerca::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerCom => Some(ankercom::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerDe => Some(ankerde::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerEu => Some(ankereu::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerFr => Some(ankerfr::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerKr => Some(ankerkr::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerNordicsCom => Some(ankernordicscom::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerNz => Some(ankernz::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerPl => Some(ankerpl::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerUk => Some(ankeruk::prelude::sitemap_pause_v1()),
        RetailerCode::AnkerVn => Some(ankervn::prelude::sitemap_pause_v1()),
        _ => None,
    }
}

pub fn sitemap_greylisted_for(retailer: RetailerCode, url: &str) -> Option<bool> {
    match retailer {
        RetailerCode::AnkerAu => Some(ankerau::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerCa => Some(ankerca::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerCom => Some(ankercom::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerDe => Some(ankerde::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerEu => Some(ankereu::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerFr => Some(ankerfr::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerKr => Some(ankerkr::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerNordicsCom => Some(ankernordicscom::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerNz => Some(ankernz::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerPl => Some(ankerpl::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerUk => Some(ankeruk::prelude::sitemap_greylisted_v1(url)),
        RetailerCode::AnkerVn => Some(ankervn::prelude::sitemap_greylisted_v1(url)),
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
        RetailerCode::AnkerAu => Some(ankerau::prelude::classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerCa => Some(ankerca::prelude::classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerCom => Some(ankercom::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::AnkerDe => Some(ankerde::prelude::classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerEu => Some(ankereu::prelude::classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerFr => Some(ankerfr::prelude::classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::AnkerKr => Some(ankerkr::prelude::classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerNordicsCom => Some(ankernordicscom::prelude::classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::AnkerNz => Some(ankernz::prelude::classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerPl => Some(ankerpl::prelude::classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerUk => Some(ankeruk::prelude::classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerVn => Some(ankervn::prelude::classify_link_v1(url, source, image_count)),
        _ => None,
    }
}
