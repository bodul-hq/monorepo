//! A downloaded offer page, independent of its database row.
use bodul_shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(super) struct RawOffer {
    pub offer_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
    pub body: String,
}
