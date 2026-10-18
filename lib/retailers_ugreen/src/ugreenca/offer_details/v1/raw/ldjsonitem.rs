/// One `script[type="application/ld+json"]`. Every page renders the same four islands.
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
    Organization(Organization),
    WebPage(WebPage),
    WebSite(WebSite),
    Product(Box<Product>),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    #[serde(rename = "contactPoint")]
    pub contact_point: ContactPoint,
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
    pub description: String,
    pub url: String,
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    pub image: Vec<String>,
    pub description: String,
    /// The selected variant's.
    pub sku: String,
    /// The selected variant's barcode, or empty.
    pub mpn: String,
    /// The product id, as a string.
    #[serde(rename = "productID")]
    pub product_id: String,
    pub brand: Brand,
    pub offers: Offers,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

/// One offer object on most pages; a list of one per variant on some multi-variant pages.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum Offers {
    One(Box<Offer>),
    Many(Vec<Offer>),
}

impl Offers {
    pub fn into_vec(self) -> Vec<Offer> {
        match self {
            Offers::One(offer) => vec![*offer],
            Offers::Many(offers) => offers,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    #[serde(rename = "@type")]
    pub kind: String,
    pub sku: String,
    pub availability: String,
    /// A decimal in major units, such as `16.99`.
    pub price: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    #[serde(rename = "itemCondition")]
    pub item_condition: String,
    pub url: String,
    pub mpn: String,
    /// Empty where present on a single offer; the barcode on some offer lists; often absent.
    pub gtin13: Option<String>,
    #[serde(rename = "priceValidUntil")]
    pub price_valid_until: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{LdJsonItem, Offers};
    use html_extractor::serde_json::{self, Value, json};

    fn offer() -> Value {
        json!({"@type": "Offer", "sku": "40737", "availability": "http://schema.org/InStock",
            "price": "59.99", "priceCurrency": "CAD",
            "itemCondition": "https://schema.org/NewCondition",
            "url": "https://ca.ugreen.com/products/100w-3c1a-gan-fast-charger",
            "mpn": "6957303813346", "gtin13": "", "priceValidUntil": "2027-10-14"})
    }

    pub(crate) fn islands() -> Value {
        let island = |data: Value| json!({"type": "application/ld+json", "data": data});
        json!([
            island(
                json!({"@context": "https://schema.org", "@type": "Organization",
                "name": "UGREEN CA",
                "url": "https://ca.ugreen.com/products/100w-3c1a-gan-fast-charger",
                "contactPoint": {"@type": "ContactPoint", "email": "service@ugreen.com"},
                "sameAs": ["https://www.facebook.com/UgreenGlobal"]})
            ),
            island(json!({"@context": "https://schema.org", "@type": "WebPage",
                "name": "UGREEN Nexode 100W USB C Charger", "description": "d",
                "url": "https://ca.ugreen.com/products/100w-3c1a-gan-fast-charger"})),
            island(json!({"@context": "https://schema.org", "@type": "WebSite",
                "name": "UGREEN Nexode 100W USB C Charger", "url": "https://ca.ugreen.com",
                "potentialAction": {"@type": "SearchAction",
                    "target": "https://ca.ugreen.com?/search?q={search_term_string}",
                    "query-input": "required name=search_term_string"}})),
            island(json!({"@context": "http://schema.org/", "@type": "Product",
                "name": "UGREEN Nexode 100W USB C Charger ",
                "url": "https://ca.ugreen.com/products/100w-3c1a-gan-fast-charger",
                "image": ["https://ca.ugreen.com/cdn/shop/files/charger.png?v=1746739856&width=600"],
                "description": "Fast GaN charger", "sku": "40737", "mpn": "6957303813346",
                "productID": "8006139642144",
                "brand": {"@type": "Brand", "name": "UGREEN STORE"},
                "offers": offer()}))
        ])
    }

    #[test]
    fn the_four_islands_deserialize() {
        let items = serde_json::from_value::<Vec<LdJsonItem>>(islands()).expect("deserializes");
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn offers_are_one_object_or_a_list() {
        let one = serde_json::from_value::<Offers>(offer()).expect("one offer");
        let many = serde_json::from_value::<Offers>(json!([offer(), offer()])).expect("a list");

        assert_eq!(one.into_vec().len(), 1);
        assert_eq!(many.into_vec().len(), 2);
    }

    #[test]
    fn an_unknown_island_type_fails() {
        let item = json!({"type": "application/ld+json",
            "data": {"@type": "BreadcrumbList", "itemListElement": []}});
        assert!(serde_json::from_value::<LdJsonItem>(item).is_err());
    }
}
