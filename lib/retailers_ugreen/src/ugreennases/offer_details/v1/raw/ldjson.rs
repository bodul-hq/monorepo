#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    /// `jdgm-server-jld` on Judge.me's review island, absent on the others.
    #[serde(default)]
    pub class: Option<String>,
    pub data: LdJson,
}

/// The island types seen in the corpus. Every page carries the shop's `Organization` twice, a
/// `WebPage` and a `WebSite`; the theme renders no product island. On 6 of the 18 saved pages
/// Judge.me adds a review `Product`, without offers.
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

/// Judge.me's review island (`script.jdgm-server-jld`): the product's name, an image, its rating
/// and recent reviews, but no offer. Not read: it states no fact normalization needs.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    /// `/products/<handle>#product`.
    #[serde(rename = "@id")]
    pub id: String,
    pub name: String,
    #[serde(rename = "aggregateRating")]
    pub aggregate_rating: AggregateRating,
    pub image: String,
    pub review: Vec<Review>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateRating {
    #[serde(rename = "@type")]
    pub rating_type: String,
    /// A decimal string, such as `4.73`.
    #[serde(rename = "ratingValue")]
    pub rating_value: String,
    #[serde(rename = "reviewCount")]
    pub review_count: u64,
    #[serde(rename = "bestRating")]
    pub best_rating: i64,
    #[serde(rename = "worstRating")]
    pub worst_rating: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    #[serde(rename = "@type")]
    pub review_type: String,
    #[serde(rename = "reviewRating")]
    pub review_rating: Rating,
    pub author: Person,
    #[serde(rename = "reviewBody")]
    pub review_body: String,
    #[serde(rename = "datePublished")]
    pub date_published: String,
    /// Photos the reviewer attached; on 4 of the 54 saved reviews.
    #[serde(default)]
    pub image: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rating {
    #[serde(rename = "@type")]
    pub rating_type: String,
    #[serde(rename = "ratingValue")]
    pub rating_value: i64,
    #[serde(rename = "bestRating")]
    pub best_rating: i64,
    #[serde(rename = "worstRating")]
    pub worst_rating: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Person {
    #[serde(rename = "@type")]
    pub person_type: String,
    pub name: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{LdJson, LdJsonScript, WebPageDescription};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "Organization",
            "name": "UGREEN AIoT ES",
            "url": "https://ai-es.ugreen.com",
            "logo": "https://ai-es.ugreen.com/cdn/shop/files/favicon_green.png?v=1776425030&width=500",
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
            "url": "https://ai-es.ugreen.com/products/ugreen-nasync-dxp2800-gt-2-bay-nas-storage"
        })
    }

    pub(crate) fn web_site() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": "UGREEN AIoT ES",
            "url": "https://ai-es.ugreen.com",
            "potentialAction": {
                "@type": "SearchAction",
                "target": "https://ai-es.ugreen.com/search?type=product&q={search_term_string}",
                "query-input": "required name=search_term_string"
            }
        })
    }

    /// Dawn's header copy of the shop.
    pub(crate) fn header_organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "name": "UGREEN AIoT ES",
            "logo": "https://ai-es.ugreen.com/cdn/shop/files/favicon_green.png?v=1776425030&width=500",
            "sameAs": ["", ""],
            "url": "https://ai-es.ugreen.com"
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

    /// Trimmed from `products-ugreen-nasync-dh2300-servidor-nas.html`.
    pub(crate) fn review_product() -> Value {
        json!({
            "@context": "https://schema.org/",
            "@type": "Product",
            "@id": "/products/ugreen-nasync-dh2300-servidor-nas#product",
            "name": "UGREEN NASync DH2300",
            "aggregateRating": {
                "@type": "AggregateRating",
                "ratingValue": "4.73",
                "reviewCount": 11,
                "bestRating": 5,
                "worstRating": 1
            },
            "image": "https://cdn.shopify.com/s/files/1/0977/9950/4154/files/DH2300-remove.png?v=1768505740",
            "review": [{
                "@type": "Review",
                "reviewRating": {"@type": "Rating", "ratingValue": 5, "bestRating": 5, "worstRating": 1},
                "author": {"@type": "Person", "name": "Daniel Francisco"},
                "reviewBody": "Cumple todas las expectativas",
                "datePublished": "2026-09-14T22:03:00Z",
                "image": ["https://review-images.judgeme.com/ugreen-nas-es/1789423381__1789423376581-img_6567__original.jpeg?quality=80&width=1024"]
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
            review_product(),
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
        let mut island = review_product();
        island["offers"] = json!([]);
        assert!(serde_json::from_value::<LdJson>(island).is_err());

        let mut island = web_page();
        island["description"] = json!({"error": "x", "message": "y"});
        assert!(serde_json::from_value::<LdJson>(island).is_err());
    }

    #[test]
    fn judge_mes_island_carries_a_class() {
        let script = json!({
            "type": "application/ld+json",
            "class": "jdgm-server-jld",
            "data": review_product()
        });
        let script = serde_json::from_value::<LdJsonScript>(script).expect("deserializes");
        assert_eq!(script.class.as_deref(), Some("jdgm-server-jld"));
    }

    #[test]
    fn a_review_without_photos_deserializes() {
        let mut island = review_product();
        island["review"][0]
            .as_object_mut()
            .expect("a review")
            .remove("image");

        assert!(serde_json::from_value::<LdJson>(island).is_ok());
    }

    #[test]
    fn an_unexpected_script_attribute_fails() {
        let script = json!({
            "type": "application/ld+json",
            "id": "judgeme",
            "data": organization()
        });
        assert!(serde_json::from_value::<LdJsonScript>(script).is_err());
    }
}
