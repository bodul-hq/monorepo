/// The anonymous inline script that assigns Shopify's analytics `meta` object. It lists every
/// variant with its numeric id and its price, and is the only island that carries the product's
/// handle, vendor and type.
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
    /// `UGREEN FR` (195 pages), `UGREEN` (109) or `Seel` (2).
    pub vendor: String,
    /// Empty on 62 of the 306 saved pages.
    #[serde(rename = "type")]
    pub product_type: String,
    pub handle: String,
    pub variants: Vec<Variant>,
    pub remote: bool,
}

/// A variant as Shopify reports it to analytics. `price` is in hundredths of the shop currency,
/// the Shopify variant price, before any automatic discount.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: u64,
    pub price: i64,
    pub name: String,
    /// Null on a product without options (the 262 single-`Product` pages).
    pub public_title: Option<String>,
    /// Empty on the three gift cards (`ugreen-carte-cadeau-*`).
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
    use super::Meta;
    use html_extractor::serde_json::{self, Value, json};

    /// The first two variants of `products-45699.html`.
    pub(crate) fn meta() -> Value {
        json!({
            "product": {
                "id": 9526794977600_u64,
                "gid": "gid://shopify/Product/9526794977600",
                "vendor": "UGREEN FR",
                "type": "Nexode Charger",
                "handle": "45699",
                "variants": [
                    {"id": 50354801180992_u64, "price": 13499,
                        "name": "UGREEN 300W Nexode Chargeur - Chargeur",
                        "public_title": "Chargeur", "sku": "45699"},
                    {"id": 50354801213760_u64, "price": 18799,
                        "name": "UGREEN 300W Nexode Chargeur - + 2 Câble USB C 100W 1M",
                        "public_title": "+ 2 Câble USB C 100W 1M", "sku": "45699+70427P"}
                ],
                "remote": false
            },
            "page": {
                "pageType": "product",
                "resourceType": "product",
                "resourceId": 9526794977600_u64,
                "requestId": "ea0b7b5f-2880-4b53-b66f-7d49a5f323b6-1792013022"
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
    fn a_null_sku_fails() {
        let mut input = meta();
        input["product"]["variants"][0]["sku"] = Value::Null;

        assert!(serde_json::from_value::<Meta>(input).is_err());
    }

    #[test]
    fn an_unknown_key_fails() {
        let mut input = meta();
        input["product"]["variants"][0]["compare_at_price"] = json!(1);

        let error = serde_json::from_value::<Meta>(input).expect_err("not in the corpus");
        assert!(
            error
                .to_string()
                .contains("unknown field `compare_at_price`")
        );
    }
}
