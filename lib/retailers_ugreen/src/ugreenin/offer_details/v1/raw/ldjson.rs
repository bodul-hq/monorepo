use html_extractor::serde_json::Value;

/// A `script[type="application/ld+json"]`: its attributes and its parsed body.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    /// `jdgm-server-jld` on Judge.me's reviews island; absent on the theme's.
    pub class: Option<String>,
    pub data: LdJson,
}

/// The island types seen in the corpus: `Organization` and `BreadcrumbList` on every page,
/// `FAQPage` on 681 pages per locale, and the product as a `Product` (701 per locale) or a
/// `ProductGroup` (3 per locale).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Organization(Organization),
    BreadcrumbList(BreadcrumbList),
    #[serde(rename = "FAQPage")]
    FaqPage(FaqPage),
    ProductGroup(ProductGroup),
    Product(ProductIsland),
}

/// Two islands on a page say `Product`: the theme's, with the offer, on every page that does
/// not render a `ProductGroup`, and Judge.me's, with the reviews, on the 246 pages per locale
/// that have any. Neither carries the other's keys.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum ProductIsland {
    Offers(Box<Product>),
    Reviews(Box<ReviewsProduct>),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    /// The page's path with a `#product` fragment.
    #[serde(rename = "@id")]
    pub id: String,
    pub brand: Brand,
    /// Google's product category, in English on both locales.
    pub category: String,
    pub description: String,
    /// The variant's barcode, mangled values included. Absent where the barcode is null.
    pub gtin: Option<String>,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    pub sku: String,
    /// The page's own URL: under `/hi` on the Hindi pages, its handle percent-encoded.
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
    pub description: String,
    #[serde(rename = "hasVariant")]
    pub has_variant: Vec<GroupVariant>,
    pub name: String,
    /// The product's numeric id.
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
    /// Absent where the variant's barcode is null.
    pub gtin: Option<String>,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    pub sku: String,
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
    /// A decimal string in rupees, such as `999.00`.
    pub price: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// The page's URL with a `variant` query parameter.
    pub url: String,
}

/// Judge.me's island. Only its keys are typed: the ratings and reviews carry no offer fact.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewsProduct {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub name: String,
    pub image: String,
    #[serde(rename = "aggregateRating")]
    pub aggregate_rating: Value,
    pub review: Value,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    #[serde(rename = "legalName")]
    pub legal_name: String,
    pub url: String,
    pub logo: String,
    pub address: PostalAddress,
    #[serde(rename = "contactPoint")]
    pub contact_point: ContactPoint,
    #[serde(rename = "sameAs")]
    pub same_as: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PostalAddress {
    #[serde(rename = "@type")]
    pub address_type: String,
    pub street_address: String,
    pub address_locality: String,
    pub address_region: String,
    pub postal_code: String,
    pub address_country: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContactPoint {
    #[serde(rename = "@type")]
    pub contact_point_type: String,
    pub contact_type: String,
    pub email: String,
    pub telephone: String,
    pub area_served: String,
    pub available_language: Vec<String>,
}

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

#[cfg(test)]
pub(crate) mod tests {
    use super::{LdJson, LdJsonScript, ProductIsland};
    use html_extractor::serde_json::{self, Value, json};

    const URL: &str = "https://www.ugreenindia.com/products/ugreen-ergonomic-mouse-pad-black-25245";
    const IMAGE: &str =
        "https://www.ugreenindia.com/cdn/shop/files/LP668-Black.png?v=1763832672&width=1920";

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "address": {
                "@type": "PostalAddress",
                "addressCountry": "IN",
                "addressLocality": "Bengaluru",
                "addressRegion": "Karnataka",
                "postalCode": "560092",
                "streetAddress": "Durga Complex, 2nd Floor, #38/1, 3rd Cross, Sahakarnagar G Block"
            },
            "contactPoint": {
                "@type": "ContactPoint",
                "areaServed": "IN",
                "availableLanguage": ["English", "Hindi"],
                "contactType": "customer service",
                "email": "info@ugreenindia.com",
                "telephone": "+91-94038-93363"
            },
            "legalName": "Sophus Technosol Private Limited",
            "logo": "https://www.ugreenindia.com/cdn/shop/files/logo-16710693681.jpg?v=1718373793&width=900",
            "name": "Ugreen India",
            "sameAs": ["https://www.youtube.com/c/UgreenOfficial"],
            "url": "https://www.ugreenindia.com"
        })
    }

    pub(crate) fn breadcrumbs() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "BreadcrumbList",
            "itemListElement": [
                {"@type": "ListItem", "item": "https://www.ugreenindia.com/", "name": "Home", "position": 1},
                {"@type": "ListItem", "item": URL, "name": "UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest", "position": 2}
            ]
        })
    }

    pub(crate) fn faq() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "FAQPage",
            "mainEntity": [{
                "@type": "Question",
                "name": "Is it washable?",
                "acceptedAnswer": {"@type": "Answer", "text": "Yes."}
            }]
        })
    }

    pub(crate) fn offer(id: u64, availability: &str) -> Value {
        let path = "/products/ugreen-ergonomic-mouse-pad-black-25245";
        json!({
            "@id": format!("{path}?variant={id}#offer"),
            "@type": "Offer",
            "availability": format!("http://schema.org/{availability}"),
            "price": "999.00",
            "priceCurrency": "INR",
            "url": format!("{URL}?variant={id}")
        })
    }

    fn group_variant(id: u64, sku: &str, colour: &str, gtin: Option<&str>) -> Value {
        let mut variant = json!({
            "@id": format!("/products/ugreen-ergonomic-mouse-pad-black-25245?variant={id}#variant"),
            "@type": "Product",
            "image": IMAGE,
            "name": format!("UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest - {colour}"),
            "offers": offer(id, "InStock"),
            "sku": sku
        });
        if let Some(gtin) = gtin {
            variant["gtin"] = json!(gtin);
        }
        variant
    }

    /// The island of `super::productjson::tests::product()`.
    pub(crate) fn product_group() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/ugreen-ergonomic-mouse-pad-black-25245#product",
            "@type": "ProductGroup",
            "brand": {"@type": "Brand", "name": "UGREEN"},
            "category": "Mouse Pads",
            "description": "Product guide",
            "hasVariant": [
                group_variant(43454938677347, "25245", "Black", Some("6941876222452")),
                group_variant(43742608687203, "25243", "Green", None),
                group_variant(43742608719971, "25244", "Gray", None)
            ],
            "name": "UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest",
            "productGroupID": "7702424813667",
            "url": URL
        })
    }

    /// The single-variant `Product` island the theme renders for the group's first variant.
    pub(crate) fn product() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/ugreen-ergonomic-mouse-pad-black-25245#product",
            "@type": "Product",
            "brand": {"@type": "Brand", "name": "UGREEN"},
            "category": "Mouse Pads",
            "description": "Product guide",
            "gtin": "6941876222452",
            "image": IMAGE,
            "name": "UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest",
            "offers": offer(43454938677347, "InStock"),
            "sku": "25245",
            "url": URL
        })
    }

    pub(crate) fn reviews() -> Value {
        json!({
            "@context": "https://schema.org/",
            "@id": "/products/ugreen-ergonomic-mouse-pad-black-25245#product",
            "@type": "Product",
            "aggregateRating": {
                "@type": "AggregateRating", "bestRating": 5, "ratingValue": "5.00",
                "reviewCount": 1, "worstRating": 1
            },
            "image": "https://cdn.shopify.com/s/files/1/0581/3780/4899/files/LP668-Black.png?v=1763832672",
            "name": "UGREEN LP668 Ergonomic Mouse Pad with Wrist Rest",
            "review": [{
                "@type": "Review",
                "author": {"@type": "Person", "name": "Anonymous"},
                "datePublished": "2026-09-14T14:31:55Z",
                "reviewBody": "Go for it.",
                "reviewRating": {"@type": "Rating", "bestRating": 5, "ratingValue": 5, "worstRating": 1}
            }]
        })
    }

    #[test]
    fn every_island_type_deserializes() {
        for island in [
            organization(),
            breadcrumbs(),
            faq(),
            product_group(),
            product(),
            reviews(),
        ] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
    }

    #[test]
    fn the_two_product_islands_are_told_apart_by_their_keys() {
        let theme = serde_json::from_value::<LdJson>(product()).expect("deserializes");
        assert!(matches!(theme, LdJson::Product(ProductIsland::Offers(_))));

        let reviews = serde_json::from_value::<LdJson>(reviews()).expect("deserializes");
        assert!(matches!(
            reviews,
            LdJson::Product(ProductIsland::Reviews(_))
        ));
    }

    #[test]
    fn a_product_without_a_gtin_deserializes() {
        let mut island = product();
        island.as_object_mut().expect("an object").remove("gtin");

        let island = serde_json::from_value::<LdJson>(island).expect("deserializes");
        assert!(matches!(
            island,
            LdJson::Product(ProductIsland::Offers(product)) if product.gtin.is_none()
        ));
    }

    #[test]
    fn an_unseen_island_type_fails() {
        for island_type in ["WebSite", "WebPage", "ItemList"] {
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
        for mut island in [organization(), faq(), product_group(), product(), reviews()] {
            island["mpn"] = json!("1");

            assert!(
                serde_json::from_value::<LdJson>(island.clone()).is_err(),
                "{island}"
            );
        }
    }

    #[test]
    fn an_unexpected_script_attribute_fails() {
        let script = json!({"type": "application/ld+json", "data": organization()});
        serde_json::from_value::<LdJsonScript>(script.clone()).expect("deserializes");

        let mut with_id = script;
        with_id["id"] = json!("organization");
        assert!(serde_json::from_value::<LdJsonScript>(with_id).is_err());
    }
}
