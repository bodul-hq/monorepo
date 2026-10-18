#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: LdJson,
}

/// The island types seen in the corpus: every page renders exactly one of each (29 of 29), in the
/// order Product, BreadcrumbList, Organization. Unlike the other Minisforum shops' Shopify
/// themes, a multi-variant product is a `Product` with one offer per variant, not a
/// `ProductGroup`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Product(Box<Product>),
    BreadcrumbList(BreadcrumbList),
    Organization(Organization),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    /// The Shopify product id, as a number.
    #[serde(rename = "productID")]
    pub product_id: u64,
    /// One offer per variant, in the variants' order.
    pub offers: Vec<Offer>,
    /// Present on 11 of the 29 saved pages: the products with reviews.
    #[serde(rename = "aggregateRating", default)]
    pub aggregate_rating: Option<AggregateRating>,
    pub brand: Brand,
    pub name: String,
    /// Plain text, with the HTML entities of the product's description left in it (`&amp;`) and
    /// whitespace padding. Blank on 7 of the 29 saved pages.
    pub description: String,
    /// Empty on 6 of the 29 saved pages.
    pub category: String,
    /// The product's path on the storefront, such as `/products/minisforum-ms-a1`, percent-encoded
    /// where the handle is not ASCII (2 pages).
    pub url: String,
    /// The SKU of the variant the page opens on; empty where that variant has none (1 page).
    pub sku: String,
    pub image: Image,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    #[serde(rename = "@type")]
    pub image_type: String,
    /// The product's primary image at a display width of 1024.
    pub url: String,
    /// Equal to `url` on every page.
    pub image: String,
    /// Empty on every page.
    pub name: String,
    pub width: String,
    pub height: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@type")]
    pub brand_type: String,
    /// Free text entered per shop: `Minisforum HK` (25 pages), `Minisforum CN` (3) and
    /// `Minisforum` (1) appear.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    #[serde(rename = "@type")]
    pub offer_type: String,
    /// The variant's `public_title`, or `Default Title` for a product without options.
    pub name: String,
    pub availability: String,
    /// A number in major units, such as `2199.0`.
    pub price: f64,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// A date about a fortnight ahead of the page's retrieval.
    #[serde(rename = "priceValidUntil")]
    pub price_valid_until: String,
    /// Missing where the variant has no SKU (1 of the 78 saved offers).
    #[serde(default)]
    pub sku: Option<String>,
    /// A path with a `variant` query parameter, such as `/products/a?variant=1`.
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateRating {
    #[serde(rename = "@type")]
    pub rating_type: String,
    /// A decimal string such as `4.88`.
    #[serde(rename = "ratingValue")]
    pub rating_value: String,
    #[serde(rename = "reviewCount")]
    pub review_count: String,
    #[serde(rename = "worstRating")]
    pub worst_rating: String,
    #[serde(rename = "bestRating")]
    pub best_rating: String,
}

/// Always two entries on the saved pages: the shop (empty name) and the product.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BreadcrumbList {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "itemListElement")]
    pub item_list_element: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListItem {
    #[serde(rename = "@type")]
    pub item_type: String,
    pub position: i64,
    pub name: String,
    pub item: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub logo: String,
    pub url: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::LdJson;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) const IMAGE: &str =
        "https://minisforum.hk/cdn/shop/files/MS-S1-Max.png?v=1758849971&width=1024";

    fn offer(variant: u64, name: &str, price: f64, sku: &str) -> Value {
        json!({
            "@type": "Offer",
            "name": name,
            "availability": "https://schema.org/InStock",
            "price": price,
            "priceCurrency": "HKD",
            "priceValidUntil": "2026-10-25",
            "sku": sku,
            "url": format!("/products/minisforum-ms-s1-max?variant={variant}")
        })
    }

    /// Trimmed from `products-minisforum-ms-s1-max.html`; the island of
    /// `super::super::variants::tests::variants()`.
    pub(crate) fn product() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "Product",
            "productID": 8611751919794_u64,
            "offers": [
                offer(46559370805426, "AMD Ryzen™ AI Max+ 395 / 128GB + 2TB SSD", 32999.0, "CS-MMSS1MD"),
                offer(67586234319026, "AMD Ryzen™ AI Max+ PRO 495 / 192GB + 2TB SSD", 54999.0, "CS-MMSS14D")
            ],
            "aggregateRating": {
                "@type": "AggregateRating",
                "ratingValue": "5.0",
                "reviewCount": "3",
                "worstRating": "1.0",
                "bestRating": "5.0"
            },
            "brand": {"@type": "Brand", "name": "Minisforum HK"},
            "name": "MS-S1 Max",
            "description": "MINISFORUM MS-S1 Max MINI WORKSTATION\n\nRAM：LPDDR5x Unified Memory\n",
            "category": "Mini PC",
            "url": "/products/minisforum-ms-s1-max",
            "sku": "CS-MMSS1MD",
            "image": {
                "@type": "ImageObject",
                "url": IMAGE,
                "image": IMAGE,
                "name": "",
                "width": "1024",
                "height": "1024"
            }
        })
    }

    pub(crate) fn breadcrumbs() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "BreadcrumbList",
            "itemListElement": [
                {"@type": "ListItem", "position": 1, "name": "", "item": "https://minisforum.hk"},
                {
                    "@type": "ListItem",
                    "position": 2,
                    "name": "MS-S1 Max",
                    "item": "https://minisforum.hk/products/minisforum-ms-s1-max"
                }
            ]
        })
    }

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "Organization",
            "name": "Minisforum HK",
            "logo": "https://minisforum.hk/cdn/shop/files/logo_e5ca10ae-b441-4650-a3b9-2b38a74850fe_3000x.png?v=1675752105",
            "url": "https://minisforum.hk"
        })
    }

    #[test]
    fn every_island_type_deserializes() {
        for island in [product(), breadcrumbs(), organization()] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
    }

    #[test]
    fn a_product_without_reviews_or_an_offer_sku_deserializes() {
        let mut input = product();
        let object = input.as_object_mut().expect("an object");
        object.remove("aggregateRating");
        object["offers"][0]
            .as_object_mut()
            .expect("an offer")
            .remove("sku");

        let LdJson::Product(product) = serde_json::from_value(input).expect("deserializes") else {
            panic!("a product");
        };
        assert!(product.aggregate_rating.is_none());
        assert_eq!(product.offers[0].sku, None);
    }

    #[test]
    fn an_unseen_island_type_fails() {
        for island_type in ["ProductGroup", "FAQPage"] {
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
        for island in [product(), breadcrumbs(), organization()] {
            let mut island = island;
            island["gtin"] = json!("1");

            assert!(serde_json::from_value::<LdJson>(island).is_err());
        }
    }

    #[test]
    fn an_unknown_key_in_an_offer_fails() {
        let mut input = product();
        input["offers"][0]["itemCondition"] = json!("https://schema.org/NewCondition");

        assert!(serde_json::from_value::<LdJson>(input).is_err());
    }
}
