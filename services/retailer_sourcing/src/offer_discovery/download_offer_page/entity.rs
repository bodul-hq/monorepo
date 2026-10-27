//! A downloaded offer page, independent of its database row.
use bodul_shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(super) struct RawOffer {
    pub scraped_at: Option<chrono::DateTime<chrono::Utc>>,
    pub offer_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
    pub body: String,
}
