/// One `script[type="application/ld+json"]`. The theme's own scripts carry only the `type`; a
/// reviews app adds its own with an `id` (`a2r-03hu34xd-uhj7389-x` on all 71 of its islands).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    /// Present on the reviews app's scripts only: 71 islands on 70 of the 73 saved pages
    /// (`products-minisforum-ai-x1-pro-470.html` carries two).
    pub id: Option<String>,
    pub data: LdJson,
}

/// The island types seen in the corpus, per page: the Organization (73 of 73 pages), a
/// `ProductGroup` for a multi-variant product (70), a theme `Product` for a single-variant one
/// (1, `products-exclusive-use-link.html`), and neither for a product the theme renders no offer
/// for (2). A `FAQPage` appears on 11 pages. The reviews app's `Product` (71 islands on 70 pages)
/// comes on top of those.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Organization(Organization),
    ProductGroup(ProductGroup),
    Product(Product),
    #[serde(rename = "FAQPage")]
    FaqPage(FaqPage),
}

/// Two unrelated islands share the `Product` type: the theme's and the reviews app's. They are
/// told apart by their keys: the theme's carries `@id`, `url` and `category`, the reviews app's
/// `review` and `mpn`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum Product {
    Theme(Box<ThemeProduct>),
    Reviews(Box<ReviewsProduct>),
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductGroup {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub brand: Brand,
    pub category: String,
    /// Plain text; empty on 23 of the 70 islands.
    pub description: String,
    #[serde(rename = "hasVariant")]
    pub has_variant: Vec<GroupVariant>,
    pub name: String,
    #[serde(rename = "productGroupID")]
    pub product_group_id: String,
    /// The product URL with the handle percent-encoded: `.../products/minisforum-u%E5%9E%8B%E6%9E%95`
    /// for the record's `minisforum-u型枕` (2 pages carry such a handle).
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupVariant {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub variant_type: String,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    /// Absent where the variant has no SKU (12 of the 177 variants in a group).
    pub sku: Option<String>,
}

/// The theme's `Product` island, rendered for a product with a single variant.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeProduct {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub brand: Brand,
    /// Empty on the one saved island of this kind.
    pub category: String,
    pub description: String,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    pub sku: Option<String>,
    pub url: String,
}

/// The reviews app's `Product` island, 71 on 70 of the 73 saved pages. It restates the product with
/// its reviews, but its facts are not reliable: `image` is a path without a host
/// (`https:files/...`), `offers.itemCondition` is `UsedCondition` even on new products, and
/// `offers.price` is the yen amount with a dot as thousands separator (`51.999` for ¥51,999; 33
/// islands show two digits before the dot, 19 one, 18 three, 1 none). Nothing is read from it.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewsProduct {
    #[serde(rename = "@context")]
    pub context: String,
    pub brand: Brand,
    pub description: String,
    pub image: String,
    /// `null` on all 71.
    pub mpn: Option<String>,
    pub name: String,
    pub offers: ReviewsOffer,
    pub review: Vec<Review>,
    /// Absent where the product has no rating (30 of the 71 islands).
    #[serde(rename = "aggregateRating")]
    pub aggregate_rating: Option<AggregateRating>,
    /// `null` on 1 island.
    pub sku: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    #[serde(rename = "@type")]
    pub review_type: String,
    pub author: Person,
    #[serde(rename = "reviewBody")]
    pub review_body: String,
    #[serde(rename = "reviewRating")]
    pub review_rating: Rating,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Person {
    #[serde(rename = "@type")]
    pub person_type: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rating {
    #[serde(rename = "@type")]
    pub rating_type: String,
    #[serde(rename = "ratingValue")]
    pub rating_value: RatingValue,
}

/// A number on every review but one (`products-minisforum-deg2.html` renders `"4"`).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum RatingValue {
    Number(u64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateRating {
    #[serde(rename = "@type")]
    pub rating_type: String,
    #[serde(rename = "bestRating")]
    pub best_rating: String,
    #[serde(rename = "ratingValue")]
    pub rating_value: String,
    #[serde(rename = "reviewCount")]
    pub review_count: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewsOffer {
    #[serde(rename = "@type")]
    pub offer_type: String,
    pub availability: String,
    #[serde(rename = "itemCondition")]
    pub item_condition: String,
    pub price: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    #[serde(rename = "priceValidUntil")]
    pub price_valid_until: String,
    pub seller: Seller,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seller {
    #[serde(rename = "@type")]
    pub seller_type: String,
    pub name: String,
}

/// The FAQ block of 11 saved pages.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaqPage {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "mainEntity")]
    pub main_entity: Vec<Question>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    #[serde(rename = "@type")]
    pub question_type: String,
    pub name: String,
    #[serde(rename = "acceptedAnswer")]
    pub accepted_answer: Answer,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Answer {
    #[serde(rename = "@type")]
    pub answer_type: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@type")]
    pub brand_type: String,
    pub name: String,
}

/// The theme's offer.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub offer_type: String,
    pub availability: String,
    /// A decimal string in major units. Yen have no fraction, so it is a whole number such as
    /// `130399` on all 178 saved offers; the record renders the same price as `13039900`.
    pub price: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// The product URL with a `variant` query parameter.
    pub url: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{LdJson, LdJsonScript, Product, RatingValue};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "name": "Minisforum JP",
            "logo": "https://www.minisforum.jp/cdn/shop/files/MINISFORUM_logo_NO_BG.avif?v=1788419229&width=500",
            "url": "https://www.minisforum.jp"
        })
    }

    fn group_variant(id: u64, price: &str, sku: &str, kind: &str) -> Value {
        let handle = "minisforum-ms-03-core-ultra-5-336h-ultra-9-386h";
        json!({
            "@id": format!("/products/{handle}?variant={id}#variant"),
            "@type": "Product",
            "image": "https://www.minisforum.jp/cdn/shop/files/MS-01MS-03_NO_BP.webp?v=1787303758&width=1920",
            "name": format!("MINISFORUM MS-03 ローカルAIエッジワークステーション - Intel® Core™ Ultra 5 336H / {kind}"),
            "offers": {
                "@id": format!("/products/{handle}?variant={id}#offer"),
                "@type": "Offer",
                "availability": "http://schema.org/InStock",
                "price": price,
                "priceCurrency": "JPY",
                "url": format!("https://www.minisforum.jp/products/{handle}?variant={id}")
            },
            "sku": sku
        })
    }

    /// The island of `super::productjson::tests::product()`.
    pub(crate) fn product_group() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/minisforum-ms-03-core-ultra-5-336h-ultra-9-386h#product",
            "@type": "ProductGroup",
            "brand": {"@type": "Brand", "name": "Minisforum JP"},
            "category": "デスクトップPC",
            "description": "MS-03 ローカルAIエッジワークステーションはAI処理能力と、リモート制御機能を両立",
            "hasVariant": [
                group_variant(56749131137190, "130399", "MS-03-33600JP", "ベアボーンキット（OSライセンスなし）"),
                group_variant(56749137494182, "249999", "MS-03-33631JP", "32GB RAM + 1TB SSD")
            ],
            "name": "MINISFORUM MS-03 ローカルAIエッジワークステーション",
            "productGroupID": "15388676128934",
            "url": "https://www.minisforum.jp/products/minisforum-ms-03-core-ultra-5-336h-ultra-9-386h"
        })
    }

    /// The reviews app's island for the same product, with a review, an aggregate rating and the
    /// dotted yen price.
    pub(crate) fn reviews_product() -> Value {
        json!({
            "@context": "https://schema.org/",
            "@type": "Product",
            "brand": {"@type": "Brand", "name": "Minisforum JP"},
            "description": "MS-03 ローカルAIエッジワークステーション",
            "sku": "MS-03-33600JP",
            "mpn": null,
            "image": "https:files/MS-01MS-03_NO_BP.webp",
            "name": "MINISFORUM MS-03 ローカルAIエッジワークステーション",
            "review": [{
                "@type": "Review",
                "reviewRating": {"@type": "Rating", "ratingValue": 5},
                "author": {"@type": "Person", "name": "Unu-ma"},
                "reviewBody": "とても快適です。"
            }],
            "aggregateRating": {
                "@type": "AggregateRating",
                "ratingValue": "5.0",
                "bestRating": "5",
                "reviewCount": 1
            },
            "offers": {
                "@type": "Offer",
                "url": "https://www.minisforum.jp/products/minisforum-ms-03-core-ultra-5-336h-ultra-9-386h?variant=56749131137190",
                "priceCurrency": "JPY",
                "price": "130.399",
                "priceValidUntil": "2027-10-15",
                "itemCondition": "https://schema.org/UsedCondition",
                "availability": "http://schema.org/InStock",
                "seller": {"@type": "Organization", "name": "Shopify"}
            }
        })
    }

    /// The theme's single-variant island, as on `products-exclusive-use-link.html`.
    pub(crate) fn theme_product() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/exclusive-use-link#product",
            "@type": "Product",
            "brand": {"@type": "Brand", "name": "Minisforum JP"},
            "category": "",
            "description": "専用ページ",
            "image": "https://www.minisforum.jp/cdn/shop/products/LOGO_1.jpg?v=1765418810&width=1920",
            "name": "特殊販売専用",
            "offers": {
                "@id": "/products/exclusive-use-link?variant=40185880051878#offer",
                "@type": "Offer",
                "availability": "http://schema.org/InStock",
                "price": "100",
                "priceCurrency": "JPY",
                "url": "https://www.minisforum.jp/products/exclusive-use-link?variant=40185880051878"
            },
            "sku": "SPECIALJP02",
            "url": "https://www.minisforum.jp/products/exclusive-use-link"
        })
    }

    pub(crate) fn faq_page() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "FAQPage",
            "mainEntity": [{
                "@type": "Question",
                "name": "保証期間はどのくらいですか？",
                "acceptedAnswer": {"@type": "Answer", "text": "1年間です。"}
            }]
        })
    }

    #[test]
    fn every_island_type_deserializes() {
        for island in [
            organization(),
            product_group(),
            reviews_product(),
            theme_product(),
            faq_page(),
        ] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
    }

    #[test]
    fn the_two_product_islands_are_told_apart() {
        let theme = serde_json::from_value::<LdJson>(theme_product()).expect("deserializes");
        let reviews = serde_json::from_value::<LdJson>(reviews_product()).expect("deserializes");

        assert!(matches!(theme, LdJson::Product(Product::Theme(_))));
        assert!(matches!(reviews, LdJson::Product(Product::Reviews(_))));
    }

    #[test]
    fn the_scripts_id_is_the_reviews_apps() {
        let theme = json!({"type": "application/ld+json", "data": organization()});
        let reviews = json!({
            "type": "application/ld+json",
            "id": "a2r-03hu34xd-uhj7389-x",
            "data": reviews_product()
        });

        let theme = serde_json::from_value::<LdJsonScript>(theme).expect("deserializes");
        let reviews = serde_json::from_value::<LdJsonScript>(reviews).expect("deserializes");
        assert_eq!(theme.id, None);
        assert_eq!(reviews.id.as_deref(), Some("a2r-03hu34xd-uhj7389-x"));
    }

    #[test]
    fn a_new_script_attribute_fails() {
        let input = json!({"type": "application/ld+json", "class": "x", "data": organization()});

        assert!(serde_json::from_value::<LdJsonScript>(input).is_err());
    }

    #[test]
    fn an_unreviewed_product_has_no_rating_and_a_missing_sku_is_null() {
        let mut input = reviews_product();
        let island = input.as_object_mut().expect("an object");
        island.remove("aggregateRating");
        island.remove("review");
        island.insert("review".to_string(), json!([]));
        island.insert("sku".to_string(), Value::Null);

        let Ok(LdJson::Product(Product::Reviews(product))) = serde_json::from_value(input) else {
            panic!("the reviews island deserializes");
        };
        assert!(product.aggregate_rating.is_none());
        assert!(product.sku.is_none());
    }

    #[test]
    fn a_rating_is_a_number_or_a_string() {
        let mut input = reviews_product();
        input["review"][0]["reviewRating"]["ratingValue"] = json!("4");

        let Ok(LdJson::Product(Product::Reviews(product))) = serde_json::from_value(input) else {
            panic!("the reviews island deserializes");
        };
        assert_eq!(
            product.review[0].review_rating.rating_value,
            RatingValue::Text("4".to_string())
        );
    }

    #[test]
    fn an_unseen_island_type_fails() {
        for island_type in ["BreadcrumbList", "WebSite"] {
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
        for mut island in [
            organization(),
            product_group(),
            reviews_product(),
            theme_product(),
            faq_page(),
        ] {
            island["gtin"] = json!("1");

            assert!(serde_json::from_value::<LdJson>(island).is_err());
        }
    }
}
