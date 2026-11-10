//! Compatibility routing for legacy Nordic downloads. Queries and raw source
//! models belong to each retailer; normalized document formats remain unchanged.
pub(crate) mod sitemap;
#[cfg(test)]
mod tests;
use ::shared::{NormalizedData, retailer::RetailerCode};
#[cfg(test)]
use html_extractor::serde_json::{self, json};
#[cfg(test)]
use money::Currency;
use money::Money;
use retailer_sauce::{
    identifier::{Gtin, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku},
    structuralized::StructuralizedData,
};
use serde::{Deserialize, Serialize};
pub(crate) trait Integration: Sync {
    fn endpoint(&self) -> &'static str;
    fn country(&self) -> &'static str;
    fn request(&self, url: &str) -> Result<String, String>;
    fn validate_download(&self, url: &str, body: &str) -> Result<(), String>;
    fn extract(&self, body: &str) -> Result<String, String>;
    fn normalize(&self, body: &str) -> Result<NormalizedData, String>;
    fn structuralize(&self, body: &str) -> Result<StructuralizedData, String>;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Market {
    Sweden,
    Norway,
    Finland,
    Iceland,
    Denmark,
}

impl Market {
    pub fn for_retailer(code: RetailerCode) -> Option<Self> {
        match code {
            RetailerCode::AnkerSe => Some(Self::Sweden),
            RetailerCode::AnkerNo => Some(Self::Norway),
            RetailerCode::AnkerFi => Some(Self::Finland),
            RetailerCode::AnkerIs => Some(Self::Iceland),
            RetailerCode::AnkerDk => Some(Self::Denmark),
            _ => None,
        }
    }

    fn integration(self) -> &'static dyn Integration {
        match self {
            Self::Sweden => &crate::ankerse::offer_details::v1::graphql::Store,
            Self::Norway => &crate::ankerno::offer_details::v1::graphql::Store,
            Self::Finland => &crate::ankerfi::offer_details::v1::graphql::Store,
            Self::Iceland => &crate::ankeris::offer_details::v1::graphql::Store,
            Self::Denmark => &crate::ankerdk::offer_details::v1::graphql::Store,
        }
    }
    pub fn endpoint(self) -> &'static str {
        self.integration().endpoint()
    }
    pub fn country(self) -> &'static str {
        self.integration().country()
    }
    pub fn request(self, url: &str) -> Result<String, String> {
        self.integration().request(url)
    }
    pub fn validate_download(self, url: &str, body: &str) -> Result<(), String> {
        self.integration().validate_download(url, body)
    }
    pub fn extract(self, body: &str) -> Result<String, String> {
        self.integration().extract(body)
    }
    pub fn normalize(self, body: &str) -> Result<NormalizedData, String> {
        self.integration().normalize(body)
    }
    pub fn structuralize(self, body: &str) -> Result<StructuralizedData, String> {
        self.integration().structuralize(body)
    }
    #[cfg(test)]
    fn currency(self) -> Currency {
        match self {
            Self::Sweden => Currency::SEK,
            Self::Norway => Currency::NOK,
            Self::Finland => Currency::EUR,
            Self::Iceland => Currency::ISK,
            Self::Denmark => Currency::DKK,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Document {
    pub(crate) country: String,
    pub(crate) language: String,
    pub(crate) id: ShopifyProductGid,
    pub(crate) handle: ShopifyHandle,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) canonical: String,
    pub(crate) images: Vec<String>,
    pub(crate) variants: Vec<Variant>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Variant {
    pub(crate) id: ShopifyProductVariationGid,
    pub(crate) sku: Option<Sku>,
    pub(crate) gtin: Option<Gtin>,
    pub(crate) price: Money,
    pub(crate) comparison: Option<Money>,
    pub(crate) available: bool,
}
