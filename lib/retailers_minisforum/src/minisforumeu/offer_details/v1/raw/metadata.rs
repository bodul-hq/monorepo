//! Company and navigation metadata, validated but excluded from the normalized offer.

use html_extractor::serde_json::Value;

/// The theme's small Organization and the expanded company block coexist on newer pages.
/// All fields after `url` are absent from the older theme block.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@type")]
    pub island_type: String,
    pub name: String,
    pub logo: String,
    pub url: String,
    pub legal_name: Option<String>,
    pub description: Option<String>,
    pub founding_date: Option<String>,
    pub founders: Option<Vec<Founder>>,
    pub address: Option<PostalAddress>,
    pub contact_point: Option<Vec<ContactPoint>>,
    pub same_as: Option<Vec<String>>,
    pub area_served: Option<GeoCircle>,
    pub knows_about: Option<Vec<String>>,
    pub brand: Option<Vec<Brand>>,
    /// A recursive company-wide catalog, not the page's product or its offers.
    /// Preserve this unused region as JSON, as with the SEO app's graph.
    pub has_offer_catalog: Option<Value>,
    pub aggregate_rating: Option<CompanyRating>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Founder {
    #[serde(rename = "@type")]
    pub founder_type: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostalAddress {
    #[serde(rename = "@type")]
    pub address_type: String,
    pub address_locality: String,
    pub address_region: String,
    pub address_country: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContactPoint {
    #[serde(rename = "@type")]
    pub contact_point_type: String,
    pub contact_type: String,
    pub email: String,
    pub available_language: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeoCircle {
    #[serde(rename = "@type")]
    pub area_type: String,
    pub geo_midpoint: GeoCoordinates,
    pub geo_radius: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeoCoordinates {
    #[serde(rename = "@type")]
    pub coordinates_type: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@type")]
    pub brand_type: String,
    pub name: String,
    pub description: String,
}

/// Company ratings encode their counts as strings; product review counts are numbers.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompanyRating {
    #[serde(rename = "@type")]
    pub rating_type: String,
    pub rating_value: String,
    pub best_rating: String,
    pub worst_rating: String,
    pub review_count: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BreadcrumbList {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@type")]
    pub breadcrumb_type: String,
    pub item_list_element: Vec<Breadcrumb>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Breadcrumb {
    #[serde(rename = "@type")]
    pub item_type: String,
    pub position: u64,
    pub name: String,
    pub item: String,
}

#[cfg(test)]
mod tests {
    use html_extractor::serde_json::{self, Value, json};

    use super::super::ldjson::LdJson;

    fn organization() -> Value {
        serde_json::from_str(include_str!("../fixtures/organization.json")).unwrap()
    }

    #[test]
    fn expanded_organization_keeps_address_and_company_rating() {
        let LdJson::Organization(organization) = serde_json::from_value(organization()).unwrap()
        else {
            panic!("an organization");
        };
        assert_eq!(organization.address.unwrap().address_country, "CN");
        assert_eq!(organization.contact_point.unwrap().len(), 2);
        assert_eq!(organization.aggregate_rating.unwrap().review_count, "15000");
    }

    #[test]
    fn organization_still_rejects_unknown_fields_and_wrong_types() {
        for pointer in ["", "/address", "/contactPoint/0", "/areaServed/geoMidpoint"] {
            let mut input = organization();
            input.pointer_mut(pointer).unwrap()["unexpected"] = json!(true);
            let error = serde_json::from_value::<LdJson>(input).unwrap_err();
            assert!(
                error.to_string().contains("unknown field `unexpected`"),
                "{error}"
            );
        }
        let mut input = organization();
        input["address"]["addressCountry"] = json!(42);
        assert!(serde_json::from_value::<LdJson>(input).is_err());
    }

    #[test]
    fn breadcrumbs_still_reject_unknown_fields() {
        let fixture: Value =
            serde_json::from_str(include_str!("../fixtures/breadcrumbs.json")).unwrap();
        for pointer in ["", "/itemListElement/0"] {
            let mut input = fixture.clone();
            input.pointer_mut(pointer).unwrap()["unexpected"] = json!(true);
            let error = serde_json::from_value::<LdJson>(input).unwrap_err();
            assert!(
                error.to_string().contains("unknown field `unexpected`"),
                "{error}"
            );
        }
    }
}
