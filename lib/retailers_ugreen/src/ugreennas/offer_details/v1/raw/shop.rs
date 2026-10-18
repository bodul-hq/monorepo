/// `script#apple-pay-shop-capabilities`: names the shop currency, the unit Liquid's amounts are
/// rendered in.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopScript {
    pub id: String,
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: ShopCapabilities,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ShopCapabilities {
    pub shop_id: u64,
    pub country_code: String,
    pub currency_code: String,
    pub merchant_capabilities: Vec<String>,
    pub merchant_id: String,
    pub merchant_name: String,
    pub required_billing_contact_fields: Vec<String>,
    pub required_shipping_contact_fields: Vec<String>,
    pub shipping_type: String,
    pub supported_networks: Vec<String>,
    pub total: Total,
    pub shopify_payments_enabled: bool,
    pub supports_subscriptions: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Total {
    #[serde(rename = "type")]
    pub total_type: String,
    pub label: String,
    pub amount: String,
}
