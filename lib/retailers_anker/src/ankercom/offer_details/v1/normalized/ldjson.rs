use super::super::raw;
use super::{NormalizeError, money};
use money::Money;
use retailer_sauce::availability::Availability;
use retailer_sauce::item_condition::ItemCondition;
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
        items: Vec<raw::ldjsonitem::LdJsonItem>,
    ) -> Result<LdJson, NormalizeError> {
        let mut product: Option<Product> = None;

        for item in items {
            match item.data {
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
    // Commented out: the record's `title` with its markup HTML-escaped, and nothing more.
    // pub name: String,
    pub description: String,
    pub images: Vec<String>,
    pub sku: String,
    pub gtin: Option<String>,
    pub brand: String,
    pub offer: Offer,
}

impl Product {
    fn from_raw(raw: raw::ldjsonitem::Product) -> Result<Product, NormalizeError> {
        let offer = Offer::from_raw(raw.offers)?;

        Ok(Product {
            // name: raw.name,
            description: raw.description,
            images: raw.image,
            sku: raw.sku,
            gtin: raw.gtin,
            brand: raw.brand.name,
            offer,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    pub availability: Availability,
    pub price: Money,
    pub item_condition: ItemCondition,
    pub url: String,
}

impl Offer {
    fn from_raw(raw: raw::ldjsonitem::Offer) -> Result<Offer, NormalizeError> {
        let price = money(raw.price, &raw.price_currency, "offers.price")?;

        Ok(Offer {
            availability: Availability::from_schema_org(raw.availability),
            price,
            item_condition: ItemCondition::from_schema_org(raw.item_condition),
            url: raw.url,
        })
    }
}

/* Corporation normalization is disabled; the organisation record is not needed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Corporation {
    pub id: String,
    pub name: String,
    pub legal_name: String,
    pub url: String,

    pub logo: String,

    pub email: String,

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

    fn islands(bodies: &[Value]) -> Vec<raw::ldjsonitem::LdJsonItem> {
        bodies
            .iter()
            .map(|data| serde_json::from_value(json!({ "data": data })).unwrap())
            .collect()
    }

    fn product_island(condition: &str) -> Value {
        json!({
            "@type": "Product", "@context": "https://schema.org",
            "brand": {"@type": "Brand", "name": "Anker"},
            "description": "d", "image": ["i"], "name": "Anker &lt;b&gt;633&lt;/b&gt;",
            "sku": "A1641011", "gtin": "194644098117",
            "offers": {
                "@type": "Offer", "availability": "https://schema.org/SoldOut",
                "itemCondition": condition, "price": 79.99,
                "priceCurrency": "USD", "url": "u"
            }
        })
    }

    #[test]
    fn a_document_without_a_product_island_fails_to_normalize() {
        let corporation = json!({
            "@type": "Corporation", "@context": "https://schema.org", "@id": "i",
            "name": "Anker", "legalName": "Anker Innovations", "url": "u", "logo": "l",
            "email": "e", "sameAs": []
        });

        let error = LdJson::from_raw(islands(&[corporation])).unwrap_err();

        assert_eq!(error, NormalizeError::MissingProductIsland);
    }

    #[test]
    fn the_offer_types_its_price_availability_and_condition() {
        let ld_json = LdJson::from_raw(islands(&[product_island(
            "https://schema.org/NewCondition",
        )]))
        .unwrap();
        let offer = ld_json.product.offer;

        assert_eq!(offer.price, Money::from_raw(79.99, "USD").unwrap());
        assert_eq!(offer.availability, Availability::SoldOut);
        assert_eq!(offer.item_condition, ItemCondition::New);
    }

    #[test]
    fn the_islands_escaped_name_does_not_cross() {
        let ld_json = LdJson::from_raw(islands(&[product_island(
            "https://schema.org/NewCondition",
        )]))
        .unwrap();

        let output = serde_json::to_value(ld_json).unwrap();

        assert!(output["product"].get("name").is_none());
        assert!(!output.to_string().contains("&lt;"));
    }
}
