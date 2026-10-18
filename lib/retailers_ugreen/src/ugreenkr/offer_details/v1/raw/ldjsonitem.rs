//! Typed schema.org islands. Product offers and review-only Products have distinct schemas.
use html_extractor::serde_json::Number;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonItem {
    #[serde(rename = "type")]
    pub script_type: String,
    /// Present only on review scripts.
    pub class: Option<String>,
    pub data: LdJson,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Organization(Organization),
    Product(ProductIsland),
    ProductGroup(ProductGroup),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ProductIsland {
    Offers(Box<Product>),
    Reviews(ReviewsProduct),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    pub logo: String,
    /// The Korean theme renders this; Horizon omits it.
    #[serde(rename = "sameAs")]
    pub same_as: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    /// Omitted by the Korean and custom US templates.
    #[serde(rename = "@id")]
    pub id: Option<String>,
    pub name: String,
    pub url: String,
    pub image: Images,
    pub description: String,
    pub brand: Brand,
    pub category: Option<String>,
    /// Absent when the merchant has not supplied an identifier.
    pub sku: Option<String>,
    pub gtin: Option<String>,
    pub offers: Offers,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProductGroup {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub brand: Brand,
    pub category: String,
    #[serde(rename = "productGroupID")]
    pub product_group_id: String,
    pub has_variant: Vec<GroupVariant>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupVariant {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
    pub image: String,
    pub sku: Option<String>,
    pub gtin: Option<String>,
    pub offers: Offer,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Images {
    One(String),
    Many(Vec<String>),
}
impl Images {
    pub fn first(&self) -> Option<&str> {
        match self {
            Self::One(s) => Some(s),
            Self::Many(v) => v.first().map(String::as_str),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Offers {
    One(Offer),
    Many(Vec<Offer>),
}
impl Offers {
    pub fn into_vec(self) -> Vec<Offer> {
        match self {
            Self::One(o) => vec![o],
            Self::Many(v) => v,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Price {
    Text(String),
    Number(Number),
}
impl Price {
    pub fn decimal(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Number(n) => n.to_string(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Offer {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@type")]
    pub kind: String,
    pub availability: String,
    pub price: Price,
    pub price_currency: String,
    pub url: String,
    pub item_condition: Option<String>,
    /// Only five custom US pages render these; retained separately from the displayed prices.
    pub price_specification: Option<Vec<PriceSpecification>>,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PriceSpecification {
    #[serde(rename = "@type")]
    pub kind: String,
    pub price_type: String,
    pub price: Price,
    pub price_currency: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewsProduct {
    #[serde(rename = "@context")]
    pub context: String,
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
    pub kind: String,
    /// A decimal string, such as `4.84`.
    #[serde(rename = "ratingValue")]
    pub rating_value: String,
    #[serde(rename = "reviewCount")]
    pub review_count: i64,
    #[serde(rename = "bestRating")]
    pub best_rating: i64,
    #[serde(rename = "worstRating")]
    pub worst_rating: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "reviewRating")]
    pub review_rating: ReviewRating,
    pub author: Author,
    #[serde(rename = "reviewBody")]
    pub review_body: String,
    #[serde(rename = "datePublished")]
    pub date_published: String,
    pub image: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRating {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "ratingValue")]
    pub rating_value: i64,
    #[serde(rename = "bestRating")]
    pub best_rating: i64,
    #[serde(rename = "worstRating")]
    pub worst_rating: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}
