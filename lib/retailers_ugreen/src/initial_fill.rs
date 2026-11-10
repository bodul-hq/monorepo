use retailer_sauce::initial_fill::InitialFillIntegration;
use shared::retailer::RetailerCode;
pub fn integration(retailer: RetailerCode) -> Option<&'static dyn InitialFillIntegration> {
    Some(match retailer.as_str().as_str() {
        "ugreenca" => &crate::ugreenca::initial_fill::v1::Integration,
        "ugreencom" => &crate::ugreencom::initial_fill::v1::Integration,
        "ugreende" => &crate::ugreende::initial_fill::v1::Integration,
        "ugreeneu" => &crate::ugreeneu::initial_fill::v1::Integration,
        "ugreenfr" => &crate::ugreenfr::initial_fill::v1::Integration,
        "ugreenin" => &crate::ugreenin::initial_fill::v1::Integration,
        "ugreenjp" => &crate::ugreenjp::initial_fill::v1::Integration,
        "ugreenkr" => &crate::ugreenkr::initial_fill::v1::Integration,
        "ugreennas" => &crate::ugreennas::initial_fill::v1::Integration,
        "ugreennasau" => &crate::ugreennasau::initial_fill::v1::Integration,
        "ugreennasca" => &crate::ugreennasca::initial_fill::v1::Integration,
        "ugreennasde" => &crate::ugreennasde::initial_fill::v1::Integration,
        "ugreennases" => &crate::ugreennases::initial_fill::v1::Integration,
        "ugreennaseu" => &crate::ugreennaseu::initial_fill::v1::Integration,
        "ugreennasfr" => &crate::ugreennasfr::initial_fill::v1::Integration,
        "ugreennasit" => &crate::ugreennasit::initial_fill::v1::Integration,
        "ugreennasjp" => &crate::ugreennasjp::initial_fill::v1::Integration,
        "ugreennasuk" => &crate::ugreennasuk::initial_fill::v1::Integration,
        "ugreennl" => &crate::ugreennl::initial_fill::v1::Integration,
        "ugreenuk" => &crate::ugreenuk::initial_fill::v1::Integration,
        "ugreenus" => &crate::ugreenus::initial_fill::v1::Integration,
        _ => return None,
    })
}
pub fn config(
    retailer: RetailerCode,
) -> Option<retailer_sauce::shopify::initial_fill::StorefrontConfig> {
    integration(retailer).map(|i| i.config())
}
