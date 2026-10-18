//! Expresses the storefront's product facts in the system's vocabulary. The schema.org island and
//! the `__NEXT_DATA__` record both cross, because reconciling them is structuralization's job;
//! what nothing needs - several duplicate renderings among it - stays behind as the commented-out
//! exclusion records.

pub mod ldjson;
pub mod product;

use super::raw;
use money::{Money, MoneyError};
use serde::{Deserialize, Serialize};
use std::num::ParseFloatError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum NormalizeError {
    #[error("no `Product` ld+json island on the document")]
    MissingProductIsland,

    #[error("`{field}` `{value}` is not a number: {source}")]
    PriceNotANumber {
        field: String,
        value: String,
        source: ParseFloatError,
    },

    #[error("`{field}` is not money: {source:?}")]
    NotMoney { field: String, source: MoneyError },

    #[error("discount `{code}` `{field}` `{value}` is not an RFC 3339 instant: {source}")]
    DiscountInstantNotAnInstant {
        code: String,
        field: String,
        value: String,
        source: chrono::ParseError,
    },

    #[error("discount `{code}` has an unknown `value_type` `{value_type}`")]
    UnknownDiscountValueType { code: String, value_type: String },
}

fn money(amount: f64, currency: &str, field: &str) -> Result<Money, NormalizeError> {
    Money::from_raw(amount, currency).map_err(|source| NormalizeError::NotMoney {
        field: field.to_string(),
        source,
    })
}

fn decimal_money(amount: &str, currency: &str, field: &str) -> Result<Money, NormalizeError> {
    money(decimal(amount, field)?, currency, field)
}

fn decimal(raw: &str, field: &str) -> Result<f64, NormalizeError> {
    raw.parse::<f64>()
        .map_err(|source| NormalizeError::PriceNotANumber {
            field: field.to_string(),
            value: raw.to_string(),
            source,
        })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkerItalyComNormalizedDocument {
    // Commented out: `it` on every page; the retailer code already names the storefront.
    // pub locale: String,
    pub ld_json: ldjson::LdJson,
    pub canonical_url: String,
    // Commented out: page metadata beyond `canonical`, lifted above; `noindex` is false on every
    // page.
    // pub seo: PageSeo,
    pub product: product::ProductRecord,
}

impl AnkerItalyComNormalizedDocument {
    pub fn from_raw(
        raw: raw::AnkerItalyComRawDocument,
    ) -> Result<AnkerItalyComNormalizedDocument, NormalizeError> {
        let next_data = raw.next_data.data;
        let page_props = next_data.props.page_props;

        Ok(AnkerItalyComNormalizedDocument {
            // locale: next_data.locale,
            ld_json: ldjson::LdJson::from_raw(raw.ld_json)?,
            canonical_url: page_props.seo.canonical,
            // seo: PageSeo::from_raw(page_props.seo),
            product: product::ProductRecord::from_raw(page_props.product)?,
        })
    }
}

/* Page SEO normalization is disabled beyond `canonical`: nothing downstream reads the rest.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PageSeo {
    pub title: String,
    pub description: String,

    pub canonical: String,

    pub noindex: bool,
    pub thumbnail: String,
    pub language_alternates: Vec<LanguageAlternate>,
    pub open_graph: OpenGraph,
}

impl PageSeo {
    fn from_raw(raw: raw::nextdata::Seo) -> PageSeo {
        PageSeo {
            title: raw.title,
            description: raw.description,
            canonical: raw.canonical,
            noindex: raw.noindex,
            thumbnail: raw.thumbnail,
            language_alternates: raw
                .language_alternates
                .into_iter()
                .map(LanguageAlternate::from_raw)
                .collect(),
            open_graph: OpenGraph::from_raw(raw.open_graph),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LanguageAlternate {
    pub href: String,
    pub href_lang: String,
}

impl LanguageAlternate {
    fn from_raw(raw: raw::nextdata::LanguageAlternate) -> LanguageAlternate {
        LanguageAlternate {
            href: raw.href,
            href_lang: raw.href_lang,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenGraph {
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub site_name: String,
    pub images: Vec<OpenGraphImage>,
}

impl OpenGraph {
    fn from_raw(raw: raw::nextdata::OpenGraph) -> OpenGraph {
        OpenGraph {
            title: raw.title,
            description: raw.description,
            kind: raw.type_,
            site_name: raw.site_name,
            images: raw
                .images
                .into_iter()
                .map(OpenGraphImage::from_raw)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenGraphImage {
    pub url: String,
    pub alt: String,
    pub width: String,
    pub height: String,
}

impl OpenGraphImage {
    fn from_raw(raw: raw::nextdata::OpenGraphImage) -> OpenGraphImage {
        OpenGraphImage {
            url: raw.url,
            alt: raw.alt,
            width: raw.width,
            height: raw.height,
        }
    }
}
*/
