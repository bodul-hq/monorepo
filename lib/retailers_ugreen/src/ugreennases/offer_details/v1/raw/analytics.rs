/// The anonymous inline script that assigns Shopify's analytics `meta` object. It is on every
/// page, and the only island that lists every variant with its price whether or not the product
/// renders a picker.
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
    /// Empty on every saved page.
    #[serde(rename = "type")]
    pub product_type: String,
    pub handle: String,
    pub variants: Vec<Variant>,
    pub remote: bool,
}

/// A variant as Shopify reports it to analytics. `price` is in hundredths of the shop currency.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub price: i64,
    pub name: String,
    /// Null on a product without options.
    pub public_title: Option<String>,
    /// A string on all 37 saved variants, never empty.
    pub sku: String,
}

/// The request, not the product: a product page of `resourceId` the product id, under a fresh
/// `requestId`.
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

    /// Trimmed from `products-ugreen-nasync-dxp2800-gt-2-bay-nas-storage.html`.
    pub(crate) fn meta() -> Value {
        json!({
            "product": {
                "id": 10254048985370_u64,
                "gid": "gid://shopify/Product/10254048985370",
                "vendor": "UGREEN NAS US",
                "type": "",
                "handle": "ugreen-nasync-dxp2800-gt-2-bay-nas-storage",
                "variants": [
                    {"id": 52296690008346_u64, "price": 45999,
                        "name": "UGREEN NASync DXP2800 GT - DXP2800 GT (Envío desde el 27 de octubre)",
                        "public_title": "DXP2800 GT (Envío desde el 27 de octubre)", "sku": "16051"},
                    {"id": 52640999375130_u64, "price": 48999,
                        "name": "UGREEN NASync DXP2800 GT - DXP2800 GT + 120W DC UPS (Envío desde el 27 de octubre)",
                        "public_title": "DXP2800 GT + 120W DC UPS (Envío desde el 27 de octubre)", "sku": "16051+45615"}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 10254048985370_u64,
                "requestId": "0611a23e-ceee-42c4-93b4-5d3de4133ca0-1792010431"
            }
        })
    }

    #[test]
    fn the_meta_object_deserializes() {
        let meta = serde_json::from_value::<Meta>(meta()).expect("deserializes");
        assert_eq!(meta.product.variants.len(), 2);
    }

    #[test]
    fn a_variant_without_options_has_no_public_title() {
        let mut input = meta();
        input["product"]["variants"][0]["public_title"] = Value::Null;

        let meta = serde_json::from_value::<Meta>(input).expect("deserializes");
        assert!(meta.product.variants[0].public_title.is_none());
    }

    #[test]
    fn an_unknown_key_fails() {
        let mut variant = meta();
        variant["product"]["variants"][0]["barcode"] = json!("6942752010514");
        assert!(serde_json::from_value::<Meta>(variant).is_err());

        let script = json!({"meta": meta(), "ShopifyAnalytics": {}});
        assert!(serde_json::from_value::<AnalyticsScript>(script).is_err());
    }
}
