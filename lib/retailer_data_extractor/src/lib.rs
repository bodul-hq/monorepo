use bodul_shared::link::LinkKind;
/// Retailer identifier accepted by [`architecture_for`].
use bodul_shared::prelude::RetailerCode;
use bodul_shared::{NormalizedData, SitemapConfig};
use html_extractor::io::Architecture;

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

pub fn normalize_for(
    retailer_code: RetailerCode,
    json: &str,
) -> Option<Result<NormalizedData, String>> {
    retailers_anker::prelude::normalize_for(retailer_code, json)
        .or_else(|| retailers_mi::prelude::normalize_for(retailer_code, json))
        .or_else(|| retailers_minisforum::prelude::normalize_for(retailer_code, json))
        .or_else(|| retailers_ugreen::prelude::normalize_for(retailer_code, json))
}

pub fn sitemap_config_for(retailer_code: &RetailerCode) -> Option<SitemapConfig> {
    retailers_anker::prelude::sitemap_config_for(retailer_code)
        .or_else(|| retailers_mi::prelude::sitemap_config_for(retailer_code))
        .or_else(|| retailers_minisforum::prelude::sitemap_config_for(retailer_code))
        .or_else(|| retailers_ugreen::prelude::sitemap_config_for(retailer_code))
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
    // Placeholder until the crate has code, so `make test` still builds and
    // runs it. Replace it with real tests.
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailer_data_extractor");
    }
}
