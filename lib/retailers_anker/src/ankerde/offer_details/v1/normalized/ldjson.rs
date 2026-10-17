use super::super::raw;
use super::{NormalizeError, decimal_money};
use money::Money;
use retailer_sauce::availability::Availability;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LdJson {
    pub product: Product,
    // Commented out: the organisation record is not needed, and nothing downstream reads it.
    // pub corporation: Option<Corporation>,
    // Commented out: home and product details are duplicated elsewhere, and the intermediate
    // collection is not relevant.
    // pub breadcrumb_list: Option<BreadcrumbList>,
    // Commented out: questions and answers are not needed.
    // pub faq_page: Option<FaqPage>,
}

impl LdJson {
    pub(crate) fn from_raw(
        items: Vec<raw::ldjsonitem::LdJsonScript>,
    ) -> Result<LdJson, NormalizeError> {
        let mut product: Option<Product> = None;

        // One script holds an array of islands, not one island.
        for record in items.into_iter().flat_map(|script| script.data) {
            match record {
                raw::ldjsonitem::LdJson::Product(island) => {
                    // First island wins; no document in the subset carries two.
                    if product.is_none() {
                        product = Some(Product::from_raw(island)?);
                    }
                }
                raw::ldjsonitem::LdJson::Corporation(_) => {}
                raw::ldjsonitem::LdJson::BreadcrumbList(_) => {}
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
    // Commented out: the record's featured image with `&width=1600` appended, on every page.
    // pub images: String,
    pub url: String,
    pub brand: String,
    pub offer: Offer,
}

impl Product {
    fn from_raw(raw: raw::ldjsonitem::Product) -> Result<Product, NormalizeError> {
        let offer = Offer::from_raw(raw.offers)?;

        Ok(Product {
            // name: raw.name,
            description: raw.description,
            // images: raw.image,
            url: raw.url,
            brand: raw.brand.name,
            offer,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    pub availability: Availability,
    pub price: Money,
}

impl Offer {
    fn from_raw(raw: raw::ldjsonitem::Offer) -> Result<Offer, NormalizeError> {
        Ok(Offer {
            availability: Availability::from_schema_org(raw.availability),
            price: decimal_money(&raw.price, &raw.price_currency, "offers.price")?,
        })
    }
}

/* Corporation normalization is disabled; the organisation record is not needed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Corporation {
    pub name: String,
    pub url: String,
    pub logo: String,

    pub same_as: Vec<String>,

    pub id: Option<String>,

    pub legal_name: Option<String>,

    pub email: Option<StringOrList>,

    pub description: Option<String>,

    pub telephone: Option<String>,
}

impl Corporation {
    fn from_raw(raw: raw::ldjsonitem::Corporation) -> Corporation {
        Corporation {
            name: raw.name,
            url: raw.url,
            logo: raw.logo,
            same_as: raw.same_as,
            id: raw.id,
            legal_name: raw.legal_name,
            email: raw.email.map(StringOrList::from_raw),
            description: raw.description,
            telephone: raw.telephone,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum StringOrList {
    One(String),
    Many(Vec<String>),
}

impl StringOrList {
    fn from_raw(raw: raw::ldjsonitem::StringOrList) -> StringOrList {
        match raw {
            raw::ldjsonitem::StringOrList::One(value) => StringOrList::One(value),
            raw::ldjsonitem::StringOrList::Many(value) => StringOrList::Many(value),
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

    fn script(islands: Vec<Value>) -> Vec<raw::ldjsonitem::LdJsonScript> {
        vec![serde_json::from_value(json!({ "data": islands })).unwrap()]
    }

    fn product_island() -> Value {
        json!({
            "@type": "Product", "@context": "https://schema.org",
            "name": "Anker Nano Ladegerät", "description": "d",
            "image": "https://cdn.shopify.com/a.png?v=1&width=1600",
            "url": "https://www.anker.com/de/products/a2345",
            "brand": {"@type": "Brand", "name": "Anker"},
            "offers": {
                "@type": "Offer", "availability": "https://schema.org/OutOfStock",
                "price": "159.99", "priceCurrency": "EUR"
            }
        })
    }

    #[test]
    fn a_document_without_a_product_island_fails_to_normalize() {
        let breadcrumbs = json!({
            "@type": "BreadcrumbList", "@context": "https://schema.org", "itemListElement": []
        });

        let error = LdJson::from_raw(script(vec![breadcrumbs])).unwrap_err();

        assert_eq!(error, NormalizeError::MissingProductIsland);
    }

    #[test]
    fn the_offer_types_its_price_and_availability() {
        let offer = LdJson::from_raw(script(vec![product_island()]))
            .unwrap()
            .product
            .offer;

        assert_eq!(offer.price, Money::from_raw(159.99, "EUR").unwrap());
        assert_eq!(offer.availability, Availability::OutOfStock);
    }

    #[test]
    fn a_price_that_is_not_a_number_fails_with_its_value() {
        let mut island = product_island();
        island["offers"]["price"] = json!("ab 159,99");

        let error = LdJson::from_raw(script(vec![island])).unwrap_err();

        assert!(matches!(
            error,
            NormalizeError::PriceNotANumber { field, value, .. }
                if field == "offers.price" && value == "ab 159,99"
        ));
    }

    #[test]
    fn neither_the_duplicate_name_nor_the_resized_image_crosses() {
        let product = LdJson::from_raw(script(vec![product_island()]))
            .unwrap()
            .product;

        let output = serde_json::to_value(product).unwrap();

        assert!(output.get("name").is_none());
        assert!(!output.to_string().contains("width=1600"));
    }
}
