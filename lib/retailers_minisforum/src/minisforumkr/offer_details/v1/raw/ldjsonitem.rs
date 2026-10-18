/// A `script[type="application/ld+json"]`, its attributes alongside its parsed body.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: LdJsonItem,
}

/// Every page carries the `Organization`; a product page adds a `ProductGroup`, or a `Product`
/// where the product has a single variant and no options.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type", deny_unknown_fields)]
pub enum LdJsonItem {
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
    pub has_variant: Vec<VariantProduct>,
    pub name: String,
    /// The product's numeric id.
    #[serde(rename = "productGroupID")]
    pub product_group_id: String,
    pub url: String,
}

/// A `hasVariant` entry: one variant, as a `Product`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantProduct {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub item_type: String,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    /// Missing where the variant has no SKU.
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
    /// A schema.org `ItemAvailability` URL.
    pub availability: String,
    /// In major units of `priceCurrency`, as a string.
    pub price: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// The product URL with the variant's `?variant=<id>`.
    pub url: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::LdJsonItem;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "name": "Minisforum KR",
            "logo": "https://www.minisforum.kr/cdn/shop/files/logo.avif?v=1751943160&width=500",
            "url": "https://www.minisforum.kr"
        })
    }

    fn variant_product(id: u64, price: &str, sku: &str, availability: &str) -> Value {
        json!({
            "@id": format!("/products/minisforum-um890-pro?variant={id}#variant"),
            "@type": "Product",
            "image": "https://www.minisforum.kr/cdn/shop/files/UM890Pro1.jpg?v=1776075578&width=1920",
            "name": format!("MINISFORUM UM890 Pro 고성능 미니PC - {sku}"),
            "offers": {
                "@id": format!("/products/minisforum-um890-pro?variant={id}#offer"),
                "@type": "Offer",
                "availability": availability,
                "price": price,
                "priceCurrency": "KRW",
                "url": format!("https://www.minisforum.kr/products/minisforum-um890-pro?variant={id}")
            },
            "sku": sku
        })
    }

    pub(crate) fn product_group() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/minisforum-um890-pro#product",
            "@type": "ProductGroup",
            "brand": {"@type": "Brand", "name": "Minisforum KR"},
            "category": "데스크톱 컴퓨터",
            "description": "",
            "hasVariant": [
                variant_product(41332824080430, "756900", "UM890Pro00EU", "http://schema.org/InStock"),
                variant_product(41332824113198, "1071992", "UM890Pro32EU", "http://schema.org/OutOfStock")
            ],
            "name": "MINISFORUM UM890 Pro 고성능 미니PC",
            "productGroupID": "7128100143150",
            "url": "https://www.minisforum.kr/products/minisforum-um890-pro"
        })
    }

    pub(crate) fn product() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/u-shaped-neck-pillow#product",
            "@type": "Product",
            "brand": {"@type": "Brand", "name": "Minisforum KR"},
            "category": "노트북 가방",
            "description": "",
            "image": "https://www.minisforum.kr/cdn/shop/files/U.jpg?v=1763457715&width=1920",
            "name": "MINISFORUM U자형 목베개",
            "offers": {
                "@id": "/products/u-shaped-neck-pillow?variant=41622560604206#offer",
                "@type": "Offer",
                "availability": "http://schema.org/OutOfStock",
                "price": "68000",
                "priceCurrency": "KRW",
                "url": "https://www.minisforum.kr/products/u-shaped-neck-pillow?variant=41622560604206"
            },
            "sku": "U-SHAPEDP",
            "url": "https://www.minisforum.kr/products/u-shaped-neck-pillow"
        })
    }

    #[test]
    fn each_item_reads_by_its_type() {
        for (item, expected) in [
            (organization(), "Organization"),
            (product_group(), "ProductGroup"),
            (product(), "Product"),
        ] {
            let parsed = serde_json::from_value::<LdJsonItem>(item).expect("deserializes");
            let name = match parsed {
                LdJsonItem::Organization(_) => "Organization",
                LdJsonItem::ProductGroup(_) => "ProductGroup",
                LdJsonItem::Product(_) => "Product",
            };
            assert_eq!(name, expected);
        }
    }

    #[test]
    fn an_unseen_type_fails() {
        let error = serde_json::from_value::<LdJsonItem>(json!({"@type": "BreadcrumbList"}))
            .expect_err("no breadcrumbs in the corpus");
        assert!(
            error
                .to_string()
                .contains("unknown variant `BreadcrumbList`")
        );
    }

    #[test]
    fn a_variant_may_lack_a_sku() {
        let mut group = product_group();
        group["hasVariant"][0]
            .as_object_mut()
            .expect("an object")
            .remove("sku");

        serde_json::from_value::<LdJsonItem>(group).expect("one variant in the corpus has none");
    }
}
