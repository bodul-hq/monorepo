/// `meta#in-context-paypal-metadata`'s attributes: the one statement on every page of the
/// currency the prices are rendered in, `EUR` on all 18 saved pages. The analytics prices name
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

#[cfg(test)]
pub(crate) mod tests {
    use super::PayPalMetadata;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn metadata() -> Value {
        json!({
            "data-currency": "EUR",
            "data-environment": "production",
            "data-locale": "es_ES",
            "data-paypal-v4": "true",
            "data-shop-id": "97799504154",
            "data-venmo-supported": "false",
            "id": "in-context-paypal-metadata"
        })
    }

    #[test]
    fn the_attributes_deserialize() {
        let metadata = serde_json::from_value::<PayPalMetadata>(metadata()).expect("deserializes");
        assert_eq!(metadata.currency, "EUR");
    }

    #[test]
    fn a_new_attribute_fails() {
        let mut input = metadata();
        input["data-intent"] = json!("capture");

        assert!(serde_json::from_value::<PayPalMetadata>(input).is_err());
    }
}
