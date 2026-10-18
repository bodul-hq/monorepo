#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: LdJson,
}

/// The island types seen in the corpus. A multi-variant product renders a `ProductGroup`, a
/// single-variant one a `Product`, and a product without a price neither.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Organization(Organization),
    ProductGroup(ProductGroup),
    Product(Product),
    #[serde(rename = "FAQPage")]
    FaqPage(FaqPage),
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
    /// Absent where the variant has no SKU.
    pub sku: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
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
    /// A decimal string in major units, such as `639.00`.
    pub price: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// The product URL with a `variant` query parameter.
    pub url: String,
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
    use super::LdJson;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "name": "Minisforum UK",
            "logo": "https://www.minisforum.uk/cdn/shop/files/Logo-Minisforum.svg?v=1785924046&width=500",
            "url": "https://www.minisforum.uk"
        })
    }

    fn group_variant(id: u64, price: &str, sku: &str, memory: &str) -> Value {
        json!({
            "@id": format!("/products/ai-x1-pro?variant={id}#variant"),
            "@type": "Product",
            "image": "https://www.minisforum.uk/cdn/shop/files/ai-x1-pro-mini-pc-1.png?v=1787902570&width=1920",
            "name": format!("Minisforum AI X1 Pro-370 - AMD Ryzen™ AI 9 HX 370 / {memory} / UK Warehouse"),
            "offers": {
                "@id": format!("/products/ai-x1-pro?variant={id}#offer"),
                "@type": "Offer",
                "availability": "http://schema.org/InStock",
                "price": price,
                "priceCurrency": "GBP",
                "url": format!("https://www.minisforum.uk/products/ai-x1-pro?variant={id}")
            },
            "sku": sku
        })
    }

    /// The island of `super::productjson::tests::product()`.
    pub(crate) fn product_group() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/ai-x1-pro#product",
            "@type": "ProductGroup",
            "brand": {"@type": "Brand", "name": "Minisforum UK"},
            "category": "Desktop Computers",
            "description": "",
            "hasVariant": [
                group_variant(50279469515030, "639.00", "X1PRO00UK", "Barebone (Not include os license)"),
                group_variant(49777802674454, "1079.00", "X1PRO31UK", "32GB RAM + 1TB SSD")
            ],
            "name": "Minisforum AI X1 Pro-370",
            "productGroupID": "9875989692694",
            "url": "https://www.minisforum.uk/products/ai-x1-pro"
        })
    }

    #[test]
    fn every_island_type_deserializes() {
        let faq = json!({"@context": "https://schema.org", "@type": "FAQPage", "mainEntity": [
            {"@type": "Question", "name": "1. Where do you ship to?",
                "acceptedAnswer": {"@type": "Answer", "text": "<p>UK</p>"}}]});

        for island in [organization(), product_group(), faq] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
    }

    #[test]
    fn an_unseen_island_type_fails() {
        let error = serde_json::from_value::<LdJson>(json!({"@type": "BreadcrumbList"}))
            .expect_err("no breadcrumb island is typed");
        assert!(
            error
                .to_string()
                .contains("unknown variant `BreadcrumbList`")
        );
    }

    #[test]
    fn an_unknown_key_in_an_island_fails() {
        let mut island = product_group();
        island["gtin"] = json!("1");

        assert!(serde_json::from_value::<LdJson>(island).is_err());
    }
}
