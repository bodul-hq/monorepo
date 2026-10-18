use html_extractor::serde_json::{self, Value};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    /// Only the reviews app's script has one, `a2r-03hu34xd-uhj7389-x` on all 70.
    pub id: Option<String>,
    pub data: LdJson,
}

/// The island types seen in the corpus. Every page renders an `Organization` (118 pages). A
/// product renders a `ProductGroup` (110 pages) or, where it has a single option-less variant, a
/// `Product` (6), and a product without a price renders neither (2). A reviews app adds a second,
/// differently shaped `Product` to 70 pages; see [`Product`]. `minisforumuk`'s `FAQPage` island
/// appears on no saved page, so it is not typed.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Organization(Organization),
    ProductGroup(ProductGroup),
    Product(Product),
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
    /// Google product category, in the page's language.
    pub category: String,
    pub description: String,
    #[serde(rename = "hasVariant")]
    pub has_variant: Vec<GroupVariant>,
    pub name: String,
    #[serde(rename = "productGroupID")]
    pub product_group_id: String,
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
    /// Absent where the variant has no SKU (28 of the 366 variants in a group: the record's SKU is null or empty there).
    pub sku: Option<String>,
}

/// The two `Product` islands on the saved pages come from different sources and share nothing
/// but their type: the storefront's own, which restates the record's offers, and a reviews app's.
/// They are told apart by the reviews app's `review` key, so that a malformed island reports the
/// field at fault instead of "no variant matched".
#[derive(Debug, Clone, PartialEq)]
pub enum Product {
    /// The storefront's, on the 6 pages whose product has one option-less variant.
    Native(Box<NativeProduct>),
    /// The reviews app's, on 70 pages. It states one offer of its own, the shop's `UsedCondition`
    /// for every product, so it is not a restatement of the record.
    Reviewed(Box<ReviewedProduct>),
}

impl<'de> Deserialize<'de> for Product {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if value.get("review").is_some() {
            serde_json::from_value(value)
                .map(|product| Product::Reviewed(Box::new(product)))
                .map_err(serde::de::Error::custom)
        } else {
            serde_json::from_value(value)
                .map(|product| Product::Native(Box::new(product)))
                .map_err(serde::de::Error::custom)
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProduct {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub brand: Brand,
    pub category: String,
    pub description: String,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    pub sku: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedProduct {
    #[serde(rename = "@context")]
    pub context: String,
    pub brand: Brand,
    pub description: String,
    /// Not a usable URL, as in `https:files/895d7-mini-itx-1.png`.
    pub image: String,
    /// `null` on all 70.
    pub mpn: Option<String>,
    pub name: String,
    pub sku: String,
    pub offers: ReviewedOffer,
    /// Empty on 6 of the 70 pages.
    pub review: Vec<Review>,
    /// Absent exactly where `review` is empty.
    #[serde(rename = "aggregateRating")]
    pub aggregate_rating: Option<AggregateRating>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedOffer {
    #[serde(rename = "@type")]
    pub offer_type: String,
    pub url: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// A decimal string in major units, such as `539.00`.
    pub price: String,
    /// `2027-10-15` on all 70.
    #[serde(rename = "priceValidUntil")]
    pub price_valid_until: String,
    /// `https://schema.org/UsedCondition` on all 70, new products included.
    #[serde(rename = "itemCondition")]
    pub item_condition: String,
    pub availability: String,
    pub seller: Seller,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seller {
    #[serde(rename = "@type")]
    pub seller_type: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    #[serde(rename = "@type")]
    pub review_type: String,
    pub author: Author,
    #[serde(rename = "reviewBody")]
    pub review_body: String,
    #[serde(rename = "reviewRating")]
    pub review_rating: Rating,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
    #[serde(rename = "@type")]
    pub author_type: String,
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

/// A number on 126 of the 128 saved reviews; the app renders `"4"` for the other 2.
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
    /// A decimal string, such as `4.5`.
    #[serde(rename = "ratingValue")]
    pub rating_value: String,
    #[serde(rename = "reviewCount")]
    pub review_count: u64,
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
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub offer_type: String,
    pub availability: String,
    /// A decimal string in major units with a point, such as `1079.00`; no thousands separator
    /// appears on any saved page.
    pub price: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// The product URL with a `variant` query parameter.
    pub url: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{LdJson, Product};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "name": "Minisforum FR",
            "logo": "https://minisforumpc.fr/cdn/shop/files/Logo.svg?v=1782286833&width=500",
            "url": "https://minisforumpc.fr"
        })
    }

    fn group_variant(id: u64, price: &str, sku: &str, kind: &str) -> Value {
        json!({
            "@id": format!("/products/minisforum-ms-03-workstation?variant={id}#variant"),
            "@type": "Product",
            "image": "https://minisforumpc.fr/cdn/shop/files/ms-03-local-ai-edge-workstation-1.png?v=1788346698&width=1920",
            "name": format!("Minisforum MS-03 Workstation Locale IA Edge - Intel® Core™ Ultra 5 336H / {kind}"),
            "offers": {
                "@id": format!("/products/minisforum-ms-03-workstation?variant={id}#offer"),
                "@type": "Offer",
                "availability": "http://schema.org/InStock",
                "price": price,
                "priceCurrency": "EUR",
                "url": format!("https://minisforumpc.fr/products/minisforum-ms-03-workstation?variant={id}")
            },
            "sku": sku
        })
    }

    /// The island of `super::productjson::tests::product()`.
    pub(crate) fn product_group() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/minisforum-ms-03-workstation#product",
            "@type": "ProductGroup",
            "brand": {"@type": "Brand", "name": "Minisforum FR"},
            "category": "Ordinateurs de bureau",
            "description": "",
            "hasVariant": [
                group_variant(64248383471985, "799.00", "MS-03-33600EU", "Barebone (Licence OS non incluse)"),
                group_variant(64248383504753, "1399.00", "MS-03-33631EU", "32GB RAM + 1TB SSD")
            ],
            "name": "Minisforum MS-03 Workstation Locale IA Edge",
            "productGroupID": "15885734478193",
            "url": "https://minisforumpc.fr/products/minisforum-ms-03-workstation"
        })
    }

    /// The storefront's own `Product` island, for a one-variant product.
    pub(crate) fn native_product() -> Value {
        let group = product_group();
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/minisforum-ms-03-workstation#product",
            "@type": "Product",
            "brand": group["brand"],
            "category": "Ordinateurs de bureau",
            "description": "",
            "image": group["hasVariant"][0]["image"],
            "name": group["name"],
            "offers": group["hasVariant"][0]["offers"],
            "sku": "MS-03-33600EU",
            "url": group["url"]
        })
    }

    /// The reviews app's `Product` island for the fixture product, in the shape of the one on
    /// `products-minisforum-m2-356h.html`.
    pub(crate) fn reviewed_product() -> Value {
        json!({
            "@context": "https://schema.org/",
            "@type": "Product",
            "brand": {"@type": "Brand", "name": "Minisforum FR"},
            "description": "",
            "sku": "MS-03-33600EU",
            "mpn": null,
            "image": "https:files/ms-03-local-ai-edge-workstation-1.png",
            "name": "Minisforum MS-03 Workstation Locale IA Edge",
            "review": [
                {
                    "@type": "Review",
                    "reviewRating": {"@type": "Rating", "ratingValue": 5},
                    "author": {"@type": "Person", "name": "Wanda Brakus"},
                    "reviewBody": "Excellent rapport qualité-prix, coupe et confort！！！"
                },
                {
                    "@type": "Review",
                    "reviewRating": {"@type": "Rating", "ratingValue": "4"},
                    "author": {"@type": "Person", "name": "Ms. Florence O'Connell"},
                    "reviewBody": "Les performances de ce produit sont étonnamment bonnes."
                }
            ],
            "aggregateRating": {
                "@type": "AggregateRating",
                "ratingValue": "4.5",
                "bestRating": "5",
                "reviewCount": 2
            },
            "offers": {
                "@type": "Offer",
                "url": "https://minisforumpc.fr/products/minisforum-ms-03-workstation?variant=64248383471985",
                "priceCurrency": "EUR",
                "price": "799.00",
                "priceValidUntil": "2027-10-15",
                "itemCondition": "https://schema.org/UsedCondition",
                "availability": "http://schema.org/InStock",
                "seller": {"@type": "Organization", "name": "Shopify"}
            }
        })
    }

    #[test]
    fn every_island_type_deserializes() {
        for island in [
            organization(),
            product_group(),
            native_product(),
            reviewed_product(),
        ] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
    }

    #[test]
    fn the_two_product_islands_are_told_apart() {
        assert!(matches!(
            serde_json::from_value::<LdJson>(native_product()),
            Ok(LdJson::Product(Product::Native(_)))
        ));
        assert!(matches!(
            serde_json::from_value::<LdJson>(reviewed_product()),
            Ok(LdJson::Product(Product::Reviewed(_)))
        ));
    }

    #[test]
    fn a_reviewed_product_without_reviews_has_no_rating() {
        let mut island = reviewed_product();
        island["review"] = json!([]);
        island
            .as_object_mut()
            .expect("an object")
            .remove("aggregateRating");

        assert!(matches!(
            serde_json::from_value::<LdJson>(island),
            Ok(LdJson::Product(Product::Reviewed(product))) if product.aggregate_rating.is_none()
        ));
    }

    #[test]
    fn an_unseen_island_type_fails() {
        for island_type in ["BreadcrumbList", "FAQPage"] {
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
        let mut group = product_group();
        group["gtin"] = json!("1");
        assert!(serde_json::from_value::<LdJson>(group).is_err());

        let mut reviewed = reviewed_product();
        reviewed["gtin"] = json!("1");
        let error = serde_json::from_value::<LdJson>(reviewed).expect_err("an unknown key");
        assert!(error.to_string().contains("unknown field `gtin`"));
    }
}
