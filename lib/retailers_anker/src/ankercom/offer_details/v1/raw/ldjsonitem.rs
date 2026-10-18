#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonItem {
    pub data: LdJson,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Product(Product),
    Corporation(Corporation),
    BreadcrumbList(BreadcrumbList),
    #[serde(rename = "FAQPage")]
    FaqPage(FaqPage),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub description: String,

    pub image: Vec<String>,
    pub sku: String,

    pub gtin: Option<String>,
    pub brand: Brand,

    pub offers: Offer,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    #[serde(rename = "@type")]
    pub kind: String,

    pub availability: String,

    pub price: f64,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    #[serde(rename = "itemCondition")]
    pub item_condition: String,

    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corporation {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub name: String,
    #[serde(rename = "legalName")]
    pub legal_name: String,
    pub url: String,

    pub logo: String,

    pub email: String,

    #[serde(rename = "sameAs")]
    pub same_as: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BreadcrumbList {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "itemListElement")]
    pub item_list_element: Vec<BreadcrumbItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BreadcrumbItem {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,

    pub item: String,
    pub position: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaqPage {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "mainEntity")]
    pub main_entity: Vec<FaqQuestion>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaqQuestion {
    #[serde(rename = "@type")]
    pub kind: String,

    pub name: String,
    #[serde(rename = "acceptedAnswer")]
    pub accepted_answer: FaqAnswer,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaqAnswer {
    #[serde(rename = "@type")]
    pub kind: String,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::{LdJson, LdJsonItem};
    use html_extractor::serde_json;

    #[test]
    fn the_tag_is_stripped_before_the_variant_sees_the_object() {
        let product = r#"{"@type":"Product","@context":"https://schema.org",
            "brand":{"@type":"Brand","name":"Anker"},"description":"d","gtin":"g",
            "image":["i"],"name":"n","offers":{"@type":"Offer",
            "availability":"https://schema.org/InStock","itemCondition":"https://schema.org/NewCondition",
            "price":99.99,"priceCurrency":"USD","url":"u"},"sku":"A1"}"#;

        match serde_json::from_str::<LdJson>(product).expect("the Product island deserializes") {
            LdJson::Product(product) => {
                assert_eq!(product.sku, "A1");
                assert_eq!(product.offers.price, 99.99);
                assert_eq!(product.brand.name, "Anker");
                assert_eq!(product.gtin.as_deref(), Some("g"));
            }
            other => panic!("read the Product island as {other:?}"),
        }
    }

    #[test]
    fn a_product_without_a_gtin_still_deserializes() {
        let without = r#"{"@type":"Product","@context":"c","brand":{"@type":"Brand","name":"Anker"},
            "description":"d","image":["i"],"name":"n","offers":{"@type":"Offer","availability":"a",
            "itemCondition":"i","price":1.0,"priceCurrency":"USD","url":"u"},"sku":"A1"}"#;

        let LdJson::Product(product) =
            serde_json::from_str::<LdJson>(without).expect("gtin is optional")
        else {
            panic!("expected a Product");
        };
        assert!(product.gtin.is_none());
    }

    #[test]
    fn an_unknown_type_is_rejected_rather_than_ignored() {
        let unknown = r#"{"@type":"VideoObject","@context":"c","name":"n"}"#;
        assert!(serde_json::from_str::<LdJson>(unknown).is_err());
    }

    #[test]
    fn the_island_wrapper_carries_the_parsed_body() {
        let item = r#"{"data":{"@type":"BreadcrumbList","@context":"c","itemListElement":[
            {"@type":"ListItem","item":"i","name":"Home","position":1}]}}"#;

        let item: LdJsonItem = serde_json::from_str(item).expect("the wrapper deserializes");
        let LdJson::BreadcrumbList(list) = item.data else {
            panic!("expected a BreadcrumbList");
        };
        assert_eq!(list.item_list_element[0].name, "Home");
        assert_eq!(list.item_list_element[0].position, 1);
    }
}
