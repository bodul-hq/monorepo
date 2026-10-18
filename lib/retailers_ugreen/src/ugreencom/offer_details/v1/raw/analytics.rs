/// The anonymous inline script that assigns Shopify's analytics `meta` object: on this site the
/// product's only record.
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
    /// Empty on all 66 saved pages.
    #[serde(rename = "type")]
    pub product_type: String,
    pub handle: String,
    /// Exactly one on all 66 saved pages.
    pub variants: Vec<Variant>,
    /// False on all 66 saved pages.
    pub remote: bool,
}

/// A summary of a variant: id, price, name and SKU.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    /// Hundredths of the shop currency, which the record does not name. A placeholder on this
    /// site: `130000` on 65 pages and `500` on one.
    pub price: i64,
    pub name: String,
    /// Null on every saved variant, each the product's only one.
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

#[cfg(test)]
pub(crate) mod tests {
    use super::AnalyticsScript;
    use html_extractor::serde_json::{self, Value, json};

    /// Trimmed from `products-usa-15495.html`.
    pub(crate) fn script() -> Value {
        json!({
            "meta": {
                "product": {
                    "id": 9027458236635_u64,
                    "gid": "gid://shopify/Product/9027458236635",
                    "vendor": "UGREEN Brandsite",
                    "type": "",
                    "handle": "usa-15495",
                    "variants": [
                        {"id": 47897996820699_u64, "price": 130000,
                            "name": "UGREEN Revodok 105 5-in-1 USB-C Hub",
                            "public_title": null, "sku": "usa-15495"}
                    ],
                    "remote": false
                },
                "page": {
                    "pageType": "product",
                    "resourceType": "product",
                    "resourceId": 9027458236635_u64,
                    "requestId": "1e95e06d-ef41-449c-9ea3-d729847caa6a-1792012918"
                }
            }
        })
    }

    #[test]
    fn the_meta_object_deserializes() {
        let script = serde_json::from_value::<AnalyticsScript>(script()).expect("deserializes");
        assert_eq!(script.meta.product.variants.len(), 1);
        assert_eq!(script.meta.product.variants[0].price, 130000);
    }

    #[test]
    fn a_new_key_fails() {
        let mut input = script();
        input["meta"]["product"]["variants"][0]["barcode"] = json!("6957303815495");

        assert!(serde_json::from_value::<AnalyticsScript>(input).is_err());
    }
}
