//! Retired product URLs verified against the regional storefronts on 2026-10-26.
use bodul_shared::retailer::RetailerCode;

/// Resolve known retired handles for downloading, including commands already queued.
/// All other URLs retain normal HTTP error/retry handling.
pub fn offer_download_url(retailer: RetailerCode, url: &str) -> &str {
    match (retailer, url) {
        (RetailerCode::UgreenNasDe, "https://ai-de.ugreen.com/products/dxp4800-pro") => {
            "https://ai-de.ugreen.com/products/ugreen-nasync-dxp4800-pro-4-bay-nas"
        }
        (RetailerCode::UgreenNasUk, "https://ai-uk.ugreen.com/products/dxp4800-pro") => {
            "https://ai-uk.ugreen.com/products/ugreen-nasync-dxp4800-pro-4-bay-nas"
        }
        _ => url,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_handles_resolve_only_in_their_own_storefront() {
        for (retailer, region) in [
            (RetailerCode::UgreenNasDe, "de"),
            (RetailerCode::UgreenNasUk, "uk"),
        ] {
            let old = format!("https://ai-{region}.ugreen.com/products/dxp4800-pro");
            let current = format!(
                "https://ai-{region}.ugreen.com/products/ugreen-nasync-dxp4800-pro-4-bay-nas"
            );
            assert_eq!(offer_download_url(retailer, &old), current);
            assert_eq!(offer_download_url(retailer, &current), current);
            assert_eq!(offer_download_url(RetailerCode::UgreenIn, &old), old);
            for url in [
                "https://shop.example/products/dxp4800-pro",
                "https://ai-de.ugreen.com/products/other",
            ] {
                assert_eq!(offer_download_url(retailer, url), url);
            }
        }
    }
}
