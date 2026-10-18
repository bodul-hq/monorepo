#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: LdJson,
}

/// The island types seen in the corpus: every saved page carries an `Organization` and a
/// `Product`, in that order.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Organization(Organization),
    Product(Product),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    /// `https:files/ugreen_logo.svg` on every page: the theme mangles the URL.
    pub logo: String,
    #[serde(rename = "sameAs")]
    pub same_as: Vec<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    /// The product page, its handle percent-encoded.
    pub url: String,
    /// The featured image alone, at a display `width`.
    pub image: Vec<String>,
    /// Plain text with the page's line breaks; empty on 5 saved pages.
    pub description: String,
    /// The selected variant's SKU. Missing on the 11 pages whose selected variant has none.
    pub sku: Option<String>,
    pub brand: Brand,
    /// One offer per variant, in the analytics' order.
    pub offers: Vec<Offer>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@type")]
    pub brand_type: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    #[serde(rename = "@type")]
    pub offer_type: String,
    /// Missing on the 19 saved variants without a SKU.
    pub sku: Option<String>,
    pub availability: String,
    /// Whole yen as a float, such as `13980.0`.
    pub price: f64,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// The product URL with `?variant=<id>`.
    pub url: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::LdJson;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "name": "株式会社ユーグリーン・ジャパン",
            "logo": "https:files/ugreen_logo.svg",
            "sameAs": ["https://x.com/ugreenjapan", ""],
            "url": "https://ugreen.jp"
        })
    }

    fn offer(id: u64, price: f64, sku: &str) -> Value {
        json!({
            "@type": "Offer",
            "sku": sku,
            "availability": "http://schema.org/InStock",
            "price": price,
            "priceCurrency": "JPY",
            "url": format!("https://ugreen.jp/products/lightning-usb-cable?variant={id}")
        })
    }

    /// Trimmed from `products-lightning-usb-cable.html`.
    pub(crate) fn product() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@type": "Product",
            "name": "UGREENライトニング USB ケーブル",
            "url": "https://ugreen.jp/products/lightning-usb-cable",
            "image": ["https://ugreen.jp/cdn/shop/files/60521-1.png?v=1726277735&width=1920"],
            "description": "\nL字型コネクター設計、\nApple 正規ライセンス MFi 取得",
            "sku": "60521",
            "brand": {"@type": "Brand", "name": "UGREEN-JP"},
            "offers": [
                offer(49142586999064, 1399.0, "60521"),
                offer(49154830631192, 1599.0, "70733")
            ]
        })
    }

    #[test]
    fn every_island_type_deserializes() {
        for island in [organization(), product()] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
    }

    #[test]
    fn a_product_and_an_offer_without_a_sku_deserialize() {
        let mut island = product();
        island.as_object_mut().expect("an object").remove("sku");
        island["offers"][0]
            .as_object_mut()
            .expect("an offer")
            .remove("sku");

        let Ok(LdJson::Product(product)) = serde_json::from_value::<LdJson>(island) else {
            panic!("a product island");
        };
        assert_eq!(product.sku, None);
        assert_eq!(product.offers[0].sku, None);
    }

    #[test]
    fn an_unseen_island_type_fails() {
        for island_type in ["BreadcrumbList", "WebPage", "ProductGroup"] {
            let error = serde_json::from_value::<LdJson>(json!({"@type": island_type}))
                .expect_err("the island type is not typed");
            assert!(
                error
                    .to_string()
                    .contains(&format!("unknown variant `{island_type}`"))
            );
        }
    }

    #[test]
    fn an_unknown_key_in_an_island_fails() {
        let mut island = product();
        island["gtin13"] = json!("1");
        assert!(serde_json::from_value::<LdJson>(island).is_err());

        let mut offer = product();
        offer["offers"][0]["gtin13"] = json!("1");
        assert!(serde_json::from_value::<LdJson>(offer).is_err());
    }
}
