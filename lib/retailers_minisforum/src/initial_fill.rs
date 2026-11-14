use retailer_sauce::initial_fill::InitialFillIntegration;
use shared::retailer::RetailerCode;
pub fn integration(retailer: RetailerCode) -> Option<&'static dyn InitialFillIntegration> {
    Some(match retailer.as_str().as_str() {
        "minisforumau" => &crate::minisforumau::initial_fill::v1::Integration,
        "minisforumca" => &crate::minisforumca::initial_fill::v1::Integration,
        "minisforumeu" => &crate::minisforumeu::initial_fill::v1::Integration,
        "minisforumfr" => &crate::minisforumfr::initial_fill::v1::Integration,
        "minisforumhk" => &crate::minisforumhk::initial_fill::v1::Integration,
        "minisforumjp" => &crate::minisforumjp::initial_fill::v1::Integration,
        "minisforumkr" => &crate::minisforumkr::initial_fill::v1::Integration,
        "minisforumru" => &crate::minisforumru::initial_fill::v1::Integration,
        "minisforumuk" => &crate::minisforumuk::initial_fill::v1::Integration,
        "minisforumus" => &crate::minisforumus::initial_fill::v1::Integration,
        _ => return None,
    })
}
pub fn config(
    retailer: RetailerCode,
) -> Option<retailer_sauce::shopify::initial_fill::StorefrontConfig> {
    integration(retailer).map(|i| i.config())
}
