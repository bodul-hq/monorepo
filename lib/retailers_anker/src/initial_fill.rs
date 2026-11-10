use retailer_sauce::initial_fill::InitialFillIntegration;
use shared::retailer::RetailerCode;
pub fn integration(retailer: RetailerCode) -> Option<&'static dyn InitialFillIntegration> {
    Some(match retailer.as_str().as_str() {
        "ankercom" => &crate::ankercom::initial_fill::v1::Integration,
        "ankerde" => &crate::ankerde::initial_fill::v1::Integration,
        "ankeruk" => &crate::ankeruk::initial_fill::v1::Integration,
        "ankereu" => &crate::ankereu::initial_fill::v1::Integration,
        "ankerpl" => &crate::ankerpl::initial_fill::v1::Integration,
        "ankerfr" => &crate::ankerfr::initial_fill::v1::Integration,
        "ankernz" => &crate::ankernz::initial_fill::v1::Integration,
        "ankervn" => &crate::ankervn::initial_fill::v1::Integration,
        "ankeritalycom" => &crate::ankeritalycom::initial_fill::v1::Integration,
        "ankerjapancom" => &crate::ankerjapancom::initial_fill::v1::Integration,
        "ankerse" => &crate::ankerse::initial_fill::v1::Integration,
        "ankerno" => &crate::ankerno::initial_fill::v1::Integration,
        "ankerfi" => &crate::ankerfi::initial_fill::v1::Integration,
        "ankeris" => &crate::ankeris::initial_fill::v1::Integration,
        "ankerdk" => &crate::ankerdk::initial_fill::v1::Integration,
        "ankerau" => &crate::ankerau::initial_fill::v1::Integration,
        "ankerca" => &crate::ankerca::initial_fill::v1::Integration,
        _ => return None,
    })
}
pub fn config(
    retailer: RetailerCode,
) -> Option<retailer_sauce::shopify::initial_fill::StorefrontConfig> {
    integration(retailer).map(|i| i.config())
}
