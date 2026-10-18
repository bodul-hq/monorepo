mod pretty;

use std::time::Duration;

use bodul_shared::link::LinkKind;
/// Retailer identifier accepted by [`architecture_for`].
use bodul_shared::prelude::RetailerCode;
use bodul_shared::{NormalizedData, SitemapConfig};
use html_extractor::Backend;
use html_extractor::io::Architecture;
use retailer_sauce::structuralized::StructuralizedData;

pub use pretty::pretty_json;

/// Returns the registered extraction architecture for a retailer.
///
/// Returns an I/O error with kind `NotFound` when no architecture is registered.
pub fn architecture_for(retailer_code: RetailerCode) -> Result<Architecture, std::io::Error> {
    let architecture = retailers_anker::prelude::architecture_for(retailer_code)
        .or_else(|| retailers_mi::prelude::architecture_for(retailer_code))
        .or_else(|| retailers_minisforum::prelude::architecture_for(retailer_code))
        .or_else(|| retailers_ugreen::prelude::architecture_for(retailer_code))
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no architecture for retailer {retailer_code:?}"),
            )
        })?;

    Ok(architecture)
}

/// Extracts a retailer's offer page into JSON text with its registered architecture.
///
/// Returns an error when no architecture is registered or the page can't be parsed.
pub fn extract_for(retailer_code: RetailerCode, html: &str) -> Result<String, String> {
    let architecture = architecture_for(retailer_code).map_err(|e| e.to_string())?;
    let extracted = Backend::Scraper
        .extract(html, &architecture)
        .map_err(|e| e.to_string())?;

    Ok(extracted.json.to_string())
}

pub fn normalize_for(
    retailer_code: RetailerCode,
    json: &str,
) -> Option<Result<NormalizedData, String>> {
    retailers_anker::prelude::normalize_for(retailer_code, json)
        .or_else(|| retailers_mi::prelude::normalize_for(retailer_code, json))
        .or_else(|| retailers_minisforum::prelude::normalize_for(retailer_code, json))
        .or_else(|| retailers_ugreen::prelude::normalize_for(retailer_code, json))
}

/// Returns `None` when the retailer has no structuralizer registered by its family crate.
pub fn structuralize_for(
    retailer_code: RetailerCode,
    json: &str,
) -> Option<Result<StructuralizedData, String>> {
    retailers_anker::prelude::structuralize_for(retailer_code, json)
        .or_else(|| retailers_minisforum::prelude::structuralize_for(retailer_code, json))
        .or_else(|| retailers_ugreen::prelude::structuralize_for(retailer_code, json))
}

pub fn sitemap_config_for(retailer_code: &RetailerCode) -> Option<SitemapConfig> {
    retailers_anker::prelude::sitemap_config_for(retailer_code)
        .or_else(|| retailers_mi::prelude::sitemap_config_for(retailer_code))
        .or_else(|| retailers_minisforum::prelude::sitemap_config_for(retailer_code))
        .or_else(|| retailers_ugreen::prelude::sitemap_config_for(retailer_code))
}

/// Whether `retailer_code`'s crawl downloads the child sitemap at `url`.
/// Returns `None` for a retailer this crate doesn't know.
pub fn sitemap_greylisted_for(retailer_code: RetailerCode, url: &str) -> Option<bool> {
    retailers_anker::prelude::sitemap_greylisted_for(retailer_code, url)
        .or_else(|| retailers_mi::prelude::sitemap_greylisted_for(retailer_code, url))
        .or_else(|| retailers_minisforum::prelude::sitemap_greylisted_for(retailer_code, url))
        .or_else(|| retailers_ugreen::prelude::sitemap_greylisted_for(retailer_code, url))
}

/// How long `retailer_code`'s crawl pauses between groups of sitemap
/// downloads. Returns `None` for a retailer this crate doesn't know.
pub fn sitemap_pause_for(retailer_code: RetailerCode) -> Option<Duration> {
    retailers_anker::prelude::sitemap_pause_for(retailer_code)
        .or_else(|| retailers_mi::prelude::sitemap_pause_for(retailer_code))
        .or_else(|| retailers_minisforum::prelude::sitemap_pause_for(retailer_code))
        .or_else(|| retailers_ugreen::prelude::sitemap_pause_for(retailer_code))
}

pub fn classify_link_for(
    retailer_code: RetailerCode,
    url: &str,
    source: &str,
    image_count: usize,
) -> Option<LinkKind> {
    retailers_anker::prelude::classify_link_for(retailer_code, url, source, image_count)
        .or_else(|| {
            retailers_mi::prelude::classify_link_for(retailer_code, url, source, image_count)
        })
        .or_else(|| {
            retailers_minisforum::prelude::classify_link_for(
                retailer_code,
                url,
                source,
                image_count,
            )
        })
        .or_else(|| {
            retailers_ugreen::prelude::classify_link_for(retailer_code, url, source, image_count)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use html_extractor::serde_json;

    #[test]
    fn extracts_a_page_with_the_retailers_architecture() {
        let html = r#"<html><head>
            <script type="application/ld+json">{"@type":"Product"}</script>
            <script id="__NEXT_DATA__" type="application/json">{"page":"/p"}</script>
        </head><body></body></html>"#;

        let json = extract_for(RetailerCode::AnkerAu, html).unwrap();

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json).unwrap(),
            serde_json::from_str::<serde_json::Value>(r#"{"ld-json":[{"data":{"@type":"Product"},"type":"application/ld+json"}],"next_data":{"data":{"page":"/p"},"id":"__NEXT_DATA__","type":"application/json"}}"#).unwrap()
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
            let extracted =
                extract_for(code, "<html><body></body></html>").expect("architecture is wired");
            let value: serde_json::Value = serde_json::from_str(&extracted).unwrap();
            assert_eq!(value["ld-json"], serde_json::json!([]), "{code:?}");
            // An incomplete document must report an error through the facade, not panic
            // in an unimplemented storefront hook.
            assert!(
                normalize_for(code, &extracted)
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
            RetailerCode::MiCom,
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
            RetailerCode::MiCom,
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
        assert!(structuralize_for(RetailerCode::MiCom, "{}").is_none());
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
            let result = structuralize_for(retailer_code, "{}")
                .unwrap_or_else(|| panic!("{retailer_code:?} has a structuralizer"));

            assert!(result.is_err(), "{retailer_code:?}");
        }
    }
}
