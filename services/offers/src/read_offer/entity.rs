use crate::offers::event::OfferFacts;
use chrono::DateTime as ChronoDateTime;
use chrono::Utc as ChronoUtc;
use chrono::{DateTime, Utc};
use money::Money;
use retailer_sauce::identifier::ProductIdentifiers;
use retailer_sauce::localization::Localization;
use shared::retailer::RetailerCode;

use crate::offers::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Debug, PartialEq)]
pub struct RetailerOffer {
    pub facts: OfferFacts,
    pub identifiers: ProductIdentifiers,
    pub localization: Localization,
    pub scraped_at: Option<ChronoDateTime<ChronoUtc>>,
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
    pub images: Vec<String>,
    pub created_at: DateTime<Utc>,
}
