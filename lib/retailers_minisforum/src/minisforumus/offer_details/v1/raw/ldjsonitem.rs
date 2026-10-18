//! The theme's own `ld+json` scripts. Review apps add more, with an `id` or a `class` on the tag;
//! the architecture leaves those out.

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonItem {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: LdJson,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Product(Product),
    Organization(Organization),
    BreadcrumbList(BreadcrumbList),
    WebSite(WebSite),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    /// Plain text; empty on some products.
    pub description: String,
    pub url: String,
    pub image: String,
    pub additional_image: Option<Vec<String>>,
    pub brand: Brand,
    pub category: String,
    pub keywords: String,
    pub manufacturer: Manufacturer,
    pub mpn: Option<String>,
    pub sku: Option<String>,
    pub offers: AggregateOffer,
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
pub struct Manufacturer {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
    pub url: String,
}

/// Prices are decimal strings in `price_currency`, the market's currency. `low_price` and
/// `high_price` do not always span every variant, so nothing reads them as the variants' range.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AggregateOffer {
    #[serde(rename = "@type")]
    pub kind: String,
    pub availability: String,
    pub high_price: String,
    pub low_price: String,
    pub item_condition: String,
    pub offer_count: u64,
    pub price_currency: String,
    pub seller: Seller,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seller {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

/// Rendered twice: once in full, once as just a name and an address.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    pub alternate_name: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub founding_date: Option<String>,
    pub brand: Option<Brand>,
    pub same_as: Option<Vec<String>>,
    pub contact_point: Option<ContactPoint>,
    pub area_served: Option<Vec<Country>>,
    pub knows_about: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContactPoint {
    #[serde(rename = "@type")]
    pub kind: String,
    pub email: String,
    pub contact_type: String,
    pub available_language: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Country {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BreadcrumbList {
    #[serde(rename = "@context")]
    pub context: String,
    pub item_list_element: Vec<BreadcrumbItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BreadcrumbItem {
    #[serde(rename = "@type")]
    pub kind: String,
    pub position: i64,
    pub name: String,
    pub item: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WebSite {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    pub potential_action: SearchAction,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchAction {
    #[serde(rename = "@type")]
    pub kind: String,
    pub target: EntryPoint,
    #[serde(rename = "query-input")]
    pub query_input: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntryPoint {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "urlTemplate")]
    pub url_template: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::LdJson;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn product() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "Product",
            "name": "MINISFORUM MS-01 SE",
            "description": "MINISFORUM MS-01 SE Workstation",
            "url": "https://store.minisforum.com/en-ca/products/minisforum-ms-01-se-workstation",
            "image": "https://store.minisforum.com/cdn/shop/files/MS-01.png?v=1&width=1200",
            "additionalImage": [
                "https://store.minisforum.com/cdn/shop/files/MS-01-2.png?v=1&width=1200"
            ],
            "brand": {"@type": "Brand", "name": "MINISFORUM"},
            "category": "Work Station",
            "keywords": "active, All, Intel",
            "manufacturer": {"@type": "Organization", "name": "MINISFORUM",
                "url": "https://store.minisforum.com"},
            "mpn": "MS-01SE00US",
            "sku": "MS-01SE00US",
            "offers": {
                "@type": "AggregateOffer",
                "availability": "https://schema.org/InStock",
                "highPrice": "1123.08",
                "lowPrice": "1123.08",
                "itemCondition": "https://schema.org/NewCondition",
                "offerCount": 2,
                "priceCurrency": "CAD",
                "seller": {"@type": "Organization", "name": "MINISFORUM"},
                "url": "https://store.minisforum.com/en-ca/products/minisforum-ms-01-se-workstation"
            }
        })
    }

    pub(crate) fn organization() -> Value {
        json!({"@context": "https://schema.org", "@type": "Organization", "name": "Minisforum",
            "url": "https://store.minisforum.com"})
    }

    pub(crate) fn breadcrumbs() -> Value {
        json!({"@context": "https://schema.org", "@type": "BreadcrumbList",
            "itemListElement": [{"@type": "ListItem", "position": 1, "name": "Minisforum",
                "item": "https://store.minisforum.com"}]})
    }

    pub(crate) fn website() -> Value {
        json!({"@context": "https://schema.org", "@type": "WebSite", "name": "Minisforum",
            "url": "https://store.minisforum.com",
            "potentialAction": {"@type": "SearchAction",
                "target": {"@type": "EntryPoint",
                    "urlTemplate": "https://store.minisforum.com/search?q={search_term_string}"},
                "query-input": "required name=search_term_string"}})
    }

    #[test]
    fn every_theme_island_deserializes() {
        for island in [product(), organization(), breadcrumbs(), website()] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
    }

    #[test]
    fn an_unseen_type_fails() {
        let error = serde_json::from_value::<LdJson>(json!({"@type": "FAQPage"}))
            .expect_err("no FAQPage is typed");
        assert!(error.to_string().contains("unknown variant `FAQPage`"));
    }

    #[test]
    fn an_unseen_product_key_fails() {
        let mut island = product();
        island["gtin13"] = json!("1");

        assert!(serde_json::from_value::<LdJson>(island).is_err());
    }
}
