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
use ugreenca::prelude::classify_link_v1 as prelude_classify_link_v1;
use ugreenca::prelude::offer_detail_architecture_v1 as prelude_offer_detail_architecture_v1;
use ugreenca::prelude::offer_detail_normalize_v1 as prelude_offer_detail_normalize_v1;
use ugreenca::prelude::offer_detail_structuralize_v1 as prelude_offer_detail_structuralize_v1;
use ugreenca::prelude::sitemap_config_v1 as prelude_sitemap_config_v1;
use ugreenca::prelude::sitemap_greylisted_v1 as prelude_sitemap_greylisted_v1;
use ugreenca::prelude::sitemap_pause_v1 as prelude_sitemap_pause_v1;
use ugreencom::prelude::classify_link_v1 as ugreencom_prelude_classify_link_v1;
use ugreencom::prelude::offer_detail_architecture_v1 as ugreencom_prelude_offer_detail_architecture_v1;
use ugreencom::prelude::offer_detail_normalize_v1 as ugreencom_prelude_offer_detail_normalize_v1;
use ugreencom::prelude::offer_detail_structuralize_v1 as ugreencom_prelude_offer_detail_structuralize_v1;
use ugreencom::prelude::sitemap_config_v1 as ugreencom_prelude_sitemap_config_v1;
use ugreencom::prelude::sitemap_greylisted_v1 as ugreencom_prelude_sitemap_greylisted_v1;
use ugreencom::prelude::sitemap_pause_v1 as ugreencom_prelude_sitemap_pause_v1;
use ugreende::prelude::classify_link_v1 as ugreende_prelude_classify_link_v1;
use ugreende::prelude::offer_detail_architecture_v1 as ugreende_prelude_offer_detail_architecture_v1;
use ugreende::prelude::offer_detail_normalize_v1 as ugreende_prelude_offer_detail_normalize_v1;
use ugreende::prelude::offer_detail_structuralize_v1 as ugreende_prelude_offer_detail_structuralize_v1;
use ugreende::prelude::sitemap_config_v1 as ugreende_prelude_sitemap_config_v1;
use ugreende::prelude::sitemap_greylisted_v1 as ugreende_prelude_sitemap_greylisted_v1;
use ugreende::prelude::sitemap_pause_v1 as ugreende_prelude_sitemap_pause_v1;
use ugreeneu::prelude::classify_link_v1 as ugreeneu_prelude_classify_link_v1;
use ugreeneu::prelude::offer_detail_architecture_v1 as ugreeneu_prelude_offer_detail_architecture_v1;
use ugreeneu::prelude::offer_detail_normalize_v1 as ugreeneu_prelude_offer_detail_normalize_v1;
use ugreeneu::prelude::offer_detail_structuralize_v1 as ugreeneu_prelude_offer_detail_structuralize_v1;
use ugreeneu::prelude::sitemap_config_v1 as ugreeneu_prelude_sitemap_config_v1;
use ugreeneu::prelude::sitemap_greylisted_v1 as ugreeneu_prelude_sitemap_greylisted_v1;
use ugreeneu::prelude::sitemap_pause_v1 as ugreeneu_prelude_sitemap_pause_v1;
use ugreenfr::prelude::classify_link_v1 as ugreenfr_prelude_classify_link_v1;
use ugreenfr::prelude::offer_detail_architecture_v1 as ugreenfr_prelude_offer_detail_architecture_v1;
use ugreenfr::prelude::offer_detail_normalize_v1 as ugreenfr_prelude_offer_detail_normalize_v1;
use ugreenfr::prelude::offer_detail_structuralize_v1 as ugreenfr_prelude_offer_detail_structuralize_v1;
use ugreenfr::prelude::sitemap_config_v1 as ugreenfr_prelude_sitemap_config_v1;
use ugreenfr::prelude::sitemap_greylisted_v1 as ugreenfr_prelude_sitemap_greylisted_v1;
use ugreenfr::prelude::sitemap_pause_v1 as ugreenfr_prelude_sitemap_pause_v1;
use ugreenin::prelude::classify_link_v1 as ugreenin_prelude_classify_link_v1;
use ugreenin::prelude::offer_detail_architecture_v1 as ugreenin_prelude_offer_detail_architecture_v1;
use ugreenin::prelude::offer_detail_normalize_v1 as ugreenin_prelude_offer_detail_normalize_v1;
use ugreenin::prelude::offer_detail_structuralize_v1 as ugreenin_prelude_offer_detail_structuralize_v1;
use ugreenin::prelude::sitemap_config_v1 as ugreenin_prelude_sitemap_config_v1;
use ugreenin::prelude::sitemap_greylisted_v1 as ugreenin_prelude_sitemap_greylisted_v1;
use ugreenin::prelude::sitemap_pause_v1 as ugreenin_prelude_sitemap_pause_v1;
use ugreenjp::prelude::classify_link_v1 as ugreenjp_prelude_classify_link_v1;
use ugreenjp::prelude::offer_detail_architecture_v1 as ugreenjp_prelude_offer_detail_architecture_v1;
use ugreenjp::prelude::offer_detail_normalize_v1 as ugreenjp_prelude_offer_detail_normalize_v1;
use ugreenjp::prelude::offer_detail_structuralize_v1 as ugreenjp_prelude_offer_detail_structuralize_v1;
use ugreenjp::prelude::sitemap_config_v1 as ugreenjp_prelude_sitemap_config_v1;
use ugreenjp::prelude::sitemap_greylisted_v1 as ugreenjp_prelude_sitemap_greylisted_v1;
use ugreenjp::prelude::sitemap_pause_v1 as ugreenjp_prelude_sitemap_pause_v1;
use ugreenkr::prelude::classify_link_v1 as ugreenkr_prelude_classify_link_v1;
use ugreenkr::prelude::offer_detail_architecture_v1 as ugreenkr_prelude_offer_detail_architecture_v1;
use ugreenkr::prelude::offer_detail_normalize_v1 as ugreenkr_prelude_offer_detail_normalize_v1;
use ugreenkr::prelude::offer_detail_structuralize_v1 as ugreenkr_prelude_offer_detail_structuralize_v1;
use ugreenkr::prelude::sitemap_config_v1 as ugreenkr_prelude_sitemap_config_v1;
use ugreenkr::prelude::sitemap_greylisted_v1 as ugreenkr_prelude_sitemap_greylisted_v1;
use ugreenkr::prelude::sitemap_pause_v1 as ugreenkr_prelude_sitemap_pause_v1;
use ugreennas::prelude::classify_link_v1 as ugreennas_prelude_classify_link_v1;
use ugreennas::prelude::offer_detail_architecture_v1 as ugreennas_prelude_offer_detail_architecture_v1;
use ugreennas::prelude::offer_detail_normalize_v1 as ugreennas_prelude_offer_detail_normalize_v1;
use ugreennas::prelude::offer_detail_structuralize_v1 as ugreennas_prelude_offer_detail_structuralize_v1;
use ugreennas::prelude::sitemap_config_v1 as ugreennas_prelude_sitemap_config_v1;
use ugreennas::prelude::sitemap_greylisted_v1 as ugreennas_prelude_sitemap_greylisted_v1;
use ugreennas::prelude::sitemap_pause_v1 as ugreennas_prelude_sitemap_pause_v1;
use ugreennasau::prelude::classify_link_v1 as ugreennasau_prelude_classify_link_v1;
use ugreennasau::prelude::offer_detail_architecture_v1 as ugreennasau_prelude_offer_detail_architecture_v1;
use ugreennasau::prelude::offer_detail_normalize_v1 as ugreennasau_prelude_offer_detail_normalize_v1;
use ugreennasau::prelude::offer_detail_structuralize_v1 as ugreennasau_prelude_offer_detail_structuralize_v1;
use ugreennasau::prelude::sitemap_config_v1 as ugreennasau_prelude_sitemap_config_v1;
use ugreennasau::prelude::sitemap_greylisted_v1 as ugreennasau_prelude_sitemap_greylisted_v1;
use ugreennasau::prelude::sitemap_pause_v1 as ugreennasau_prelude_sitemap_pause_v1;
use ugreennasca::prelude::classify_link_v1 as ugreennasca_prelude_classify_link_v1;
use ugreennasca::prelude::offer_detail_architecture_v1 as ugreennasca_prelude_offer_detail_architecture_v1;
use ugreennasca::prelude::offer_detail_normalize_v1 as ugreennasca_prelude_offer_detail_normalize_v1;
use ugreennasca::prelude::offer_detail_structuralize_v1 as ugreennasca_prelude_offer_detail_structuralize_v1;
use ugreennasca::prelude::sitemap_config_v1 as ugreennasca_prelude_sitemap_config_v1;
use ugreennasca::prelude::sitemap_greylisted_v1 as ugreennasca_prelude_sitemap_greylisted_v1;
use ugreennasca::prelude::sitemap_pause_v1 as ugreennasca_prelude_sitemap_pause_v1;
use ugreennasde::prelude::classify_link_v1 as ugreennasde_prelude_classify_link_v1;
use ugreennasde::prelude::offer_detail_architecture_v1 as ugreennasde_prelude_offer_detail_architecture_v1;
use ugreennasde::prelude::offer_detail_normalize_v1 as ugreennasde_prelude_offer_detail_normalize_v1;
use ugreennasde::prelude::offer_detail_structuralize_v1 as ugreennasde_prelude_offer_detail_structuralize_v1;
use ugreennasde::prelude::sitemap_config_v1 as ugreennasde_prelude_sitemap_config_v1;
use ugreennasde::prelude::sitemap_greylisted_v1 as ugreennasde_prelude_sitemap_greylisted_v1;
use ugreennasde::prelude::sitemap_pause_v1 as ugreennasde_prelude_sitemap_pause_v1;
use ugreennases::prelude::classify_link_v1 as ugreennases_prelude_classify_link_v1;
use ugreennases::prelude::offer_detail_architecture_v1 as ugreennases_prelude_offer_detail_architecture_v1;
use ugreennases::prelude::offer_detail_normalize_v1 as ugreennases_prelude_offer_detail_normalize_v1;
use ugreennases::prelude::offer_detail_structuralize_v1 as ugreennases_prelude_offer_detail_structuralize_v1;
use ugreennases::prelude::sitemap_config_v1 as ugreennases_prelude_sitemap_config_v1;
use ugreennases::prelude::sitemap_greylisted_v1 as ugreennases_prelude_sitemap_greylisted_v1;
use ugreennases::prelude::sitemap_pause_v1 as ugreennases_prelude_sitemap_pause_v1;
use ugreennaseu::prelude::classify_link_v1 as ugreennaseu_prelude_classify_link_v1;
use ugreennaseu::prelude::offer_detail_architecture_v1 as ugreennaseu_prelude_offer_detail_architecture_v1;
use ugreennaseu::prelude::offer_detail_normalize_v1 as ugreennaseu_prelude_offer_detail_normalize_v1;
use ugreennaseu::prelude::offer_detail_structuralize_v1 as ugreennaseu_prelude_offer_detail_structuralize_v1;
use ugreennaseu::prelude::sitemap_config_v1 as ugreennaseu_prelude_sitemap_config_v1;
use ugreennaseu::prelude::sitemap_greylisted_v1 as ugreennaseu_prelude_sitemap_greylisted_v1;
use ugreennaseu::prelude::sitemap_pause_v1 as ugreennaseu_prelude_sitemap_pause_v1;
use ugreennasfr::prelude::classify_link_v1 as ugreennasfr_prelude_classify_link_v1;
use ugreennasfr::prelude::offer_detail_architecture_v1 as ugreennasfr_prelude_offer_detail_architecture_v1;
use ugreennasfr::prelude::offer_detail_normalize_v1 as ugreennasfr_prelude_offer_detail_normalize_v1;
use ugreennasfr::prelude::offer_detail_structuralize_v1 as ugreennasfr_prelude_offer_detail_structuralize_v1;
use ugreennasfr::prelude::sitemap_config_v1 as ugreennasfr_prelude_sitemap_config_v1;
use ugreennasfr::prelude::sitemap_greylisted_v1 as ugreennasfr_prelude_sitemap_greylisted_v1;
use ugreennasfr::prelude::sitemap_pause_v1 as ugreennasfr_prelude_sitemap_pause_v1;
use ugreennasit::prelude::classify_link_v1 as ugreennasit_prelude_classify_link_v1;
use ugreennasit::prelude::offer_detail_architecture_v1 as ugreennasit_prelude_offer_detail_architecture_v1;
use ugreennasit::prelude::offer_detail_normalize_v1 as ugreennasit_prelude_offer_detail_normalize_v1;
use ugreennasit::prelude::offer_detail_structuralize_v1 as ugreennasit_prelude_offer_detail_structuralize_v1;
use ugreennasit::prelude::sitemap_config_v1 as ugreennasit_prelude_sitemap_config_v1;
use ugreennasit::prelude::sitemap_greylisted_v1 as ugreennasit_prelude_sitemap_greylisted_v1;
use ugreennasit::prelude::sitemap_pause_v1 as ugreennasit_prelude_sitemap_pause_v1;
use ugreennasjp::prelude::classify_link_v1 as ugreennasjp_prelude_classify_link_v1;
use ugreennasjp::prelude::offer_detail_architecture_v1 as ugreennasjp_prelude_offer_detail_architecture_v1;
use ugreennasjp::prelude::offer_detail_normalize_v1 as ugreennasjp_prelude_offer_detail_normalize_v1;
use ugreennasjp::prelude::offer_detail_structuralize_v1 as ugreennasjp_prelude_offer_detail_structuralize_v1;
use ugreennasjp::prelude::sitemap_config_v1 as ugreennasjp_prelude_sitemap_config_v1;
use ugreennasjp::prelude::sitemap_greylisted_v1 as ugreennasjp_prelude_sitemap_greylisted_v1;
use ugreennasjp::prelude::sitemap_pause_v1 as ugreennasjp_prelude_sitemap_pause_v1;
use ugreennasuk::prelude::classify_link_v1 as ugreennasuk_prelude_classify_link_v1;
use ugreennasuk::prelude::offer_detail_architecture_v1 as ugreennasuk_prelude_offer_detail_architecture_v1;
use ugreennasuk::prelude::offer_detail_normalize_v1 as ugreennasuk_prelude_offer_detail_normalize_v1;
use ugreennasuk::prelude::offer_detail_structuralize_v1 as ugreennasuk_prelude_offer_detail_structuralize_v1;
use ugreennasuk::prelude::sitemap_config_v1 as ugreennasuk_prelude_sitemap_config_v1;
use ugreennasuk::prelude::sitemap_greylisted_v1 as ugreennasuk_prelude_sitemap_greylisted_v1;
use ugreennasuk::prelude::sitemap_pause_v1 as ugreennasuk_prelude_sitemap_pause_v1;
use ugreennl::prelude::classify_link_v1 as ugreennl_prelude_classify_link_v1;
use ugreennl::prelude::offer_detail_architecture_v1 as ugreennl_prelude_offer_detail_architecture_v1;
use ugreennl::prelude::offer_detail_normalize_v1 as ugreennl_prelude_offer_detail_normalize_v1;
use ugreennl::prelude::offer_detail_structuralize_v1 as ugreennl_prelude_offer_detail_structuralize_v1;
use ugreennl::prelude::sitemap_config_v1 as ugreennl_prelude_sitemap_config_v1;
use ugreennl::prelude::sitemap_greylisted_v1 as ugreennl_prelude_sitemap_greylisted_v1;
use ugreennl::prelude::sitemap_pause_v1 as ugreennl_prelude_sitemap_pause_v1;
use ugreenuk::prelude::classify_link_v1 as ugreenuk_prelude_classify_link_v1;
use ugreenuk::prelude::offer_detail_architecture_v1 as ugreenuk_prelude_offer_detail_architecture_v1;
use ugreenuk::prelude::offer_detail_normalize_v1 as ugreenuk_prelude_offer_detail_normalize_v1;
use ugreenuk::prelude::offer_detail_structuralize_v1 as ugreenuk_prelude_offer_detail_structuralize_v1;
use ugreenuk::prelude::sitemap_config_v1 as ugreenuk_prelude_sitemap_config_v1;
use ugreenuk::prelude::sitemap_greylisted_v1 as ugreenuk_prelude_sitemap_greylisted_v1;
use ugreenuk::prelude::sitemap_pause_v1 as ugreenuk_prelude_sitemap_pause_v1;
use ugreenus::prelude::classify_link_v1 as ugreenus_prelude_classify_link_v1;
use ugreenus::prelude::offer_detail_architecture_v1 as ugreenus_prelude_offer_detail_architecture_v1;
use ugreenus::prelude::offer_detail_normalize_v1 as ugreenus_prelude_offer_detail_normalize_v1;
use ugreenus::prelude::offer_detail_structuralize_v1 as ugreenus_prelude_offer_detail_structuralize_v1;
use ugreenus::prelude::sitemap_config_v1 as ugreenus_prelude_sitemap_config_v1;
use ugreenus::prelude::sitemap_greylisted_v1 as ugreenus_prelude_sitemap_greylisted_v1;
use ugreenus::prelude::sitemap_pause_v1 as ugreenus_prelude_sitemap_pause_v1;

use html_extractor::io::Architecture;
use retailer_sauce::structuralized::StructuralizedData;
use shared::SitemapConfig;
use shared::link::LinkKind;
use shared::prelude::{NormalizedData, RetailerCode};

pub fn architecture_for(retailer: RetailerCode) -> Option<Architecture> {
    match retailer {
        RetailerCode::UgreenCa => Some(prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenCom => Some(ugreencom_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenDe => Some(ugreende_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenEu => Some(ugreeneu_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenFr => Some(ugreenfr_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenIn => Some(ugreenin_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenJp => Some(ugreenjp_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenKr => Some(ugreenkr_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNas => Some(ugreennas_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNasAu => Some(ugreennasau_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNasCa => Some(ugreennasca_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNasDe => Some(ugreennasde_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNasEs => Some(ugreennases_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNasEu => Some(ugreennaseu_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNasFr => Some(ugreennasfr_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNasIt => Some(ugreennasit_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNasJp => Some(ugreennasjp_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNasUk => Some(ugreennasuk_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenNl => Some(ugreennl_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenUk => Some(ugreenuk_prelude_offer_detail_architecture_v1()),
        RetailerCode::UgreenUs => Some(ugreenus_prelude_offer_detail_architecture_v1()),
        _ => None,
    }
}

pub fn normalize_for(retailer: RetailerCode, json: &str) -> Option<Result<NormalizedData, String>> {
    match retailer {
        RetailerCode::UgreenCa => Some(prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenCom => Some(ugreencom_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenDe => Some(ugreende_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenEu => Some(ugreeneu_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenFr => Some(ugreenfr_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenIn => Some(ugreenin_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenJp => Some(ugreenjp_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenKr => Some(ugreenkr_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNas => Some(ugreennas_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasAu => Some(ugreennasau_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasCa => Some(ugreennasca_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasDe => Some(ugreennasde_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasEs => Some(ugreennases_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasEu => Some(ugreennaseu_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasFr => Some(ugreennasfr_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasIt => Some(ugreennasit_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasJp => Some(ugreennasjp_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNasUk => Some(ugreennasuk_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenNl => Some(ugreennl_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenUk => Some(ugreenuk_prelude_offer_detail_normalize_v1(json)),
        RetailerCode::UgreenUs => Some(ugreenus_prelude_offer_detail_normalize_v1(json)),
        _ => None,
    }
}

/// Only the storefronts with a structuralizer have an arm; every other code is `None`.
pub fn structuralize_for(
    retailer: RetailerCode,
    json: &str,
) -> Option<Result<StructuralizedData, String>> {
    match retailer {
        RetailerCode::UgreenCom => Some(ugreencom_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenJp => Some(ugreenjp_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNas => Some(ugreennas_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasEu => Some(ugreennaseu_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasFr => Some(ugreennasfr_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasIt => Some(ugreennasit_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasUk => Some(ugreennasuk_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenEu => Some(ugreeneu_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenFr => Some(ugreenfr_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenIn => Some(ugreenin_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasAu => Some(ugreennasau_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasCa => Some(ugreennasca_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasDe => Some(ugreennasde_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasEs => Some(ugreennases_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenCa => Some(prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenDe => Some(ugreende_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenKr => Some(ugreenkr_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNl => Some(ugreennl_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenUk => Some(ugreenuk_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenUs => Some(ugreenus_prelude_offer_detail_structuralize_v1(json)),
        RetailerCode::UgreenNasJp => Some(ugreennasjp_prelude_offer_detail_structuralize_v1(json)),
        _ => None,
    }
}

pub fn sitemap_config_for(retailer: &RetailerCode) -> Option<SitemapConfig> {
    match retailer {
        RetailerCode::UgreenCa => Some(prelude_sitemap_config_v1()),
        RetailerCode::UgreenCom => Some(ugreencom_prelude_sitemap_config_v1()),
        RetailerCode::UgreenDe => Some(ugreende_prelude_sitemap_config_v1()),
        RetailerCode::UgreenEu => Some(ugreeneu_prelude_sitemap_config_v1()),
        RetailerCode::UgreenFr => Some(ugreenfr_prelude_sitemap_config_v1()),
        RetailerCode::UgreenIn => Some(ugreenin_prelude_sitemap_config_v1()),
        RetailerCode::UgreenJp => Some(ugreenjp_prelude_sitemap_config_v1()),
        RetailerCode::UgreenKr => Some(ugreenkr_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNas => Some(ugreennas_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNasAu => Some(ugreennasau_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNasCa => Some(ugreennasca_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNasDe => Some(ugreennasde_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNasEs => Some(ugreennases_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNasEu => Some(ugreennaseu_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNasFr => Some(ugreennasfr_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNasIt => Some(ugreennasit_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNasJp => Some(ugreennasjp_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNasUk => Some(ugreennasuk_prelude_sitemap_config_v1()),
        RetailerCode::UgreenNl => Some(ugreennl_prelude_sitemap_config_v1()),
        RetailerCode::UgreenUk => Some(ugreenuk_prelude_sitemap_config_v1()),
        RetailerCode::UgreenUs => Some(ugreenus_prelude_sitemap_config_v1()),
        _ => None,
    }
}

pub fn sitemap_pause_for(retailer: RetailerCode) -> Option<Duration> {
    match retailer {
        RetailerCode::UgreenCa => Some(prelude_sitemap_pause_v1()),
        RetailerCode::UgreenCom => Some(ugreencom_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenDe => Some(ugreende_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenEu => Some(ugreeneu_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenFr => Some(ugreenfr_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenIn => Some(ugreenin_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenJp => Some(ugreenjp_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenKr => Some(ugreenkr_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNas => Some(ugreennas_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNasAu => Some(ugreennasau_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNasCa => Some(ugreennasca_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNasDe => Some(ugreennasde_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNasEs => Some(ugreennases_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNasEu => Some(ugreennaseu_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNasFr => Some(ugreennasfr_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNasIt => Some(ugreennasit_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNasJp => Some(ugreennasjp_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNasUk => Some(ugreennasuk_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenNl => Some(ugreennl_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenUk => Some(ugreenuk_prelude_sitemap_pause_v1()),
        RetailerCode::UgreenUs => Some(ugreenus_prelude_sitemap_pause_v1()),
        _ => None,
    }
}

pub fn sitemap_greylisted_for(retailer: RetailerCode, url: &str) -> Option<bool> {
    match retailer {
        RetailerCode::UgreenCa => Some(prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenCom => Some(ugreencom_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenDe => Some(ugreende_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenEu => Some(ugreeneu_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenFr => Some(ugreenfr_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenIn => Some(ugreenin_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenJp => Some(ugreenjp_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenKr => Some(ugreenkr_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNas => Some(ugreennas_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasAu => Some(ugreennasau_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasCa => Some(ugreennasca_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasDe => Some(ugreennasde_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasEs => Some(ugreennases_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasEu => Some(ugreennaseu_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasFr => Some(ugreennasfr_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasIt => Some(ugreennasit_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasJp => Some(ugreennasjp_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNasUk => Some(ugreennasuk_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenNl => Some(ugreennl_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenUk => Some(ugreenuk_prelude_sitemap_greylisted_v1(url)),
        RetailerCode::UgreenUs => Some(ugreenus_prelude_sitemap_greylisted_v1(url)),
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
        RetailerCode::UgreenCa => Some(prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::UgreenCom => {
            Some(ugreencom_prelude_classify_link_v1(url, source, image_count))
        }
        RetailerCode::UgreenDe => Some(ugreende_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::UgreenEu => Some(ugreeneu_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::UgreenFr => Some(ugreenfr_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::UgreenIn => Some(ugreenin_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::UgreenJp => Some(ugreenjp_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::UgreenKr => Some(ugreenkr_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::UgreenNas => {
            Some(ugreennas_prelude_classify_link_v1(url, source, image_count))
        }
        RetailerCode::UgreenNasAu => Some(ugreennasau_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasCa => Some(ugreennasca_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasDe => Some(ugreennasde_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasEs => Some(ugreennases_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasEu => Some(ugreennaseu_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasFr => Some(ugreennasfr_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasIt => Some(ugreennasit_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasJp => Some(ugreennasjp_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNasUk => Some(ugreennasuk_prelude_classify_link_v1(
            url,
            source,
            image_count,
        )),
        RetailerCode::UgreenNl => Some(ugreennl_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::UgreenUk => Some(ugreenuk_prelude_classify_link_v1(url, source, image_count)),
        RetailerCode::UgreenUs => Some(ugreenus_prelude_classify_link_v1(url, source, image_count)),
        _ => None,
    }
}
