#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: LdJson,
}

/// The island types seen in the corpus: only the shop's `Organization`, one per page. No page
/// renders a `Product` island, so one appearing fails the document until it is typed and checked
/// against the analytics record.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Organization(Organization),
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

#[cfg(test)]
pub(crate) mod tests {
    use super::LdJson;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "logo": "https://www.ugreen.com/cdn/shop/files/ugreen_logo.svg?v=1761026731&width=500",
            "name": "UGREEN Official Site",
            "url": "https://www.ugreen.com"
        })
    }

    #[test]
    fn the_organization_island_deserializes() {
        let island = serde_json::from_value::<LdJson>(organization()).expect("deserializes");
        let LdJson::Organization(organization) = island;
        assert_eq!(organization.name, "UGREEN Official Site");
    }

    #[test]
    fn an_unseen_island_type_fails() {
        for island_type in ["Product", "ProductGroup", "BreadcrumbList"] {
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
        let mut island = organization();
        island["sameAs"] = json!([]);

        assert!(serde_json::from_value::<LdJson>(island).is_err());
    }
}
