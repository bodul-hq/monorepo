use super::super::raw;
use super::NormalizeError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdJson {
    pub product: Product,
    // Commented out: the organisation record is not needed, and nothing downstream reads it.
    // pub corporation: Option<Corporation>,
    // Commented out: home and product details are duplicated elsewhere, and the intermediate
    // collection is not relevant.
    // pub breadcrumb_list: Option<BreadcrumbList>,
    // Commented out: licensing boilerplate for the first gallery image.
    // pub image_object: Option<ImageObject>,
    // Commented out: questions and answers are not needed.
    // pub faq_page: Option<FaqPage>,
}

impl LdJson {
    pub(crate) fn from_raw(
        items: Vec<raw::ldjsonitem::LdJsonItem>,
    ) -> Result<LdJson, NormalizeError> {
        let mut product: Option<Product> = None;

        for item in items {
            match item.data {
                raw::ldjsonitem::LdJson::Product(island) => {
                    product.get_or_insert_with(|| Product::from_raw(island));
                }
                raw::ldjsonitem::LdJson::Corporation(_) => {}
                raw::ldjsonitem::LdJson::BreadcrumbList(_) => {}
                raw::ldjsonitem::LdJson::ImageObject(_) => {}
                raw::ldjsonitem::LdJson::FaqPage(_) => {}
            }
        }

        let product = product.ok_or(NormalizeError::MissingProductIsland)?;

        Ok(LdJson { product })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    // Commented out: the record's `title` with its markup HTML-escaped, and nothing more.
    // pub name: String,
    // Commented out: the record's `description`, HTML-escaped.
    // pub description: String,
    pub images: Vec<String>,
    // Commented out: the page's canonical URL.
    // pub url: String,
    // Commented out: `ankernordics`, the storefront's own name, not a brand.
    // pub brand: String,
    // Commented out: one per variant, with a price and no SKU, availability or condition to
    // tell them apart; the record's variants state all of it.
    // pub offers: Vec<Offer>,
}

impl Product {
    fn from_raw(raw: raw::ldjsonitem::Product) -> Product {
        Product {
            // name: raw.name,
            // description: raw.description,
            images: raw.image,
            // url: raw.url,
            // brand: raw.brand.name,
            // offers: raw.offers.into_iter().map(Offer::from_raw).collect(),
        }
    }
}

/* Offer normalization is disabled; the record's variants carry every offer fact.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Offer {
    pub price: f64,
    pub price_currency: String,

    pub high_price: String,

    pub low_price: String,
    pub offer_count: i64,
}

impl Offer {
    fn from_raw(raw: raw::ldjsonitem::Offer) -> Offer {
        Offer {
            price: raw.price,
            price_currency: raw.price_currency,
            high_price: raw.high_price,
            low_price: raw.low_price,
            offer_count: raw.offer_count,
        }
    }
}
*/

/* Corporation normalization is disabled; the organisation record is not needed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Corporation {
    pub id: String,
    pub name: String,
    pub legal_name: String,
    pub url: String,
    pub logo: String,
    pub same_as: Vec<String>,
}

impl Corporation {
    fn from_raw(raw: raw::ldjsonitem::Corporation) -> Corporation {
        Corporation {
            id: raw.id,
            name: raw.name,
            legal_name: raw.legal_name,
            url: raw.url,
            logo: raw.logo,
            same_as: raw.same_as,
        }
    }
}
*/

/* Breadcrumb normalization is disabled: home and product details are duplicated,
and the intermediate collection is not relevant.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BreadcrumbList {
    pub crumbs: Vec<Breadcrumb>,
}

impl BreadcrumbList {
    fn from_raw(raw: raw::ldjsonitem::BreadcrumbList) -> BreadcrumbList {
        BreadcrumbList {
            crumbs: raw
                .item_list_element
                .into_iter()
                .map(Breadcrumb::from_raw)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Breadcrumb {
    pub name: String,

    pub url: String,
    pub position: i64,
}

impl Breadcrumb {
    fn from_raw(raw: raw::ldjsonitem::BreadcrumbItem) -> Breadcrumb {
        Breadcrumb {
            name: raw.name,
            url: raw.item,
            position: raw.position,
        }
    }
}
*/

/* ImageObject normalization is disabled; it licenses the first gallery image and says no more.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageObject {
    pub content_url: String,
    pub license: String,
    pub acquire_license_page: String,
    pub credit_text: String,
    pub creator: String,
}

impl ImageObject {
    fn from_raw(raw: raw::ldjsonitem::ImageObject) -> ImageObject {
        ImageObject {
            content_url: raw.content_url,
            license: raw.license,
            acquire_license_page: raw.acquire_license_page,
            credit_text: raw.credit_text,
            creator: raw.creator.name,
        }
    }
}
*/

/* FAQ normalization is disabled while questions and answers are not needed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FaqPage {
    /// `None` can accompany a present FAQ island whose `mainEntity` is omitted.
    pub questions: Option<Vec<FaqQuestion>>,
}

impl FaqPage {
    fn from_raw(raw: raw::ldjsonitem::FaqPage) -> FaqPage {
        FaqPage {
            questions: raw
                .main_entity
                .map(|items| items.into_iter().map(FaqQuestion::from_raw).collect()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FaqQuestion {
    pub question: String,
    pub answer: String,
}

impl FaqQuestion {
    fn from_raw(raw: raw::ldjsonitem::FaqQuestion) -> FaqQuestion {
        FaqQuestion {
            question: raw.name,
            answer: raw.accepted_answer.text,
        }
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use html_extractor::serde_json::{self, Value, json};

    fn islands(bodies: &[Value]) -> Vec<raw::ldjsonitem::LdJsonItem> {
        bodies
            .iter()
            .map(|data| serde_json::from_value(json!({ "data": data })).unwrap())
            .collect()
    }

    fn product_island() -> Value {
        json!({
            "@type": "Product", "@context": "https://schema.org",
            "brand": {"@type": "Brand", "name": "ankernordics"},
            "description": "Anker&apos;s first", "image": ["i1", "i2"],
            "name": "Anker &lt;b&gt;315&lt;/b&gt;", "url": "https://www.ankernordics.com/products/a",
            "offers": [{"@type": "Offer", "price": 2490, "lowPrice": "2490.0",
                "highPrice": "2490.0", "priceCurrency": "SEK", "offerCount": 999}]
        })
    }

    #[test]
    fn a_document_without_a_product_island_fails_to_normalize() {
        let breadcrumbs = json!({
            "@type": "BreadcrumbList", "@context": "https://schema.org", "itemListElement": []
        });

        assert_eq!(
            LdJson::from_raw(islands(&[breadcrumbs])).unwrap_err(),
            NormalizeError::MissingProductIsland
        );
    }

    #[test]
    fn only_the_gallery_crosses_from_the_island() {
        let ld_json = LdJson::from_raw(islands(&[product_island()])).unwrap();

        assert_eq!(
            serde_json::to_value(ld_json).unwrap(),
            json!({"product": {"images": ["i1", "i2"]}})
        );
    }
}
