use super::{acquire_storefront_token, api, offer_details, product_catalog_ids};
use crate::platform::{Platform, bus::Bus, state::AppState};
use utoipa_axum::{router::OpenApiRouter, routes};
pub fn register(bus: &mut Bus, platform: &Platform) {
    super::discover_markets::register(bus, platform);
    acquire_storefront_token::prelude::register(bus, platform);
    product_catalog_ids::prelude::register(bus, platform);
    offer_details::prelude::register(bus, platform);
}
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(api::start))
}
