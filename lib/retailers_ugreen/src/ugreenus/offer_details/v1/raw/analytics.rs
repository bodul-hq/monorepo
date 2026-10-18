/// The inline script ahead of `script.analytics`, which assigns Shopify's analytics `meta`
/// object. It is the only island that lists every variant with its price, and the only one that
/// carries the product's handle, vendor and type.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyticsScript {
    pub meta: Meta,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    pub product: Product,
    pub page: Page,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    pub id: u64,
    pub gid: String,
    pub vendor: String,
    #[serde(rename = "type")]
    pub product_type: String,
    pub handle: String,
    pub variants: Vec<Variant>,
    pub remote: bool,
}

/// A variant as Shopify reports it to analytics. `price` is in hundredths of the shop currency,
/// before any automatic discount.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub price: i64,
    pub name: String,
    /// Null on a product without options.
    pub public_title: Option<String>,
    pub sku: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Page {
    pub page_type: String,
    pub resource_type: String,
    pub resource_id: u64,
    pub request_id: String,
}
