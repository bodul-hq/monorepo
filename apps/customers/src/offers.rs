//! Server-only client for the customer-facing browse contract.

use money::Currency;
use reqwest::{Client, Url};
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
pub struct Offer {
    #[serde(default)]
    pub was_price: Option<Price>,
    #[serde(default, deserialize_with = "facts::read_availability")]
    pub availability: Option<retailer_sauce::availability::Availability>,
    #[serde(default, deserialize_with = "facts::read_condition")]
    pub item_condition: Option<retailer_sauce::item_condition::ItemCondition>,
    #[serde(default, deserialize_with = "facts::read_content")]
    pub content: retailer_sauce::structuralized::content::Content,
    #[serde(flatten)]
    pub identifiers: retailer_sauce::identifier::ProductIdentifiers,
    #[serde(default, deserialize_with = "read_localization")]
    pub localization: retailer_sauce::localization::Localization,
    #[serde(default)]
    pub images: Vec<String>,
    pub id: String,
    pub retailer_code: String,
    pub title: String,
    pub price: Price,
    pub url: String,
    pub created_at: String,
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
    pub fn from_env() -> io::Result<Self> {
        let base = configuration(std::env::var("CUSTOMER_FACING_URL"))?;
        Self::new(&base, Duration::from_secs(5))
    }

    pub fn new(base: &str, timeout: Duration) -> io::Result<Self> {
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
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .build()
            .map_err(|_| io::Error::other("could not initialize offers client"))?;
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
        if response.status() != reqwest::StatusCode::OK {
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
                    || offer.content.guard().is_err()
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

fn configuration(value: Result<String, VarError>) -> io::Result<String> {
    match value {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        Err(VarError::NotPresent) => Ok("http://localhost:3003".into()),
        _ => Err(invalid_configuration()),
    }
}

fn invalid_configuration() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "CUSTOMER_FACING_URL must be an absolute HTTP(S) URL without credentials, query or fragment",
    )
}

#[cfg(test)]
pub(crate) mod tests;

fn read_localization<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<retailer_sauce::localization::Localization, D::Error> {
    let mut value = serde_json::Value::deserialize(deserializer)?;
    if let Some(market) = value
        .get_mut("target_market")
        .and_then(serde_json::Value::as_object_mut)
    {
        if market.get("code").is_some_and(serde_json::Value::is_null) {
            market.remove("code");
        }
        if market.get("label").is_some_and(serde_json::Value::is_null) {
            market.remove("label");
        }
    }
    serde_json::from_value(value).map_err(serde::de::Error::custom)
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
