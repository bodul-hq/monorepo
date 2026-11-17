use retailer_sauce::prelude::ProductIdentifiers;

use shared::retailer::RetailerCode;

use crate::offers::retailer_offer_id::RetailerOfferId;

#[derive(Clone, Debug, PartialEq)]
pub struct RetailerOfferIdentifiers {
    pub identifiers: ProductIdentifiers,
    pub id: RetailerOfferId,
    pub retailer_code: RetailerCode,
    pub url: String,
}
