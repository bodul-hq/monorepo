use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use money::Money;

use super::retailer_offer_id::RetailerOfferId;

#[derive(Debug, PartialEq)]
pub struct RetailerOfferState {
    id: RetailerOfferId,
    retailer_code: RetailerCode,
    title: String,
    price: Money,
    url: String,
    created_at: DateTime<Utc>,
}

impl RetailerOfferState {
    pub(super) fn change_title(&mut self, title: String) {
        self.title = title;
    }

    pub(super) fn change_price(&mut self, price: Money) {
        self.price = price;
    }

    pub fn new(
        id: RetailerOfferId,
        retailer_code: RetailerCode,
        title: impl Into<String>,
        price: Money,
        url: impl Into<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            retailer_code,
            title: title.into(),
            price,
            url: url.into(),
            created_at,
        }
    }

    pub fn id(&self) -> &RetailerOfferId {
        &self.id
    }

    pub fn retailer_code(&self) -> &RetailerCode {
        &self.retailer_code
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn price(&self) -> &Money {
        &self.price
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}
