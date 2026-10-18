use html_extractor::serde_json::Value;

/// A `script[type="application/ld+json"]`: its attributes and its parsed body.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    /// `jdgm-server-jld` on Judge.me's review island (29 pages); absent on the theme's.
    pub class: Option<String>,
    pub data: LdJson,
}

/// The island types seen in the corpus: the `Organization` on every page, the theme's
/// `Product` (262 pages) or `ProductGroup` (44), and Judge.me's review `Product` (29).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Organization(Organization),
    Product(ProductIsland),
    ProductGroup(ProductGroup),
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

/// Two islands on a page say `Product`: the theme's, with the offer, and Judge.me's, with the
/// reviews. Neither carries the other's required keys (`offers`, `aggregateRating`), so the
/// shape tells them apart; normalization checks it agrees with the script's `class`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum ProductIsland {
    Offer(Box<Product>),
    Reviews(Box<ReviewsProduct>),
}

/// The theme's island for a product without options.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub brand: Brand,
    /// Empty on some pages.
    pub category: String,
    pub description: String,
    /// Missing where the variant has no barcode. Some values end in a no-break space.
    pub gtin: Option<String>,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    pub sku: String,
    pub url: String,
}

/// The theme's island for a product with options: one `Product` per variant.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductGroup {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub brand: Brand,
    /// Empty on some pages.
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
    /// Missing where the variant has no barcode. Some values end in a no-break space.
    pub gtin: Option<String>,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    /// Missing on the gift cards' 3 variants, the only ones without a SKU.
    pub sku: Option<String>,
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
    /// `http://schema.org/InStock` or `http://schema.org/OutOfStock`.
    pub availability: String,
    /// A decimal string in major units, such as `134.99`: the Shopify variant price, before any
    /// automatic discount.
    pub price: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// The product URL with a `variant` query parameter: the only place the island names the
    /// variant's id.
    pub url: String,
}

/// Judge.me's review island. Recognised so it can't be mistaken for an unknown island, and
/// otherwise ignored: it carries no offer.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewsProduct {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub name: String,
    /// The rating summary: no offer fact, so not typed.
    #[serde(rename = "aggregateRating")]
    pub aggregate_rating: Value,
    pub image: String,
    /// Up to the latest reviews, with their bodies: no offer fact, so not typed.
    pub review: Value,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{LdJson, LdJsonScript, ProductIsland};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "name": "UGREEN FR",
            "logo": "https://fr.ugreen.com/cdn/shop/files/ugreen_logo.svg?v=1775564873&width=500",
            "url": "https://fr.ugreen.com"
        })
    }

    fn group_variant(id: u64, price: &str, sku: &str, title: &str) -> Value {
        json!({
            "@id": format!("/products/45699?variant={id}#variant"),
            "@type": "Product",
            "image": "https://fr.ugreen.com/cdn/shop/files/charger.png?v=1738861172&width=1920",
            "name": format!("UGREEN 300W Nexode Chargeur - {title}"),
            "offers": {
                "@id": format!("/products/45699?variant={id}#offer"),
                "@type": "Offer",
                "availability": "http://schema.org/InStock",
                "price": price,
                "priceCurrency": "EUR",
                "url": format!("https://fr.ugreen.com/products/45699?variant={id}")
            },
            "sku": sku
        })
    }

    /// The island of `super::super::analytics::tests::meta()`.
    pub(crate) fn product_group() -> Value {
        let mut first = group_variant(50354801180992, "134.99", "45699", "Chargeur");
        first["gtin"] = json!("6941876246991");
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/45699#product",
            "@type": "ProductGroup",
            "brand": {"@type": "Brand", "name": "UGREEN FR"},
            "category": "Adaptateurs secteur et chargeurs",
            "description": " \n  ",
            "hasVariant": [
                first,
                group_variant(50354801213760, "187.99", "45699+70427P", "+ 2 Câble USB C 100W 1M")
            ],
            "name": "UGREEN 300W Nexode Chargeur",
            "productGroupID": "9526794977600",
            "url": "https://fr.ugreen.com/products/45699"
        })
    }

    /// The theme's island of `products-15532.html`.
    pub(crate) fn product() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/15532#product",
            "@type": "Product",
            "brand": {"@type": "Brand", "name": "UGREEN"},
            "category": "USB-C HUB",
            "description": " \n  ",
            "gtin": "6938638046609",
            "image": "https://fr.ugreen.com/cdn/shop/files/hub.png?v=1746741214&width=1920",
            "name": "UGREEN Revodok Pro 109 Hub USB C",
            "offers": {
                "@id": "/products/15532?variant=47543757046080#offer",
                "@type": "Offer",
                "availability": "http://schema.org/InStock",
                "price": "61.99",
                "priceCurrency": "EUR",
                "url": "https://fr.ugreen.com/products/15532?variant=47543757046080"
            },
            "sku": "15532",
            "url": "https://fr.ugreen.com/products/15532"
        })
    }

    /// Judge.me's island, with one review.
    pub(crate) fn reviews(handle: &str) -> Value {
        json!({
            "@context": "https://schema.org/",
            "@type": "Product",
            "@id": format!("/products/{handle}#product"),
            "name": "UGREEN",
            "aggregateRating": {
                "@type": "AggregateRating", "ratingValue": "4.80", "reviewCount": 44,
                "bestRating": 5, "worstRating": 1
            },
            "image": "https://cdn.shopify.com/s/files/1/0726/4111/8528/files/a.png?v=1746741214",
            "review": [{
                "@type": "Review",
                "reviewRating": {"@type": "Rating", "ratingValue": 5, "bestRating": 5, "worstRating": 1},
                "author": {"@type": "Person", "name": "Bperd"},
                "reviewBody": "Tout est parfait et discret !",
                "datePublished": "2024-03-28T16:00:00Z"
            }]
        })
    }

    #[test]
    fn every_island_type_deserializes() {
        for island in [organization(), product_group(), product(), reviews("15532")] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
    }

    #[test]
    fn the_two_product_islands_are_told_apart_by_shape() {
        let theme = serde_json::from_value::<LdJson>(product()).expect("deserializes");
        assert!(matches!(theme, LdJson::Product(ProductIsland::Offer(_))));

        let reviews = serde_json::from_value::<LdJson>(reviews("15532")).expect("deserializes");
        assert!(matches!(
            reviews,
            LdJson::Product(ProductIsland::Reviews(_))
        ));
    }

    #[test]
    fn the_script_class_is_read() {
        let script: LdJsonScript = serde_json::from_value(json!({
            "class": "jdgm-server-jld",
            "type": "application/ld+json",
            "data": reviews("15532")
        }))
        .expect("deserializes");
        assert_eq!(script.class.as_deref(), Some("jdgm-server-jld"));
    }

    #[test]
    fn a_variant_without_a_sku_or_gtin_deserializes() {
        let mut island = product_group();
        let variant = island["hasVariant"][0].as_object_mut().expect("a variant");
        variant.remove("sku");
        variant.remove("gtin");

        serde_json::from_value::<LdJson>(island).expect("deserializes");
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
        let mut group = product_group();
        group["gtin"] = json!("1");
        assert!(serde_json::from_value::<LdJson>(group).is_err());

        let mut product = product();
        product["mpn"] = json!("1");
        assert!(serde_json::from_value::<LdJson>(product).is_err());

        let mut reviews = reviews("15532");
        reviews["offers"] = json!({});
        assert!(serde_json::from_value::<LdJson>(reviews).is_err());
    }

    #[test]
    fn an_unexpected_script_attribute_fails() {
        let input = json!({
            "id": "x",
            "type": "application/ld+json",
            "data": organization()
        });

        let error = serde_json::from_value::<LdJsonScript>(input).expect_err("not in the corpus");
        assert!(error.to_string().contains("unknown field `id`"));
    }
}
