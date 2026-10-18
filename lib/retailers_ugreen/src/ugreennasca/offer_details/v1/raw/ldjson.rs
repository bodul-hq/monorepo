#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: LdJson,
}

/// The island types seen in the corpus. Every page carries the shop's `Organization` twice, a
/// `WebPage` and a `WebSite`; the theme renders no product island, and the one `Product` is the
/// stock Dawn template's, on the `worry-free-purchase` add-on.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Organization(Organization),
    WebPage(WebPage),
    WebSite(WebSite),
    Product(Product),
}

/// Rendered twice per page: by the theme (with `contactPoint`) and by Dawn's header.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    pub logo: String,
    /// Only on the theme's copy.
    #[serde(rename = "contactPoint", default)]
    pub contact_point: Option<Vec<ContactPoint>>,
    /// Profile URLs on the theme's copy, empty strings on Dawn's.
    #[serde(rename = "sameAs")]
    pub same_as: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactPoint {
    #[serde(rename = "@type")]
    pub contact_point_type: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebPage {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub description: WebPageDescription,
    pub url: String,
}

/// On every product page the theme renders the description with Liquid's `json` filter on an
/// object it refuses, so the island carries Liquid's error object instead of text.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum WebPageDescription {
    Text(String),
    LiquidError(LiquidError),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiquidError {
    pub error: String,
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
    pub action_type: String,
    pub target: String,
    #[serde(rename = "query-input")]
    pub query_input: String,
}

/// Dawn's stock product island, rendered only by the `worry-free-purchase` add-on's template.
/// Not read: normalization takes the add-on's facts from the same islands as any product's.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    pub image: Vec<String>,
    pub description: String,
    pub sku: String,
    pub brand: Brand,
    pub offers: Vec<Offer>,
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
    #[serde(rename = "@type")]
    pub offer_type: String,
    pub sku: String,
    pub availability: String,
    /// In major units, as a JSON number.
    pub price: f64,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// The product URL with a `variant` query parameter.
    pub url: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{LdJson, LdJsonScript, WebPageDescription};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "Organization",
            "name": "UGREEN AIoT CA",
            "url": "https://ai-ca.ugreen.com",
            "logo": "https://ai-ca.ugreen.com/cdn/shop/files/favicon_green.png?v=1776425215&width=500",
            "contactPoint": [{"@type": "ContactPoint", "email": "service.nas@ugreen.com"}],
            "sameAs": ["https://www.facebook.com/UgreenGlobal"]
        })
    }

    pub(crate) fn web_page() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "WebPage",
            "name": "UGREEN NASync DXP2800 GT",
            "description": {"error": "json not allowed for this object"},
            "url": "https://ai-ca.ugreen.com/products/ugreen-nasync-dxp2800-gt-2-bay-nas-storage"
        })
    }

    pub(crate) fn web_site() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": "UGREEN AIoT CA",
            "url": "https://ai-ca.ugreen.com",
            "potentialAction": {
                "@type": "SearchAction",
                "target": "https://ai-ca.ugreen.com/search?type=product&q={search_term_string}",
                "query-input": "required name=search_term_string"
            }
        })
    }

    /// Dawn's header copy of the shop.
    pub(crate) fn header_organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "name": "UGREEN AIoT CA",
            "logo": "https://ai-ca.ugreen.com/cdn/shop/files/favicon_green.png?v=1776425215&width=500",
            "sameAs": ["", ""],
            "url": "https://ai-ca.ugreen.com"
        })
    }

    /// The four islands of a product page.
    pub(crate) fn scripts() -> Value {
        json!(
            [
                organization(),
                web_page(),
                web_site(),
                header_organization()
            ]
            .into_iter()
            .map(|data| json!({"type": "application/ld+json", "data": data}))
            .collect::<Vec<_>>()
        )
    }

    /// Trimmed from `products-worry-free-purchase.html`.
    pub(crate) fn add_on_product() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@type": "Product",
            "name": "Worry-Free Purchase",
            "url": "https://ai-ca.ugreen.com/products/worry-free-purchase",
            "image": ["https://ai-ca.ugreen.com/cdn/shop/files/Worry-Free-Purchase.png?v=1776407054&width=1920"],
            "description": "Worry-Free Shopping with Seel",
            "sku": "SEEL-WFP",
            "brand": {"@type": "Brand", "name": "Seel"},
            "offers": [{
                "@type": "Offer",
                "sku": "SEEL-WFP",
                "availability": "http://schema.org/InStock",
                "price": 0.98,
                "priceCurrency": "CAD",
                "url": "https://ai-ca.ugreen.com/products/worry-free-purchase?variant=48690889392378"
            }]
        })
    }

    #[test]
    fn every_island_type_deserializes() {
        for island in [
            organization(),
            header_organization(),
            web_page(),
            web_site(),
            add_on_product(),
        ] {
            serde_json::from_value::<LdJson>(island.clone())
                .unwrap_or_else(|e| panic!("{island}: {e}"));
        }
    }

    #[test]
    fn a_web_page_description_is_text_or_liquids_error() {
        let Ok(LdJson::WebPage(page)) = serde_json::from_value::<LdJson>(web_page()) else {
            panic!("a web page");
        };
        assert!(matches!(
            page.description,
            WebPageDescription::LiquidError(_)
        ));

        let mut input = web_page();
        input["description"] = json!("UGREEN");
        let Ok(LdJson::WebPage(page)) = serde_json::from_value::<LdJson>(input) else {
            panic!("a web page");
        };
        assert_eq!(
            page.description,
            WebPageDescription::Text("UGREEN".to_string())
        );
    }

    #[test]
    fn an_unseen_island_type_fails() {
        for island_type in ["BreadcrumbList", "ProductGroup"] {
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
        let mut island = add_on_product();
        island["gtin"] = json!("1");
        assert!(serde_json::from_value::<LdJson>(island).is_err());

        let mut island = web_page();
        island["description"] = json!({"error": "x", "message": "y"});
        assert!(serde_json::from_value::<LdJson>(island).is_err());
    }

    #[test]
    fn an_unexpected_script_attribute_fails() {
        let script = json!({
            "type": "application/ld+json",
            "class": "jdgm-server-jld",
            "data": organization()
        });
        assert!(serde_json::from_value::<LdJsonScript>(script).is_err());
    }
}
