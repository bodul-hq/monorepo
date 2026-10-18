/// One `script[type="application/ld+json"]`. Every page renders two: the shop's `Organization`,
/// then the product.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: LdJson,
}

/// The island types seen in the corpus. A single-variant product renders a `Product` (10,639
/// pages), a multi-variant one a `ProductGroup` (1,848 pages, 329 of them with one variant).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Organization(Organization),
    Product(Box<Product>),
    ProductGroup(Box<ProductGroup>),
}

/// The shop: `UGREEN EU` on `https://eu.ugreen.com` on every page.
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
    /// The Google product category, translated per locale. Empty on some pages.
    pub category: String,
    /// The product description as text, wrapped in the theme's carousel labels and whitespace.
    pub description: String,
    /// In the analytics' order, up to 100 per group.
    #[serde(rename = "hasVariant")]
    pub has_variant: Vec<GroupVariant>,
    /// On 18 French pages the name ends in a space.
    pub name: String,
    /// The Shopify product id, as a string.
    #[serde(rename = "productGroupID")]
    pub product_group_id: String,
    /// The page's canonical URL, up to the case of its percent escapes: on the 143 pages with a
    /// non-ASCII handle the island escapes in upper case (`%D0%BA`), the `link[rel=canonical]`
    /// in lower case (`%d0%ba`).
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupVariant {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub variant_type: String,
    /// Missing on 8,340 of the 11,416 group variants.
    #[serde(default)]
    pub gtin: Option<String>,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    pub sku: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub brand: Brand,
    /// As on `ProductGroup`.
    pub category: String,
    /// As on `ProductGroup`.
    pub description: String,
    /// Missing on 2,359 of the 10,639 single-variant pages.
    #[serde(default)]
    pub gtin: Option<String>,
    pub image: String,
    /// As on `ProductGroup`.
    pub name: String,
    pub offers: Offer,
    pub sku: String,
    /// As on `ProductGroup`.
    pub url: String,
}

/// The product's vendor again: `UGREEN EU` on most pages, but also `UGREEN ES`, `UGREEN UK`,
/// `Seel` and others.
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
    /// A dot-decimal string in major units, such as `55.99`: the Shopify variant price, before
    /// any automatic discount.
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
            "name": "UGREEN EU",
            "logo": "https://eu.ugreen.com/cdn/shop/files/ugreen_logo-_1_e168f1f4-de2e-4509-bd2e-8ea001ac3a12.svg?v=1774576532&width=500",
            "url": "https://eu.ugreen.com"
        })
    }

    fn group_variant(id: u64, price: &str, sku: &str, colour: &str, image: &str) -> Value {
        json!({
            "@id": format!("/de-at/products/pb561?variant={id}#variant"),
            "@type": "Product",
            "image": format!("https://eu.ugreen.com/cdn/shop/{image}?v=1719994480&width=1920"),
            "name": format!("UGREEN Kabelloses Magnetisches Powerbank 10000mAh - {colour}"),
            "offers": {
                "@id": format!("/de-at/products/pb561?variant={id}#offer"),
                "@type": "Offer",
                "availability": "http://schema.org/InStock",
                "price": price,
                "priceCurrency": "EUR",
                "url": format!("https://eu.ugreen.com/de-at/products/pb561?variant={id}")
            },
            "sku": sku
        })
    }

    /// The island of `de-at-products-pb561.html`, its description shortened. Only the first
    /// variant carries a GTIN.
    pub(crate) fn product_group() -> Value {
        let mut first = group_variant(
            48981450031452,
            "55.99",
            "25919",
            "Schwarz",
            "files/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-451511.png",
        );
        first["gtin"] = json!("6941876229192");
        json!({
            "@context": "http://schema.org/",
            "@id": "/de-at/products/pb561#product",
            "@type": "ProductGroup",
            "brand": {"@type": "Brand", "name": "UGREEN EU"},
            "category": "Powerbanks",
            "description": "\n\n\nVorherige Seite\n\nNächste Seite\n\n",
            "hasVariant": [
                first,
                group_variant(
                    48981449998684,
                    "49.99",
                    "25208",
                    "Weiß",
                    "products/ugreen-wireless-powerbank-10000mah-magnetische-power-bank-685866.png",
                )
            ],
            "name": "UGREEN Kabelloses Magnetisches Powerbank 10000mAh",
            "productGroupID": "8625471816028",
            "url": "https://eu.ugreen.com/de-at/products/pb561"
        })
    }

    /// The group's first variant as a single `Product` island.
    pub(crate) fn product() -> Value {
        let group = product_group();
        let variant = &group["hasVariant"][0];
        json!({
            "@context": "http://schema.org/",
            "@id": "/de-at/products/pb561#product",
            "@type": "Product",
            "brand": group["brand"],
            "category": group["category"],
            "description": group["description"],
            "gtin": variant["gtin"],
            "image": variant["image"],
            "name": group["name"],
            "offers": variant["offers"],
            "sku": variant["sku"],
            "url": group["url"]
        })
    }

    #[test]
    fn every_island_type_deserializes() {
        for island in [organization(), product_group(), product()] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
    }

    #[test]
    fn a_missing_gtin_reads_none() {
        let mut island = product();
        island.as_object_mut().expect("an object").remove("gtin");

        let Ok(LdJson::Product(product)) = serde_json::from_value::<LdJson>(island) else {
            panic!("a Product island");
        };
        assert_eq!(product.gtin, None);
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
        group["mpn"] = json!("1");
        assert!(serde_json::from_value::<LdJson>(group).is_err());

        let mut offer = product();
        offer["offers"]["itemCondition"] = json!("http://schema.org/NewCondition");
        assert!(serde_json::from_value::<LdJson>(offer).is_err());
    }
}
