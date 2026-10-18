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
    /// Empty on every product page; `SEEL-WFP` on the `worry-free-purchase` add-on.
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
    /// A string on all 140 saved variants, never empty.
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

    /// Trimmed from `products-ugreen-nasync-dxp4800-gt-4-bay-nas-storage.html`.
    pub(crate) fn meta() -> Value {
        json!({
            "product": {
                "id": 9553994776807_u64,
                "gid": "gid://shopify/Product/9553994776807",
                "vendor": "UGREEN DE",
                "type": "",
                "handle": "ugreen-nasync-dxp4800-gt-4-bay-nas-storage",
                "variants": [
                    {"id": 52118906241255_u64, "price": 58999,
                        "name": "UGREEN NASync DXP4800 GT 4-Bay 144TB NAS - DXP4800 GT 144TB",
                        "public_title": "DXP4800 GT 144TB", "sku": "16058"},
                    {"id": 52244153860327_u64, "price": 61598,
                        "name": "UGREEN NASync DXP4800 GT 4-Bay 144TB NAS - DXP4800 GT 144TB + 120W DC USV",
                        "public_title": "DXP4800 GT 144TB + 120W DC USV", "sku": "16058 + 45615"}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 9553994776807_u64,
                "requestId": "b679532b-8af9-464b-ae84-a754a3d514a1-1792010423"
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
        variant["product"]["variants"][0]["barcode"] = json!("6942752010583");
        assert!(serde_json::from_value::<Meta>(variant).is_err());

        let script = json!({"meta": meta(), "ShopifyAnalytics": {}});
        assert!(serde_json::from_value::<AnalyticsScript>(script).is_err());
    }
}
