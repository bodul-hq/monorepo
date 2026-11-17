//! Looks up each retailer in this crate by its `RetailerCode`.

use crate::ankerdk;
use crate::ankerfi;
use crate::ankeris;
use crate::ankerno;
use crate::ankerse;
use ankerau::prelude::classify_link_v1 as prelude_classify_link_v1;
use ankerau::prelude::offer_detail_architecture_v1 as prelude_offer_detail_architecture_v1;
use ankerau::prelude::offer_detail_normalize_v1 as prelude_offer_detail_normalize_v1;
use ankerau::prelude::offer_detail_structuralize_v1 as prelude_offer_detail_structuralize_v1;
use ankerau::prelude::sitemap_config_v1 as prelude_sitemap_config_v1;
use ankerau::prelude::sitemap_greylisted_v1 as prelude_sitemap_greylisted_v1;
use ankerau::prelude::sitemap_pause_v1 as prelude_sitemap_pause_v1;
use ankerca::prelude::classify_link_v1 as ankerca_prelude_classify_link_v1;
use ankerca::prelude::offer_detail_architecture_v1 as ankerca_prelude_offer_detail_architecture_v1;
use ankerca::prelude::offer_detail_normalize_v1 as ankerca_prelude_offer_detail_normalize_v1;
use ankerca::prelude::offer_detail_structuralize_v1 as ankerca_prelude_offer_detail_structuralize_v1;
use ankerca::prelude::sitemap_config_v1 as ankerca_prelude_sitemap_config_v1;
use ankerca::prelude::sitemap_greylisted_v1 as ankerca_prelude_sitemap_greylisted_v1;
use ankerca::prelude::sitemap_pause_v1 as ankerca_prelude_sitemap_pause_v1;
use ankercom::prelude::classify_link_v1 as ankercom_prelude_classify_link_v1;
use ankercom::prelude::offer_detail_architecture_v1 as ankercom_prelude_offer_detail_architecture_v1;
use ankercom::prelude::offer_detail_normalize_v1 as ankercom_prelude_offer_detail_normalize_v1;
use ankercom::prelude::offer_detail_structuralize_v1 as ankercom_prelude_offer_detail_structuralize_v1;
use ankercom::prelude::sitemap_config_v1 as ankercom_prelude_sitemap_config_v1;
use ankercom::prelude::sitemap_greylisted_v1 as ankercom_prelude_sitemap_greylisted_v1;
use ankercom::prelude::sitemap_pause_v1 as ankercom_prelude_sitemap_pause_v1;
use ankerde::prelude::classify_link_v1 as ankerde_prelude_classify_link_v1;
use ankerde::prelude::offer_detail_architecture_v1 as ankerde_prelude_offer_detail_architecture_v1;
use ankerde::prelude::offer_detail_normalize_v1 as ankerde_prelude_offer_detail_normalize_v1;
use ankerde::prelude::offer_detail_structuralize_v1 as ankerde_prelude_offer_detail_structuralize_v1;
use ankerde::prelude::sitemap_config_v1 as ankerde_prelude_sitemap_config_v1;
use ankerde::prelude::sitemap_greylisted_v1 as ankerde_prelude_sitemap_greylisted_v1;
use ankerde::prelude::sitemap_pause_v1 as ankerde_prelude_sitemap_pause_v1;
use ankerdk::prelude::classify_link_v1 as ankerdk_prelude_classify_link_v1;
use ankerdk::prelude::offer_detail_architecture_v1 as ankerdk_prelude_offer_detail_architecture_v1;
use ankerdk::prelude::offer_detail_normalize_v1 as ankerdk_prelude_offer_detail_normalize_v1;
use ankerdk::prelude::offer_detail_structuralize_v1 as ankerdk_prelude_offer_detail_structuralize_v1;
use ankerdk::prelude::sitemap_config_v1 as ankerdk_prelude_sitemap_config_v1;
use ankerdk::prelude::sitemap_greylisted_v1 as ankerdk_prelude_sitemap_greylisted_v1;
use ankerdk::prelude::sitemap_pause_v1 as ankerdk_prelude_sitemap_pause_v1;
use ankereu::prelude::classify_link_v1 as ankereu_prelude_classify_link_v1;
use ankereu::prelude::offer_detail_architecture_v1 as ankereu_prelude_offer_detail_architecture_v1;
use ankereu::prelude::offer_detail_normalize_v1 as ankereu_prelude_offer_detail_normalize_v1;
use ankereu::prelude::offer_detail_structuralize_v1 as ankereu_prelude_offer_detail_structuralize_v1;
use ankereu::prelude::sitemap_config_v1 as ankereu_prelude_sitemap_config_v1;
use ankereu::prelude::sitemap_greylisted_v1 as ankereu_prelude_sitemap_greylisted_v1;
use ankereu::prelude::sitemap_pause_v1 as ankereu_prelude_sitemap_pause_v1;
use ankerfi::prelude::classify_link_v1 as ankerfi_prelude_classify_link_v1;
use ankerfi::prelude::offer_detail_architecture_v1 as ankerfi_prelude_offer_detail_architecture_v1;
use ankerfi::prelude::offer_detail_normalize_v1 as ankerfi_prelude_offer_detail_normalize_v1;
use ankerfi::prelude::offer_detail_structuralize_v1 as ankerfi_prelude_offer_detail_structuralize_v1;
use ankerfi::prelude::sitemap_config_v1 as ankerfi_prelude_sitemap_config_v1;
use ankerfi::prelude::sitemap_greylisted_v1 as ankerfi_prelude_sitemap_greylisted_v1;
use ankerfi::prelude::sitemap_pause_v1 as ankerfi_prelude_sitemap_pause_v1;
use ankerfr::prelude::classify_link_v1 as ankerfr_prelude_classify_link_v1;
use ankerfr::prelude::offer_detail_architecture_v1 as ankerfr_prelude_offer_detail_architecture_v1;
use ankerfr::prelude::offer_detail_normalize_v1 as ankerfr_prelude_offer_detail_normalize_v1;
use ankerfr::prelude::offer_detail_structuralize_v1 as ankerfr_prelude_offer_detail_structuralize_v1;
use ankerfr::prelude::sitemap_config_v1 as ankerfr_prelude_sitemap_config_v1;
use ankerfr::prelude::sitemap_greylisted_v1 as ankerfr_prelude_sitemap_greylisted_v1;
use ankerfr::prelude::sitemap_pause_v1 as ankerfr_prelude_sitemap_pause_v1;
use ankeris::prelude::classify_link_v1 as ankeris_prelude_classify_link_v1;
use ankeris::prelude::offer_detail_architecture_v1 as ankeris_prelude_offer_detail_architecture_v1;
use ankeris::prelude::offer_detail_normalize_v1 as ankeris_prelude_offer_detail_normalize_v1;
use ankeris::prelude::offer_detail_structuralize_v1 as ankeris_prelude_offer_detail_structuralize_v1;
use ankeris::prelude::sitemap_config_v1 as ankeris_prelude_sitemap_config_v1;
use ankeris::prelude::sitemap_greylisted_v1 as ankeris_prelude_sitemap_greylisted_v1;
use ankeris::prelude::sitemap_pause_v1 as ankeris_prelude_sitemap_pause_v1;
use ankeritalycom::prelude::classify_link_v1 as ankeritalycom_prelude_classify_link_v1;
use ankeritalycom::prelude::offer_detail_architecture_v1 as ankeritalycom_prelude_offer_detail_architecture_v1;
use ankeritalycom::prelude::offer_detail_normalize_v1 as ankeritalycom_prelude_offer_detail_normalize_v1;
use ankeritalycom::prelude::offer_detail_structuralize_v1 as ankeritalycom_prelude_offer_detail_structuralize_v1;
use ankeritalycom::prelude::sitemap_config_v1 as ankeritalycom_prelude_sitemap_config_v1;
use ankeritalycom::prelude::sitemap_greylisted_v1 as ankeritalycom_prelude_sitemap_greylisted_v1;
use ankeritalycom::prelude::sitemap_pause_v1 as ankeritalycom_prelude_sitemap_pause_v1;
use ankerjapancom::prelude::classify_link_v1 as ankerjapancom_prelude_classify_link_v1;
use ankerjapancom::prelude::offer_detail_architecture_v1 as ankerjapancom_prelude_offer_detail_architecture_v1;
use ankerjapancom::prelude::offer_detail_normalize_v1 as ankerjapancom_prelude_offer_detail_normalize_v1;
use ankerjapancom::prelude::offer_detail_structuralize_v1 as ankerjapancom_prelude_offer_detail_structuralize_v1;
use ankerjapancom::prelude::sitemap_config_v1 as ankerjapancom_prelude_sitemap_config_v1;
use ankerjapancom::prelude::sitemap_greylisted_v1 as ankerjapancom_prelude_sitemap_greylisted_v1;
use ankerjapancom::prelude::sitemap_pause_v1 as ankerjapancom_prelude_sitemap_pause_v1;
use ankerkr::prelude::classify_link_v1 as ankerkr_prelude_classify_link_v1;
use ankerkr::prelude::offer_detail_architecture_v1 as ankerkr_prelude_offer_detail_architecture_v1;
use ankerkr::prelude::offer_detail_normalize_v1 as ankerkr_prelude_offer_detail_normalize_v1;
use ankerkr::prelude::sitemap_config_v1 as ankerkr_prelude_sitemap_config_v1;
use ankerkr::prelude::sitemap_greylisted_v1 as ankerkr_prelude_sitemap_greylisted_v1;
use ankerkr::prelude::sitemap_pause_v1 as ankerkr_prelude_sitemap_pause_v1;
use ankerno::prelude::classify_link_v1 as ankerno_prelude_classify_link_v1;
use ankerno::prelude::offer_detail_architecture_v1 as ankerno_prelude_offer_detail_architecture_v1;
use ankerno::prelude::offer_detail_normalize_v1 as ankerno_prelude_offer_detail_normalize_v1;
use ankerno::prelude::offer_detail_structuralize_v1 as ankerno_prelude_offer_detail_structuralize_v1;
use ankerno::prelude::sitemap_config_v1 as ankerno_prelude_sitemap_config_v1;
use ankerno::prelude::sitemap_greylisted_v1 as ankerno_prelude_sitemap_greylisted_v1;
use ankerno::prelude::sitemap_pause_v1 as ankerno_prelude_sitemap_pause_v1;
use ankernordicscom::prelude::classify_link_v1 as ankernordicscom_prelude_classify_link_v1;
use ankernordicscom::prelude::offer_detail_architecture_v1 as ankernordicscom_prelude_offer_detail_architecture_v1;
use ankernordicscom::prelude::offer_detail_normalize_v1 as ankernordicscom_prelude_offer_detail_normalize_v1;
use ankernordicscom::prelude::offer_detail_structuralize_v1 as ankernordicscom_prelude_offer_detail_structuralize_v1;
use ankernordicscom::prelude::sitemap_config_v1 as ankernordicscom_prelude_sitemap_config_v1;
use ankernordicscom::prelude::sitemap_greylisted_v1 as ankernordicscom_prelude_sitemap_greylisted_v1;
use ankernordicscom::prelude::sitemap_pause_v1 as ankernordicscom_prelude_sitemap_pause_v1;
use ankernz::prelude::classify_link_v1 as ankernz_prelude_classify_link_v1;
use ankernz::prelude::offer_detail_architecture_v1 as ankernz_prelude_offer_detail_architecture_v1;
use ankernz::prelude::offer_detail_normalize_v1 as ankernz_prelude_offer_detail_normalize_v1;
use ankernz::prelude::offer_detail_structuralize_v1 as ankernz_prelude_offer_detail_structuralize_v1;
use ankernz::prelude::sitemap_config_v1 as ankernz_prelude_sitemap_config_v1;
use ankernz::prelude::sitemap_greylisted_v1 as ankernz_prelude_sitemap_greylisted_v1;
use ankernz::prelude::sitemap_pause_v1 as ankernz_prelude_sitemap_pause_v1;
use ankerpl::prelude::classify_link_v1 as ankerpl_prelude_classify_link_v1;
use ankerpl::prelude::offer_detail_architecture_v1 as ankerpl_prelude_offer_detail_architecture_v1;
use ankerpl::prelude::offer_detail_normalize_v1 as ankerpl_prelude_offer_detail_normalize_v1;
use ankerpl::prelude::offer_detail_structuralize_v1 as ankerpl_prelude_offer_detail_structuralize_v1;
use ankerpl::prelude::sitemap_config_v1 as ankerpl_prelude_sitemap_config_v1;
use ankerpl::prelude::sitemap_greylisted_v1 as ankerpl_prelude_sitemap_greylisted_v1;
use ankerpl::prelude::sitemap_pause_v1 as ankerpl_prelude_sitemap_pause_v1;
use ankerse::prelude::classify_link_v1 as ankerse_prelude_classify_link_v1;
use ankerse::prelude::offer_detail_architecture_v1 as ankerse_prelude_offer_detail_architecture_v1;
use ankerse::prelude::offer_detail_normalize_v1 as ankerse_prelude_offer_detail_normalize_v1;
use ankerse::prelude::offer_detail_structuralize_v1 as ankerse_prelude_offer_detail_structuralize_v1;
use ankerse::prelude::sitemap_config_v1 as ankerse_prelude_sitemap_config_v1;
use ankerse::prelude::sitemap_greylisted_v1 as ankerse_prelude_sitemap_greylisted_v1;
use ankerse::prelude::sitemap_pause_v1 as ankerse_prelude_sitemap_pause_v1;
use ankeruk::prelude::classify_link_v1 as ankeruk_prelude_classify_link_v1;
use ankeruk::prelude::offer_detail_architecture_v1 as ankeruk_prelude_offer_detail_architecture_v1;
use ankeruk::prelude::offer_detail_normalize_v1 as ankeruk_prelude_offer_detail_normalize_v1;
use ankeruk::prelude::offer_detail_structuralize_v1 as ankeruk_prelude_offer_detail_structuralize_v1;
use ankeruk::prelude::sitemap_config_v1 as ankeruk_prelude_sitemap_config_v1;
use ankeruk::prelude::sitemap_greylisted_v1 as ankeruk_prelude_sitemap_greylisted_v1;
use ankeruk::prelude::sitemap_pause_v1 as ankeruk_prelude_sitemap_pause_v1;
use ankervn::prelude::classify_link_v1 as ankervn_prelude_classify_link_v1;
use ankervn::prelude::offer_detail_architecture_v1 as ankervn_prelude_offer_detail_architecture_v1;
use ankervn::prelude::offer_detail_normalize_v1 as ankervn_prelude_offer_detail_normalize_v1;
use ankervn::prelude::offer_detail_structuralize_v1 as ankervn_prelude_offer_detail_structuralize_v1;
use ankervn::prelude::sitemap_config_v1 as ankervn_prelude_sitemap_config_v1;
use ankervn::prelude::sitemap_greylisted_v1 as ankervn_prelude_sitemap_greylisted_v1;
use ankervn::prelude::sitemap_pause_v1 as ankervn_prelude_sitemap_pause_v1;

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

use html_extractor::io::Architecture;
use retailer_sauce::structuralized::StructuralizedData;
use shared::SitemapConfig;
use shared::link::LinkKind;
use shared::prelude::{NormalizedData, RetailerCode};

pub fn architecture_for(retailer: RetailerCode) -> Option<Architecture> {
    match retailer {
        RetailerCode::AnkerDk => Some(ankerdk_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerFi => Some(ankerfi_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerIs => Some(ankeris_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerNo => Some(ankerno_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerSe => Some(ankerse_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerAu => Some(prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerCa => Some(ankerca_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerCom => Some(ankercom_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerDe => Some(ankerde_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerEu => Some(ankereu_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerFr => Some(ankerfr_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerKr => Some(ankerkr_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerNordicsCom => {
            Some(ankernordicscom_prelude_offer_detail_architecture_v1())
        }
        RetailerCode::AnkerNz => Some(ankernz_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerPl => Some(ankerpl_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerUk => Some(ankeruk_prelude_offer_detail_architecture_v1()),
        RetailerCode::AnkerVn => Some(ankervn_prelude_offer_detail_architecture_v1()),
        _ => None,
    }
}

pub fn normalize_for(retailer: RetailerCode, json: &str) -> Option<Result<NormalizedData, String>> {
    match retailer {
        RetailerCode::AnkerDk => Some(ankerdk_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerFi => Some(ankerfi_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerIs => Some(ankeris_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerNo => Some(ankerno_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerSe => Some(ankerse_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerAu => Some(prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerCa => Some(ankerca_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerCom => Some(ankercom_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerDe => Some(ankerde_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerEu => Some(ankereu_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerFr => Some(ankerfr_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerKr => Some(ankerkr_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerNordicsCom => {
            Some(ankernordicscom_prelude_offer_detail_normalize_v1(json))
        }
        RetailerCode::AnkerNz => Some(ankernz_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerPl => Some(ankerpl_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerUk => Some(ankeruk_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::AnkerVn => Some(ankervn_prelude_offer_detail_normalize_v1(json)),
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
        RetailerCode::AnkerDk => Some(ankerdk_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerFi => Some(ankerfi_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerIs => Some(ankeris_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerNo => Some(ankerno_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerSe => Some(ankerse_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerAu => Some(prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerCa => Some(ankerca_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerCom => Some(ankercom_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerDe => Some(ankerde_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerEu => Some(ankereu_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerFr => Some(ankerfr_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerItalyCom => {
            Some(ankeritalycom_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::AnkerJapanCom => {
            Some(ankerjapancom_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::AnkerNordicsCom => {
            Some(ankernordicscom_prelude_offer_detail_structuralize_v1(json))
        }
        RetailerCode::AnkerNz => Some(ankernz_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerPl => Some(ankerpl_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerUk => Some(ankeruk_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerVn => Some(ankervn_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::AnkerKr => Some(Ok(StructuralizedData::just_document_json(None))),
        _ => None,
    }
}

pub fn sitemap_config_for(retailer: &RetailerCode) -> Option<SitemapConfig> {
    match retailer {
        RetailerCode::AnkerDk => Some(ankerdk_prelude_sitemap_config_v1()),
        RetailerCode::AnkerFi => Some(ankerfi_prelude_sitemap_config_v1()),
        RetailerCode::AnkerIs => Some(ankeris_prelude_sitemap_config_v1()),
        RetailerCode::AnkerNo => Some(ankerno_prelude_sitemap_config_v1()),
        RetailerCode::AnkerSe => Some(ankerse_prelude_sitemap_config_v1()),
        RetailerCode::AnkerAu => Some(prelude_sitemap_config_v1()),
        RetailerCode::AnkerCa => Some(ankerca_prelude_sitemap_config_v1()),
        RetailerCode::AnkerCom => Some(ankercom_prelude_sitemap_config_v1()),
        RetailerCode::AnkerDe => Some(ankerde_prelude_sitemap_config_v1()),
        RetailerCode::AnkerEu => Some(ankereu_prelude_sitemap_config_v1()),
        RetailerCode::AnkerFr => Some(ankerfr_prelude_sitemap_config_v1()),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom_prelude_sitemap_config_v1()),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom_prelude_sitemap_config_v1()),
        RetailerCode::AnkerKr => Some(ankerkr_prelude_sitemap_config_v1()),
        RetailerCode::AnkerNordicsCom => Some(ankernordicscom_prelude_sitemap_config_v1()),
        RetailerCode::AnkerNz => Some(ankernz_prelude_sitemap_config_v1()),
        RetailerCode::AnkerPl => Some(ankerpl_prelude_sitemap_config_v1()),
        RetailerCode::AnkerUk => Some(ankeruk_prelude_sitemap_config_v1()),
        RetailerCode::AnkerVn => Some(ankervn_prelude_sitemap_config_v1()),
        _ => None,
    }
}

pub fn sitemap_pause_for(retailer: RetailerCode) -> Option<Duration> {
    match retailer {
        RetailerCode::AnkerDk => Some(ankerdk_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerFi => Some(ankerfi_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerIs => Some(ankeris_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerNo => Some(ankerno_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerSe => Some(ankerse_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerAu => Some(prelude_sitemap_pause_v1()),
        RetailerCode::AnkerCa => Some(ankerca_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerCom => Some(ankercom_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerDe => Some(ankerde_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerEu => Some(ankereu_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerFr => Some(ankerfr_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerKr => Some(ankerkr_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerNordicsCom => Some(ankernordicscom_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerNz => Some(ankernz_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerPl => Some(ankerpl_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerUk => Some(ankeruk_prelude_sitemap_pause_v1()),
        RetailerCode::AnkerVn => Some(ankervn_prelude_sitemap_pause_v1()),
        _ => None,
    }
}

pub fn sitemap_greylisted_for(retailer: RetailerCode, url: &str) -> Option<bool> {
    match retailer {
        RetailerCode::AnkerDk => Some(ankerdk_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerFi => Some(ankerfi_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerIs => Some(ankeris_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerNo => Some(ankerno_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerSe => Some(ankerse_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerAu => Some(prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerCa => Some(ankerca_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerCom => Some(ankercom_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerDe => Some(ankerde_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerEu => Some(ankereu_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerFr => Some(ankerfr_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerKr => Some(ankerkr_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerNordicsCom => Some(ankernordicscom_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerNz => Some(ankernz_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerPl => Some(ankerpl_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerUk => Some(ankeruk_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::AnkerVn => Some(ankervn_prelude_sitemap_greylisted_v1(url)),
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
        RetailerCode::AnkerDk => Some(ankerdk_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerFi => Some(ankerfi_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerIs => Some(ankeris_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerNo => Some(ankerno_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerSe => Some(ankerse_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerAu => Some(prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerCa => Some(ankerca_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerCom => Some(ankercom_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerDe => Some(ankerde_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerEu => Some(ankereu_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerFr => Some(ankerfr_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::AnkerKr => Some(ankerkr_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerNordicsCom => Some(ankernordicscom_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::AnkerNz => Some(ankernz_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerPl => Some(ankerpl_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerUk => Some(ankeruk_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::AnkerVn => Some(ankervn_prelude_classify_link_v1(url, source, image_count)),
        _ => None,
    }
}
