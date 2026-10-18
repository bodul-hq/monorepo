#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: LdJson,
}

/// The island types seen in the corpus. A multi-variant product renders a `ProductGroup` (31
/// pages), a single-variant one a `Product` (3), and a product without a price neither (1).
/// `minisforumuk`'s `FAQPage` island appears on no saved page, so it is not typed.
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
    /// Absent where the variant has no SKU (1 of the 82 variants in a group).
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

#[cfg(test)]
pub(crate) mod tests {
    use super::LdJson;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "name": "Minisforum AU",
            "logo": "https://au.minisforum.com/cdn/shop/files/Logo.svg?v=1787638003&width=500",
            "url": "https://au.minisforum.com"
        })
    }

    fn group_variant(id: u64, price: &str, sku: &str, kind: &str) -> Value {
        json!({
            "@id": format!("/products/minisforum-ms-03-workstation?variant={id}#variant"),
            "@type": "Product",
            "image": "https://au.minisforum.com/cdn/shop/files/MinisforumMS-03.png?v=1784715204&width=1920",
            "name": format!("MINISFORUM MS-03 - Intel® Core™ Ultra 5 336H / {kind}"),
            "offers": {
                "@id": format!("/products/minisforum-ms-03-workstation?variant={id}#offer"),
                "@type": "Offer",
                "availability": "http://schema.org/InStock",
                "price": price,
                "priceCurrency": "AUD",
                "url": format!("https://au.minisforum.com/products/minisforum-ms-03-workstation?variant={id}")
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
            "brand": {"@type": "Brand", "name": "Au Minisforum"},
            "category": "Desktop Computers",
            "description": "",
            "hasVariant": [
                group_variant(45188354441331, "1142.90", "MS-03-33600AU", "Barebone"),
                group_variant(45270950707315, "2022.90", "MS-03-33631AU", "32GB RAM+1TB SSD")
            ],
            "name": "MINISFORUM MS-03",
            "productGroupID": "8313879462003",
            "url": "https://au.minisforum.com/products/minisforum-ms-03-workstation"
        })
    }

    #[test]
    fn every_island_type_deserializes() {
        let group = product_group();
        let product = json!({
            "@context": "http://schema.org/",
            "@id": "/products/minisforum-ms-03-workstation#product",
            "@type": "Product",
            "brand": group["brand"],
            "category": "Desktop Computers",
            "description": "",
            "image": group["hasVariant"][0]["image"],
            "name": group["name"],
            "offers": group["hasVariant"][0]["offers"],
            "sku": "MS-03-33600AU",
            "url": group["url"]
        });

        for island in [organization(), group, product] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
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
        let mut island = product_group();
        island["gtin"] = json!("1");

        assert!(serde_json::from_value::<LdJson>(island).is_err());
    }
}
