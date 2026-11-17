//! Server-only client for the customer-facing browse contract.

use facts::LocalizedContent;
use facts::guard_content;
use facts::normalize_content;
use facts::read_availability as facts_read_availability;
use facts::read_condition as facts_read_condition;
use io::Error as IoError;
use io::ErrorKind;
use io::Result as IoResult;
use money::Currency;
use reqwest::StatusCode;
use reqwest::redirect::Policy;
use reqwest::retry::never;
use reqwest::{Client, Url};
use retailer_sauce::availability::Availability;
use retailer_sauce::identifier::ProductIdentifiers;
use retailer_sauce::item_condition::ItemCondition;
use retailer_sauce::localization::Localization;
use serde::Deserialize;
use serde::Deserializer;
use serde::de::Error;
use serde_json::Value;
use serde_json::from_value as decode_json_value;
use std::env::var;
use std::{env::VarError, io, time::Duration};

mod facts;

pub const PAGE_SIZE: i32 = 30;

#[derive(Debug, Deserialize)]
pub struct Page {
    pub items: Vec<Offer>,
    pub limit: i32,
    pub offset: i32,
    pub has_more: bool,
}

#[derive(Debug)]
pub struct Offer {
    pub was_price: Option<Price>,
    pub availability: Option<Availability>,
    pub item_condition: Option<ItemCondition>,
    pub content: Vec<LocalizedContent>,
    pub identifiers: ProductIdentifiers,
    pub localization: Localization,
    pub images: Vec<String>,
    pub id: String,
    pub retailer_code: String,
    pub title: String,
    pub price: Price,
    pub url: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct OfferWire {
    #[serde(default)]
    pub was_price: Option<Price>,
    #[serde(default, deserialize_with = "facts_read_availability")]
    pub availability: Option<Availability>,
    #[serde(default, deserialize_with = "facts_read_condition")]
    pub item_condition: Option<ItemCondition>,
    pub content: Vec<LocalizedContent>,
    #[serde(flatten)]
    pub identifiers: ProductIdentifiers,
    #[serde(default, deserialize_with = "read_localization")]
    pub localization: Localization,
    #[serde(default)]
    pub images: Vec<String>,
    pub id: String,
    pub retailer_code: String,
    pub title: String,
    pub price: Price,
    pub url: String,
    pub created_at: String,
}

impl<'de> Deserialize<'de> for Offer {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut value = Value::deserialize(deserializer)?;
        normalize_content(&mut value).map_err(D::Error::custom)?;
        let wire: OfferWire = decode_json_value(value).map_err(D::Error::custom)?;
        guard_content(&wire.content, &wire.localization.language).map_err(D::Error::custom)?;
        let selected = wire
            .content
            .iter()
            .find(|entry| entry.language == wire.localization.language)
            .ok_or_else(|| D::Error::custom("selected content is missing"))?;
        if selected.title != wire.title || selected.url != wire.url {
            return Err(D::Error::custom(
                "selected content title and URL must match flat fields",
            ));
        }
        Ok(Self {
            was_price: wire.was_price,
            availability: wire.availability,
            item_condition: wire.item_condition,
            content: wire.content,
            identifiers: wire.identifiers,
            localization: wire.localization,
            images: wire.images,
            id: wire.id,
            retailer_code: wire.retailer_code,
            title: wire.title,
            price: wire.price,
            url: wire.url,
            created_at: wire.created_at,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Price {
    pub amount: String,
    pub currency: Currency,
}

impl Price {
    fn amount(&self) -> Option<i64> {
        self.amount
            .parse::<i64>()
            .ok()
            .filter(|amount| amount.to_string() == self.amount)
    }
}

impl Offer {
    pub fn comparable_was_price(&self) -> Option<&Price> {
        self.was_price.as_ref().filter(|was| {
            was.currency == self.price.currency
                && was
                    .amount()
                    .zip(self.price.amount())
                    .is_some_and(|(was, current)| was > current)
        })
    }
}

pub struct OffersClient {
    http: Client,
    endpoint: Url,
}

impl OffersClient {
    pub fn from_env() -> IoResult<Self> {
        let base = configuration(var("CUSTOMER_FACING_URL"))?;
        Self::new(&base, Duration::from_secs(5))
    }

    pub fn new(base: &str, timeout: Duration) -> IoResult<Self> {
        let mut endpoint = Url::parse(base).map_err(|_| invalid_configuration())?;
        if !matches!(endpoint.scheme(), "http" | "https")
            || endpoint.host_str().is_none()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || base.trim() != base
        {
            return Err(invalid_configuration());
        }
        endpoint.set_path(&format!(
            "{}/api/offers",
            endpoint.path().trim_end_matches('/')
        ));
        let http = Client::builder()
            .timeout(timeout)
            .redirect(Policy::none())
            .retry(never())
            .build()
            .map_err(|_| IoError::other("could not initialize offers client"))?;
        Ok(Self { http, endpoint })
    }

    pub async fn fetch(&self, offset: i32) -> Result<Page, ()> {
        self.request(None, offset).await
    }

    pub async fn search(&self, query: &str, offset: i32) -> Result<Page, ()> {
        self.request(Some(query), offset).await
    }

    async fn request(&self, query: Option<&str>, offset: i32) -> Result<Page, ()> {
        if offset < 0 {
            return Err(());
        }
        let mut url = self.endpoint.clone();
        if let Some(query) = query {
            url.set_path(&format!("{}/search", url.path()));
            url.query_pairs_mut().append_pair("q", query);
        }
        url.query_pairs_mut()
            .append_pair("limit", &PAGE_SIZE.to_string())
            .append_pair("offset", &offset.to_string());
        let response = self.http.get(url).send().await.map_err(|_| ())?;
        if response.status() != StatusCode::OK {
            return Err(());
        }
        let page: Page = response.json().await.map_err(|_| ())?;
        if page.limit != PAGE_SIZE
            || page.offset != offset
            || page.items.len() > PAGE_SIZE as usize
            || (page.has_more && page.items.len() != PAGE_SIZE as usize)
            || page.items.iter().any(|offer| {
                offer.identifiers.guard().is_err()
                    || offer.localization.guard().is_err()
                    || guard_content(&offer.content, &offer.localization.language).is_err()
                    || offer
                        .availability
                        .as_ref()
                        .is_some_and(|value| value.guard().is_err())
                    || offer
                        .item_condition
                        .as_ref()
                        .is_some_and(|value| value.guard().is_err())
                    || offer
                        .was_price
                        .as_ref()
                        .is_some_and(|value| value.amount().is_none())
                    || offer.id.is_empty()
                    || offer.retailer_code.is_empty()
                    || offer.created_at.is_empty()
                    || offer.price.amount().is_none()
            })
        {
            return Err(());
        }
        Ok(page)
    }
}

fn configuration(value: Result<String, VarError>) -> IoResult<String> {
    match value {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        Err(VarError::NotPresent) => Ok("http://localhost:3003".into()),
        _ => Err(invalid_configuration()),
    }
}

fn invalid_configuration() -> IoError {
    IoError::new(
        ErrorKind::InvalidInput,
        "CUSTOMER_FACING_URL must be an absolute HTTP(S) URL without credentials, query or fragment",
    )
}

#[cfg(test)]
pub(crate) mod tests;

fn read_localization<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Localization, D::Error> {
    let mut value = Value::deserialize(deserializer)?;
    if let Some(market) = value
        .get_mut("target_market")
        .and_then(Value::as_object_mut)
    {
        if market.get("code").is_some_and(Value::is_null) {
            market.remove("code");
        }
        if market.get("label").is_some_and(Value::is_null) {
            market.remove("label");
        }
    }
    decode_json_value::<Localization>(value)
        .map(|value| value.canonicalized())
        .map_err(Error::custom)
}

/// Keep these boundary rules aligned with the customers app.
pub fn normalize_query(query: &str) -> Result<String, &'static str> {
    if query.chars().any(|c| c.is_control() && !c.is_whitespace()) {
        return Err("search text must not contain control characters");
    }
    let query = query.split_whitespace().collect::<Vec<_>>().join(" ");
    if !(1..=200).contains(&query.chars().count()) || !query.chars().any(char::is_alphanumeric) {
        return Err("enter 1–200 characters including a letter or number");
    }
    Ok(query)
}
