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

#[cfg(test)]
pub(crate) mod tests {
    use super::ShopCapabilities;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn data() -> Value {
        json!({
            "shopId": 66596602144_u64,
            "countryCode": "US",
            "currencyCode": "CAD",
            "merchantCapabilities": ["supports3DS"],
            "merchantId": "gid://shopify/Shop/66596602144",
            "merchantName": "UGREEN CA",
            "requiredBillingContactFields": ["postalAddress", "email", "phone"],
            "requiredShippingContactFields": ["postalAddress", "email", "phone"],
            "shippingType": "shipping",
            "supportedNetworks": ["visa", "masterCard", "amex", "discover", "elo", "jcb"],
            "total": {"type": "pending", "label": "UGREEN CA", "amount": "1.00"},
            "shopifyPaymentsEnabled": true,
            "supportsSubscriptions": true
        })
    }

    #[test]
    fn the_keys_are_camel_case() {
        let shop = serde_json::from_value::<ShopCapabilities>(data()).expect("deserializes");
        assert_eq!(shop.currency_code, "CAD");
    }
}
