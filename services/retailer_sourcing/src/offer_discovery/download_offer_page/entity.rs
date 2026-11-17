//! A downloaded offer page, independent of its database row.

use chrono::DateTime;
use chrono::Utc;
use shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(super) struct RawOffer {
    pub scraped_at: Option<DateTime<Utc>>,
    pub offer_id: Uuid,
    pub retailer_code: RetailerCode,
    pub url: String,
    pub body: String,
}
