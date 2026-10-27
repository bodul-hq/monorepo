//! Server-only client for the customer-facing browse contract.

use money::Currency;
use reqwest::{Client, Url};
use serde::Deserialize;
use std::{env::VarError, io, time::Duration};

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
        if offset < 0 {
            return Err(());
        }
        let mut url = self.endpoint.clone();
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
                offer.id.is_empty()
                    || offer.retailer_code.is_empty()
                    || offer.created_at.is_empty()
                    || !offer
                        .price
                        .amount
                        .parse::<i64>()
                        .is_ok_and(|amount| amount.to_string() == offer.price.amount)
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
