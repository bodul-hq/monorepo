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
    // Commented out: licensing metadata for one image, not a fact about the product.
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
                    // First island wins; no document in the subset carries two.
                    if product.is_none() {
                        product = Some(Product::from_raw(island));
                    }
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
    // Commented out: the record's `title`, verbatim, on every page.
    // pub name: String,
    pub description: String,
    // Commented out: the record's `images`, the same URLs in the same order on every page.
    // pub images: Vec<String>,
    // Commented out: the page's canonical, verbatim, on every page.
    // pub url: String,
    /// `ankernordics` on every page - another storefront's name, carried as the page states it.
    pub brand: String,
    // Commented out: one entry per variant restating its price; `offerCount` is a placeholder
    // (`0` or `999`) rather than a count.
    // pub offers: Vec<Offer>,
}

impl Product {
    fn from_raw(raw: raw::ldjsonitem::Product) -> Product {
        Product {
            // name: raw.name,
            description: raw.description,
            // images: raw.image,
            // url: raw.url,
            brand: raw.brand.name,
            // offers: raw.offers.into_iter().map(Offer::from_raw).collect(),
        }
    }
}

/* Offer normalization is disabled with the field above.
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
    pub email: String,

    pub description: String,

    pub telephone: String,
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
            email: raw.email,
            description: raw.description,
            telephone: raw.telephone,
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

/* ImageObject normalization is disabled: licensing metadata for one image.
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
    pub questions: Vec<FaqQuestion>,
}

impl FaqPage {
    fn from_raw(raw: raw::ldjsonitem::FaqPage) -> FaqPage {
        FaqPage {
            questions: raw
                .main_entity
                .into_iter()
                .map(FaqQuestion::from_raw)
                .collect(),
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

    fn items(bodies: &[Value]) -> Vec<raw::ldjsonitem::LdJsonItem> {
        bodies
            .iter()
            .map(|data| serde_json::from_value(json!({ "data": data })).unwrap())
            .collect()
    }

    fn product_island() -> Value {
        json!({
            "@type": "Product", "@context": "https://schema.org",
            "name": "Anker Zolo Caricatore", "description": "d",
            "image": ["https://cdn.shopify.com/a.png"],
            "url": "https://www.ankeritaly.com/products/a121c",
            "brand": {"@type": "Brand", "name": "ankernordics"},
            "offers": [{"@type": "Offer", "price": 49.99, "priceCurrency": "EUR",
                        "highPrice": "49.99", "lowPrice": "49.99", "offerCount": 999}]
        })
    }

    #[test]
    fn a_document_without_a_product_island_fails_to_normalize() {
        let image_object = json!({
            "@type": "ImageObject", "@context": "https://schema.org",
            "acquireLicensePage": "a", "contentUrl": "c",
            "creator": {"@type": "Organization", "name": "n"}, "creditText": "c", "license": "l"
        });

        let error = LdJson::from_raw(items(&[image_object])).unwrap_err();

        assert_eq!(error, NormalizeError::MissingProductIsland);
    }

    #[test]
    fn only_the_description_and_the_brand_cross() {
        let ld_json = LdJson::from_raw(items(&[product_island()])).unwrap();

        assert_eq!(
            serde_json::to_value(ld_json).unwrap(),
            json!({"product": {"description": "d", "brand": "ankernordics"}})
        );
    }
}
