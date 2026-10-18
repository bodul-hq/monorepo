/// `meta#in-context-paypal-metadata`'s attributes: the one statement on every page of the
/// currency the prices are rendered in, EUR on EU/FR/IT and GBP on UK pages. The analytics prices name
/// none.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayPalMetadata {
    pub id: String,
    #[serde(rename = "data-shop-id")]
    pub shop_id: String,
    #[serde(rename = "data-venmo-supported")]
    pub venmo_supported: String,
    #[serde(rename = "data-environment")]
    pub environment: String,
    #[serde(rename = "data-locale")]
    pub locale: String,
    #[serde(rename = "data-paypal-v4")]
    pub paypal_v4: String,
    #[serde(rename = "data-currency")]
    pub currency: String,
}
