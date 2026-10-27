use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use money::Money;

use crate::offers::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Debug, PartialEq)]
pub struct RetailerOffer {
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
    pub images: Vec<String>,
    pub created_at: DateTime<Utc>,
}
