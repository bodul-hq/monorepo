/// A `script[type="application/ld+json"]`: its attributes and its parsed body.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonItem {
    #[serde(rename = "type")]
    pub script_type: String,
    /// `jdgm-server-jld` on the judge.me reviews island; absent on the theme's.
    pub class: Option<String>,
    pub data: LdJson,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Product(ProductIsland),
    Organization(Organization),
    WebPage(WebPage),
    WebSite(WebSite),
}

/// Two islands on a page say `Product`: the theme's, with the offers, on every page, and
/// judge.me's, with the reviews, on pages that have any. Neither carries the other's keys.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum ProductIsland {
    Offers(Product),
    Reviews(ReviewsProduct),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    pub image: Vec<String>,
    pub description: String,
    pub sku: String,
    pub brand: Brand,
    pub offers: Vec<Offer>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    #[serde(rename = "@type")]
    pub kind: String,
    pub sku: String,
    /// Only on variants whose barcode is filled in.
    pub gtin13: Option<u64>,
    pub availability: String,
    pub price: f64,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewsProduct {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub name: String,
    #[serde(rename = "aggregateRating")]
    pub aggregate_rating: AggregateRating,
    pub image: String,
    pub review: Vec<Review>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateRating {
    #[serde(rename = "@type")]
    pub kind: String,
    /// A decimal string, such as `4.84`.
    #[serde(rename = "ratingValue")]
    pub rating_value: String,
    #[serde(rename = "reviewCount")]
    pub review_count: i64,
    #[serde(rename = "bestRating")]
    pub best_rating: i64,
    #[serde(rename = "worstRating")]
    pub worst_rating: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "reviewRating")]
    pub review_rating: ReviewRating,
    pub author: Author,
    #[serde(rename = "reviewBody")]
    pub review_body: String,
    #[serde(rename = "datePublished")]
    pub date_published: String,
    pub image: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRating {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "ratingValue")]
    pub rating_value: i64,
    #[serde(rename = "bestRating")]
    pub best_rating: i64,
    #[serde(rename = "worstRating")]
    pub worst_rating: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

/// The page renders two `Organization` islands: one with contact points, one with a logo.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    pub logo: Option<String>,
    #[serde(rename = "contactPoint")]
    pub contact_point: Option<Vec<ContactPoint>>,
    #[serde(rename = "sameAs")]
    pub same_as: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactPoint {
    #[serde(rename = "@type")]
    pub kind: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebPage {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub description: WebPageDescription,
    pub url: String,
}

/// Liquid refuses to serialize some descriptions and renders its refusal in their place.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum WebPageDescription {
    Text(String),
    Refused(LiquidError),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiquidError {
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebSite {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    #[serde(rename = "potentialAction")]
    pub potential_action: SearchAction,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchAction {
    #[serde(rename = "@type")]
    pub kind: String,
    pub target: String,
    #[serde(rename = "query-input")]
    pub query_input: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{LdJson, ProductIsland, WebPageDescription};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn theme_product() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@type": "Product",
            "name": "UGREEN Air 45W USB-C 当社最小充電器 GaNII 急速充電器",
            "url": "https://store.ugreen.jp/products/air-45w-usb-c-charger",
            "image": ["https://store.ugreen.jp/cdn/shop/files/519suz8uUOL.png?v=1788427694&width=1920"],
            "description": "\n\n\n  なぜ「Air」なのか。\n\n\n それは、空気のように薄く、軽いから。 \n\n",
            "sku": "35993",
            "brand": {"@type": "Brand", "name": "UGREEN-JP"},
            "offers": [
                {"@type": "Offer", "sku": "35993", "gtin13": 6941876268399_u64,
                    "availability": "http://schema.org/InStock", "price": 2996.0,
                    "priceCurrency": "JPY",
                    "url": "https://store.ugreen.jp/products/air-45w-usb-c-charger?variant=60268391956561"},
                {"@type": "Offer", "sku": "35994",
                    "availability": "http://schema.org/OutOfStock", "price": 2996.0,
                    "priceCurrency": "JPY",
                    "url": "https://store.ugreen.jp/products/air-45w-usb-c-charger?variant=60268391989329"}
            ]
        })
    }

    pub(crate) fn reviews_product() -> Value {
        json!({
            "@context": "https://schema.org/",
            "@type": "Product",
            "@id": "/products/air-45w-usb-c-charger#product",
            "name": "UGREEN Air 45W USB-C 当社最小充電器 GaNII 急速充電器",
            "aggregateRating": {"@type": "AggregateRating", "ratingValue": "4.67",
                "reviewCount": 3, "bestRating": 5, "worstRating": 1},
            "image": "https://cdn.shopify.com/s/files/1/0638/9245/5505/files/519suz8uUOL.png?v=1788427694",
            "review": [{"@type": "Review",
                "reviewRating": {"@type": "Rating", "ratingValue": 5, "bestRating": 5, "worstRating": 1},
                "author": {"@type": "Person", "name": "匿名"},
                "reviewBody": "小さい。", "datePublished": "2026-10-01T12:52:27Z"}]
        })
    }

    pub(crate) fn web_page() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "WebPage",
            "name": "UGREEN Air 45W USB-C 当社最小充電器 GaNII 急速充電器",
            "description": {"error": "json not allowed for this object"},
            "url": "https://store.ugreen.jp/products/air-45w-usb-c-charger"
        })
    }

    #[test]
    fn the_two_product_islands_are_told_apart_by_their_keys() {
        let theme = serde_json::from_value::<LdJson>(theme_product()).expect("deserializes");
        assert!(matches!(theme, LdJson::Product(ProductIsland::Offers(_))));

        let reviews = serde_json::from_value::<LdJson>(reviews_product()).expect("deserializes");
        assert!(matches!(
            reviews,
            LdJson::Product(ProductIsland::Reviews(_))
        ));
    }

    #[test]
    fn a_product_island_with_an_unseen_key_fails() {
        let mut input = theme_product();
        input["gtin"] = json!("1");

        assert!(serde_json::from_value::<LdJson>(input).is_err());
    }

    #[test]
    fn a_refused_web_page_description_deserializes() {
        let LdJson::WebPage(page) =
            serde_json::from_value::<LdJson>(web_page()).expect("deserializes")
        else {
            panic!("a web page");
        };
        assert!(matches!(page.description, WebPageDescription::Refused(_)));
    }

    #[test]
    fn an_unseen_type_fails() {
        let error = serde_json::from_value::<LdJson>(json!({"@type": "BreadcrumbList"}))
            .expect_err("not in the corpus");
        assert!(
            error
                .to_string()
                .contains("unknown variant `BreadcrumbList`")
        );
    }
}
