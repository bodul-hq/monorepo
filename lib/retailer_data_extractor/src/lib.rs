use bodul_shared::NormalizedData;
/// Retailer identifier accepted by [`architecture_for`].
use bodul_shared::prelude::RetailerCode;
use html_extractor::io::Architecture;

/// Returns the registered extraction architecture for a retailer.
///
/// Returns an I/O error with kind `NotFound` when no architecture is registered.
pub fn architecture_for(retailer_code: RetailerCode) -> Result<Architecture, std::io::Error> {
    let architecture = retailers_anker::architecture_for(retailer_code)
        .or_else(|| retailers_mi::architecture_for(retailer_code))
        .or_else(|| retailers_minisforum::architecture_for(retailer_code))
        .or_else(|| retailers_ugreen::architecture_for(retailer_code))
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
    retailers_anker::normalize_for(retailer_code, json)
        .or_else(|| retailers_mi::normalize_for(retailer_code, json))
        .or_else(|| retailers_minisforum::normalize_for(retailer_code, json))
        .or_else(|| retailers_ugreen::normalize_for(retailer_code, json))
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
