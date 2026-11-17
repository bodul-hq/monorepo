mod v2;
use localization::KEY;
use localization::attach;
use localization::extract as localization_extract;
use localization::split;
use retailer_sauce::prelude::StorefrontConfig;
use retailers_anker::prelude::classify_link_for as retailers_anker_classify_link_for;
use retailers_anker::prelude::get_storefront_config as retailers_anker_get_storefront_config;
use retailers_anker::prelude::legacy_offer_source_for;
use retailers_anker::prelude::normalize_offer_details_for as retailers_anker_normalize_offer_details_for;
use retailers_anker::prelude::offer_details_architecture_for as retailers_anker_offer_details_architecture_for;
use retailers_anker::prelude::sitemap_config_for as retailers_anker_sitemap_config_for;
use retailers_anker::prelude::sitemap_greylisted_for as retailers_anker_sitemap_greylisted_for;
use retailers_anker::prelude::sitemap_pause_for as retailers_anker_sitemap_pause_for;
use retailers_anker::prelude::structuralize_offer_details_for as retailers_anker_structuralize_offer_details_for;
use retailers_minisforum::prelude::classify_link_for as retailers_minisforum_classify_link_for;
use retailers_minisforum::prelude::get_storefront_config as retailers_minisforum_get_storefront_config;
use retailers_minisforum::prelude::normalize_offer_details_for as retailers_minisforum_normalize_offer_details_for;
use retailers_minisforum::prelude::offer_details_architecture_for as retailers_minisforum_offer_details_architecture_for;
use retailers_minisforum::prelude::sitemap_config_for as retailers_minisforum_sitemap_config_for;
use retailers_minisforum::prelude::sitemap_greylisted_for as retailers_minisforum_sitemap_greylisted_for;
use retailers_minisforum::prelude::sitemap_pause_for as retailers_minisforum_sitemap_pause_for;
use retailers_minisforum::prelude::structuralize_offer_details_for as retailers_minisforum_structuralize_offer_details_for;
use retailers_ugreen::prelude::classify_link_for as retailers_ugreen_classify_link_for;
use retailers_ugreen::prelude::get_storefront_config as retailers_ugreen_get_storefront_config;
use retailers_ugreen::prelude::normalize_offer_details_for as retailers_ugreen_normalize_offer_details_for;
use retailers_ugreen::prelude::offer_details_architecture_for as retailers_ugreen_offer_details_architecture_for;
use retailers_ugreen::prelude::offer_download_url as retailers_ugreen_offer_download_url;
use retailers_ugreen::prelude::sitemap_config_for as retailers_ugreen_sitemap_config_for;
use retailers_ugreen::prelude::sitemap_greylisted_for as retailers_ugreen_sitemap_greylisted_for;
use retailers_ugreen::prelude::sitemap_pause_for as retailers_ugreen_sitemap_pause_for;
use retailers_ugreen::prelude::structuralize_offer_details_for as retailers_ugreen_structuralize_offer_details_for;
use serde_json::to_value;
use std::io::Error;
use std::io::ErrorKind;

pub use v2::extract_v2_for;
mod initial_fill_process;
pub mod localization;
mod pretty;
#[cfg(test)]
mod retailer_inventory_tests;

use std::time::Duration;

use html_extractor::Backend;
use html_extractor::io::Architecture;
use retailer_sauce::prelude::StructuralizedData;
use shared::link::LinkKind;
/// Retailer identifier accepted by [`offer_details_architecture_for`].
use shared::prelude::RetailerCode;
use shared::{NormalizedData, SitemapConfig};

pub use initial_fill_process::{InitialFillProcess, Step, initial_fill_process};
pub use pretty::pretty_json;

/// Loads the documented storefront configuration for a known retailer code.
pub fn get_storefront_config(code: RetailerCode) -> Option<StorefrontConfig> {
    let code = code.as_str();
    retailers_anker_get_storefront_config(&code)
        .or_else(|| retailers_minisforum_get_storefront_config(&code))
        .or_else(|| retailers_ugreen_get_storefront_config(&code))
}

/// Returns the HTML extraction architecture for a retailer's offer details page:
/// the selectors [`extract_offer_details_for`] runs over a downloaded page.
///
/// Returns an I/O error with kind `NotFound` when no architecture is registered.
pub fn offer_details_architecture_for(retailer_code: RetailerCode) -> Result<Architecture, Error> {
    let architecture = retailers_anker_offer_details_architecture_for(retailer_code)
        .or_else(|| retailers_minisforum_offer_details_architecture_for(retailer_code))
        .or_else(|| retailers_ugreen_offer_details_architecture_for(retailer_code))
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!("no architecture for retailer {retailer_code:?}"),
            )
        })?;

    Ok(architecture)
}

/// Extracts a retailer's offer page into JSON text with its registered architecture.
///
/// Returns an error when no architecture is registered or the page can't be parsed.
pub fn extract_offer_details_for(
    retailer_code: RetailerCode,
    html: &str,
) -> Result<String, String> {
    if let Some(market) = legacy_offer_source_for(retailer_code) {
        return market.extract(html);
    }
    let architecture = offer_details_architecture_for(retailer_code).map_err(|e| e.to_string())?;
    let extracted = Backend::Scraper
        .extract(html, &architecture)
        .map_err(|e| e.to_string())?;

    let mut value = extracted.json;
    value[KEY] = to_value(localization_extract(html)).map_err(|error| error.to_string())?;
    Ok(value.to_string())
}

pub fn normalize_offer_details_for(
    retailer_code: RetailerCode,
    json: &str,
) -> Option<Result<NormalizedData, String>> {
    if let Some(market) = legacy_offer_source_for(retailer_code) {
        return Some(market.normalize(json));
    }
    if matches!(
        retailer_code,
        RetailerCode::MiCom | RetailerCode::AnkerNordicsCom
    ) {
        return None;
    }
    let (json, evidence) = match split(json) {
        Ok(value) => value,
        Err(error) => return Some(Err(error)),
    };
    let json = json.as_str();
    retailers_anker_normalize_offer_details_for(retailer_code, json)
        .or_else(|| retailers_minisforum_normalize_offer_details_for(retailer_code, json))
        .or_else(|| retailers_ugreen_normalize_offer_details_for(retailer_code, json))
        .map(|result| {
            result.and_then(|mut data| {
                attach(&mut data.document_json, evidence, false)?;
                Ok(data)
            })
        })
}

/// Returns `None` when the retailer has no structuralizer registered by its family crate.
pub fn structuralize_offer_details_for(
    retailer_code: RetailerCode,
    json: &str,
) -> Option<Result<StructuralizedData, String>> {
    if let Some(market) = legacy_offer_source_for(retailer_code) {
        return Some(market.structuralize(json));
    }
    if matches!(
        retailer_code,
        RetailerCode::MiCom | RetailerCode::AnkerNordicsCom
    ) {
        return None;
    }
    let (json, evidence) = match split(json) {
        Ok(value) => value,
        Err(error) => return Some(Err(error)),
    };
    let json = json.as_str();
    retailers_anker_structuralize_offer_details_for(retailer_code, json)
        .or_else(|| retailers_minisforum_structuralize_offer_details_for(retailer_code, json))
        .or_else(|| retailers_ugreen_structuralize_offer_details_for(retailer_code, json))
        .map(|result| {
            result.and_then(|mut data| {
                attach(&mut data.document_json, evidence, true)?;
                Ok(data)
            })
        })
}

pub fn sitemap_config_for(retailer_code: &RetailerCode) -> Option<SitemapConfig> {
    retailers_anker_sitemap_config_for(retailer_code)
        .or_else(|| retailers_minisforum_sitemap_config_for(retailer_code))
        .or_else(|| retailers_ugreen_sitemap_config_for(retailer_code))
}

/// Whether `retailer_code`'s crawl downloads the child sitemap at `url`.
/// Returns `None` for a retailer this crate doesn't know.
pub fn sitemap_greylisted_for(retailer_code: RetailerCode, url: &str) -> Option<bool> {
    retailers_anker_sitemap_greylisted_for(retailer_code, url)
        .or_else(|| retailers_minisforum_sitemap_greylisted_for(retailer_code, url))
        .or_else(|| retailers_ugreen_sitemap_greylisted_for(retailer_code, url))
}

/// How long `retailer_code`'s crawl pauses between groups of sitemap
/// downloads. Returns `None` for a retailer this crate doesn't know.
pub fn sitemap_pause_for(retailer_code: RetailerCode) -> Option<Duration> {
    retailers_anker_sitemap_pause_for(retailer_code)
        .or_else(|| retailers_minisforum_sitemap_pause_for(retailer_code))
        .or_else(|| retailers_ugreen_sitemap_pause_for(retailer_code))
}

pub fn classify_link_for(
    retailer_code: RetailerCode,
    url: &str,
    source: &str,
    image_count: usize,
) -> Option<LinkKind> {
    retailers_anker_classify_link_for(retailer_code, url, source, image_count)
        .or_else(|| retailers_minisforum_classify_link_for(retailer_code, url, source, image_count))
        .or_else(|| retailers_ugreen_classify_link_for(retailer_code, url, source, image_count))
}

/// Resolve explicitly known retired product URLs without changing discovery identity.
pub fn offer_download_url(retailer_code: RetailerCode, url: &str) -> &str {
    retailers_ugreen_offer_download_url(retailer_code, url)
}

fn reject_retired_retailer(retailer: RetailerCode) -> Result<(), String> {
    if retailer == RetailerCode::AnkerNordicsCom {
        return Err("retailer ankernordicscom no longer supports offer processing".into());
    }
    Ok(())
}

/// A retailer's source request. A body means a JSON POST; otherwise use GET.
/// Nordic markets use the public Storefront API while retaining the product URL
/// as the discovery identity and storing the response verbatim for replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferRequest {
    pub url: String,
    pub json_body: Option<String>,
}

pub fn offer_request_for(retailer: RetailerCode, url: &str) -> Result<OfferRequest, String> {
    reject_retired_retailer(retailer)?;
    if let Some(market) = legacy_offer_source_for(retailer) {
        return Ok(OfferRequest {
            url: market.endpoint().into(),
            json_body: Some(market.request(url)?),
        });
    }
    Ok(OfferRequest {
        url: offer_download_url(retailer, url).into(),
        json_body: None,
    })
}

/// Reject partial, wrong-market and wrong-product responses before storage so
/// download retries fetch new source data rather than repeatedly parsing errors.
pub fn validate_offer_download(
    retailer: RetailerCode,
    url: &str,
    body: &str,
) -> Result<(), String> {
    reject_retired_retailer(retailer)?;
    if let Some(market) = legacy_offer_source_for(retailer) {
        market.validate_download(url, body)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use localization::split as localization_split;
    use serde_json::Value;
    use serde_json::from_str;
    use serde_json::from_value;
    use serde_json::json as serde_json_json;
    use serde_json::to_string as serde_json_to_string;
    use std::io::ErrorKind as StdIoErrorKind;

    use super::*;
    use html_extractor::serde_json;

    #[test]
    fn nordic_country_hooks_and_source_requests_are_reachable() {
        for (code, country, fixture) in [
            (
                RetailerCode::AnkerSe,
                "SE",
                include_str!("../tests/fixtures/anker/ankerse_response.json"),
            ),
            (
                RetailerCode::AnkerNo,
                "NO",
                include_str!("../tests/fixtures/anker/ankerno_response.json"),
            ),
            (
                RetailerCode::AnkerFi,
                "FI",
                include_str!("../tests/fixtures/anker/ankerfi_response.json"),
            ),
            (
                RetailerCode::AnkerIs,
                "IS",
                include_str!("../tests/fixtures/anker/ankeris_response.json"),
            ),
            (
                RetailerCode::AnkerDk,
                "DK",
                include_str!("../tests/fixtures/anker/ankerdk_response.json"),
            ),
        ] {
            let url = "https://www.ankernordics.com/products/t90r0121";
            assert!(offer_details_architecture_for(code).is_ok());
            assert_eq!(
                sitemap_config_for(&code).unwrap().sitemap_url,
                ["https://www.ankernordics.com/sitemap.xml"]
            );
            assert_eq!(
                sitemap_greylisted_for(code, "https://www.ankernordics.com/sitemap-0.xml"),
                Some(true)
            );
            assert_eq!(sitemap_pause_for(code), Some(Duration::from_secs(1)));
            assert_eq!(
                classify_link_for(code, url, "main", 0),
                Some(LinkKind::Product)
            );
            let request = offer_request_for(code, url).unwrap();
            let request: Value = from_str(request.json_body.as_deref().unwrap()).unwrap();
            assert_eq!(request["variables"]["country"], country);
            validate_offer_download(code, url, fixture).unwrap();
            let extracted = extract_offer_details_for(code, fixture).unwrap();
            let normalized = normalize_offer_details_for(code, &extracted)
                .unwrap()
                .unwrap()
                .document_json
                .unwrap();
            let structuralized = structuralize_offer_details_for(code, &normalized)
                .unwrap()
                .unwrap()
                .document_json
                .unwrap();
            let value: Value = from_str(&structuralized).unwrap();
            assert_eq!(
                value["variations"][0]["localization"]["target_market"]["code"],
                country
            );
            assert_eq!(RetailerCode::from_str(&code.as_str()).unwrap(), code);
            assert_eq!(
                from_str::<RetailerCode>(&serde_json_to_string(&code).unwrap()).unwrap(),
                code
            );
        }
        assert_eq!(
            RetailerCode::from_str("ankernordicscom").unwrap(),
            RetailerCode::AnkerNordicsCom
        );
        assert!(!RetailerCode::ALL.contains(&RetailerCode::AnkerNordicsCom));
        assert!(offer_details_architecture_for(RetailerCode::AnkerNordicsCom).is_err());
    }

    #[test]
    fn retired_aggregate_remains_readable_without_processing_or_download_fallback() {
        let code = RetailerCode::AnkerNordicsCom;
        for spelling in ["ankernordicscom", "AnkerNordicsCom"] {
            assert_eq!(
                from_value::<RetailerCode>(serde_json_json!(spelling)).unwrap(),
                code
            );
        }
        let url = "https://www.ankernordics.com/products/t90r0121";
        assert_eq!(
            offer_details_architecture_for(code).unwrap_err().kind(),
            StdIoErrorKind::NotFound
        );
        assert!(extract_offer_details_for(code, "<html></html>").is_err());
        assert!(normalize_offer_details_for(code, "not json").is_none());
        assert!(structuralize_offer_details_for(code, "not json").is_none());
        assert!(sitemap_config_for(&code).is_none());
        assert!(sitemap_pause_for(code).is_none());
        assert!(sitemap_greylisted_for(code, url).is_none());
        assert!(classify_link_for(code, url, "main", 0).is_none());
        assert!(offer_request_for(code, url).is_err());
        assert!(validate_offer_download(code, url, "{}").is_err());
        assert!(get_storefront_config(code).is_none());
    }

    #[test]
    fn extracts_a_page_with_the_retailers_architecture() {
        let html = r#"<html><head>
            <script type="application/ld+json">{"@type":"Product"}</script>
            <script id="__NEXT_DATA__" type="application/json">{"page":"/p"}</script>
        </head><body></body></html>"#;

        let json = extract_offer_details_for(RetailerCode::AnkerAu, html).unwrap();

        let (json, _) = localization_split(&json).unwrap();
        assert_eq!(
            from_str::<Value>(&json).unwrap(),
            from_str::<Value>(r#"{"ld-json":[{"data":{"@type":"Product"},"type":"application/ld+json"}],"next_data":{"data":{"page":"/p"},"id":"__NEXT_DATA__","type":"application/json"}}"#).unwrap()
        );
    }

    #[test]
    fn the_new_ugreen_offer_pipelines_are_reachable() {
        for code in [
            RetailerCode::UgreenKr,
            RetailerCode::UgreenNl,
            RetailerCode::UgreenUk,
            RetailerCode::UgreenUs,
        ] {
            let extracted = extract_offer_details_for(code, "<html><body></body></html>")
                .expect("architecture is wired");
            let value: Value = from_str(&extracted).unwrap();
            assert_eq!(value["ld-json"], serde_json_json!([]), "{code:?}");
            // An incomplete document must report an error through the facade, not panic
            // in an unimplemented storefront hook.
            assert!(
                normalize_offer_details_for(code, &extracted)
                    .expect("normalizer is wired")
                    .is_err(),
                "{code:?}"
            );
        }
    }

    #[test]
    fn every_retailer_with_sitemaps_downloads_every_child_by_default() {
        for retailer_code in [
            RetailerCode::AnkerAu,
            RetailerCode::MinisForumAu,
            RetailerCode::UgreenCa,
        ] {
            let greylisted = sitemap_greylisted_for(retailer_code, "https://shop.test/sitemap.xml");
            assert_eq!(greylisted, Some(true), "{retailer_code:?}");
        }
    }

    #[test]
    fn pauses_longer_for_stores_that_rate_limit() {
        for retailer_code in [
            RetailerCode::UgreenCom,
            RetailerCode::UgreenEu,
            RetailerCode::UgreenNasEu,
        ] {
            assert_eq!(
                sitemap_pause_for(retailer_code),
                Some(Duration::from_secs(5)),
                "{retailer_code:?}"
            );
        }
        for retailer_code in [
            RetailerCode::AnkerAu,
            RetailerCode::MinisForumAu,
            RetailerCode::UgreenCa,
        ] {
            assert_eq!(
                sitemap_pause_for(retailer_code),
                Some(Duration::from_secs(1)),
                "{retailer_code:?}"
            );
        }
    }

    #[test]
    fn a_retailer_without_a_structuralizer_has_none() {
        assert!(structuralize_offer_details_for(RetailerCode::MiCom, "{}").is_none());
    }

    #[test]
    fn structuralizing_reports_a_document_it_cannot_read() {
        for retailer_code in [
            RetailerCode::AnkerAu,
            RetailerCode::MinisForumAu,
            RetailerCode::MinisForumEu,
            RetailerCode::MinisForumFr,
            RetailerCode::MinisForumHk,
            RetailerCode::MinisForumJp,
            RetailerCode::UgreenCa,
            RetailerCode::UgreenCom,
            RetailerCode::UgreenJp,
            RetailerCode::UgreenNas,
            RetailerCode::UgreenNasEu,
            RetailerCode::UgreenNasFr,
            RetailerCode::UgreenNasIt,
            RetailerCode::UgreenNasUk,
            RetailerCode::UgreenEu,
            RetailerCode::UgreenFr,
            RetailerCode::UgreenIn,
            RetailerCode::UgreenNasAu,
            RetailerCode::UgreenNasCa,
            RetailerCode::UgreenNasDe,
            RetailerCode::UgreenNasEs,
        ] {
            let result = structuralize_offer_details_for(retailer_code, "{}")
                .unwrap_or_else(|| panic!("{retailer_code:?} has a structuralizer"));

            assert!(result.is_err(), "{retailer_code:?}");
        }
    }
}

#[cfg(test)]
mod migrated_wip_tests {
    use crate::v2::assert_extracted_contract;
    use serde_json::Value as SerdeJsonValue;
    use serde_json::from_str as serde_json_from_str;
    use serde_json::json as serde_json_json2;

    use super::*;

    #[test]
    fn experimental_ankerau_hooks_are_connected() {
        let code = RetailerCode::AnkerAu;
        assert!(get_storefront_config(code).is_some());
        assert!(offer_details_architecture_for(code).is_ok());
        assert!(sitemap_config_for(&code).is_some());
        assert!(normalize_offer_details_for(code, "{}").is_some());
        assert!(structuralize_offer_details_for(code, "{}").is_some());
        assert!(!initial_fill_process(code).steps.is_empty());
        let product: SerdeJsonValue =
            serde_json_from_str(include_str!("../tests/fixtures/v2/ankerau.json")).unwrap();
        let artifact = serde_json_json2!({"schema_version":2,"context":{"country":"CA","language":"FR","currency":"CAD"},"acquired_at":"2026-11-10T00:00:00Z","product":product,"pages":[]});
        assert!(extract_v2_for(code, &artifact.to_string()).is_ok());
    }

    #[test]
    fn experimental_ankerca_hooks_are_connected() {
        let code = RetailerCode::AnkerCa;
        let config = get_storefront_config(code).unwrap();
        assert_eq!(config.default_language(), "EN");
        assert_eq!(
            initial_fill_process(code).steps,
            vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails
            ]
        );
        assert!(offer_details_architecture_for(code).is_ok());
        assert!(sitemap_config_for(&code).is_some());
        assert!(sitemap_pause_for(code).is_some());
        assert!(sitemap_greylisted_for(code, "https://www.anker.com/ca/products/test").is_some());
        assert_eq!(
            classify_link_for(
                code,
                "https://www.anker.com/ca/products/test",
                "https://www.anker.com/ca/server-sitemap-index-products.xml",
                0
            ),
            Some(LinkKind::Product)
        );
        assert!(normalize_offer_details_for(code, "{}").unwrap().is_err());
        assert!(
            structuralize_offer_details_for(code, "{}")
                .unwrap()
                .is_err()
        );
        let product: SerdeJsonValue =
            serde_json_from_str(include_str!("../tests_fixtures/ankerca_product.json")).unwrap();
        let source = serde_json_json2!({"schema_version":2,"context":{"country":"CA","language":"EN","currency":"CAD"},"acquired_at":"2026-11-11T00:00:00Z","product":product,"pages":[{"request":{"query":"query Test { product { id } }","operationName":"Test","variables":{}},"response":{"extensions":{"context":{"country":"CA","language":"EN"}}}}]});
        let result: SerdeJsonValue =
            serde_json_from_str(&extract_v2_for(code, &source.to_string()).unwrap()).unwrap();
        assert_extracted_contract(&result, &source);
    }
}
