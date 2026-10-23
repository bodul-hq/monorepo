use bodul_shared::retailer::RetailerCode;

use crate::offers::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Debug, PartialEq)]
pub struct RetailerOfferIdentifiers {
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub url: String,
}
