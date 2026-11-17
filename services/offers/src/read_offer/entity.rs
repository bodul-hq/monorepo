use crate::offers::event::OfferFacts;
use retailer_sauce::prelude::Localization;
use retailer_sauce::prelude::ProductIdentifiers;

use chrono::{DateTime, Utc};
use money::Money;
use shared::retailer::RetailerCode;

use crate::offers::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Debug, PartialEq)]
pub struct RetailerOffer {
    pub facts: OfferFacts,
    pub identifiers: ProductIdentifiers,
    pub localization: Localization,
    pub scraped_at: Option<DateTime<Utc>>,
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
    pub images: Vec<String>,
    pub created_at: DateTime<Utc>,
}
