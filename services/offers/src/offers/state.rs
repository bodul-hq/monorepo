use bodul_shared::retailer::RetailerCode;
use chrono::{DateTime, Utc};
use money::Money;

use super::retailer_offer_id::RetailerOfferId;

#[derive(Debug, PartialEq)]
pub struct RetailerOfferState {
    pub(super) localization: retailer_sauce::localization::Localization,
    id: RetailerOfferId,
    retailer_code: RetailerCode,
    title: String,
    price: Money,
    url: String,
    images: Vec<String>,
    created_at: DateTime<Utc>,
    scraped_at: Option<DateTime<Utc>>,
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
            localization: Default::default(),
            id,
            retailer_code,
            title: title.into(),
            price,
            url: url.into(),
            images: Vec::new(),
            scraped_at: None,
            created_at,
        }
    }

    pub fn scraped_at(&self) -> Option<DateTime<Utc>> {
        self.scraped_at
    }

    pub(super) fn observe_scrape(&mut self, time: Option<DateTime<Utc>>) {
        self.scraped_at = self.scraped_at.max(time);
    }

    pub fn images(&self) -> &[String] {
        &self.images
    }

    pub(super) fn add_image(&mut self, image: String) {
        self.images.push(image);
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
