use bodul_shared::prelude::{NormalizedData, RetailerCode};
use html_extractor::io::Architecture;

pub fn architecture_for(_retailer: RetailerCode) -> Option<Architecture> {
    None
}

pub fn normalize_for(
    _retailer: RetailerCode,
    _json: &str,
) -> Option<Result<NormalizedData, String>> {
    None
}

#[cfg(test)]
mod tests {
    // Placeholder until the crate has code, so `make test` still builds and
    // runs it. Replace it with real tests.
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailers_wip");
    }
}
