/// The inline script ahead of `script.analytics`, which assigns Shopify's analytics `meta`
/// object. The script has no attributes, so `meta` is all the architecture extracts from it.
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

/// A variant as Shopify reports it to analytics. `price` is in hundredths of the shop currency
/// (paise).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub price: i64,
    pub name: String,
    /// Null on a product without options.
    pub public_title: Option<String>,
    /// Set on every variant of the saved pages.
    pub sku: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Page {
    pub page_type: String,
    pub resource_type: String,
    pub resource_id: u64,
    pub request_id: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{AnalyticsScript, Meta};
    use html_extractor::serde_json::{self, Value, json};

    fn variant(id: u64, sku: &str, colour: &str) -> Value {
        json!({
            "id": id,
            "name": format!("UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest - {colour}"),
            "price": 99900,
            "public_title": colour,
            "sku": sku
        })
    }

    /// The analytics of `super::productjson::tests::product()`.
    pub(crate) fn meta() -> Value {
        json!({
            "page": {
                "pageType": "product",
                "requestId": "26bcc819-328c-42f6-9c57-91332f916110-1792012639",
                "resourceId": 7702424813667_u64,
                "resourceType": "product"
            },
            "product": {
                "gid": "gid://shopify/Product/7702424813667",
                "handle": "ugreen-ergonomic-mouse-pad-black-25245",
                "id": 7702424813667_u64,
                "remote": false,
                "type": "Mouse Pad",
                "variants": [
                    variant(43454938677347, "25245", "Black"),
                    variant(43742608687203, "25243", "Green"),
                    variant(43742608719971, "25244", "Gray")
                ],
                "vendor": "UGREEN"
            }
        })
    }

    #[test]
    fn the_meta_object_deserializes() {
        let meta = serde_json::from_value::<Meta>(meta()).expect("deserializes");
        assert_eq!(meta.product.variants.len(), 3);
    }

    #[test]
    fn a_variant_without_options_has_no_public_title() {
        let mut input = meta();
        input["product"]["variants"][0]["public_title"] = Value::Null;

        let meta = serde_json::from_value::<Meta>(input).expect("deserializes");
        assert!(meta.product.variants[0].public_title.is_none());
    }

    #[test]
    fn an_attribute_on_the_script_fails() {
        let input = json!({"meta": meta(), "id": "analytics"});

        assert!(serde_json::from_value::<AnalyticsScript>(input).is_err());
    }
}
