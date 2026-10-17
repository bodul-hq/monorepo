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

    pub name: Option<String>,

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
    use super::LdJson;
    use html_extractor::serde_json;

    const PRODUCT: &str = r#"{"@type":"Product","@context":"https://schema.org",
        "brand":{"@type":"Brand","name":"Anker"},"description":"d","gtin":"g","image":["i"],
        "name":"n","offers":{"@type":"Offer","availability":"https://schema.org/InStock",
        "itemCondition":"https://schema.org/NewCondition","price":99.99,"priceCurrency":"EUR",
        "url":"u"},"sku":"A1"}"#;

    #[test]
    fn the_tag_is_stripped_before_the_variant_sees_the_object() {
        let LdJson::Product(product) =
            serde_json::from_str::<LdJson>(PRODUCT).expect("the Product island deserializes")
        else {
            panic!("expected a Product");
        };
        assert_eq!(product.sku, "A1");
        assert_eq!(product.offers.price, 99.99);
        assert_eq!(product.brand.name, "Anker");
    }

    #[test]
    fn an_integer_price_and_a_float_price_both_read() {
        for (spelling, expected) in [("99.99", 99.99_f64), ("99", 99.0_f64)] {
            let json = PRODUCT.replace("99.99", spelling);
            let LdJson::Product(product) =
                serde_json::from_str::<LdJson>(&json).expect("both price spellings read")
            else {
                panic!("expected a Product");
            };
            assert_eq!(product.offers.price, expected, "price spelled {spelling}");
        }
    }

    #[test]
    fn an_unknown_type_is_rejected_rather_than_ignored() {
        assert!(
            serde_json::from_str::<LdJson>(r#"{"@type":"VideoObject","@context":"c"}"#).is_err()
        );
    }

    #[test]
    fn a_crumb_without_a_name_still_deserializes() {
        let list = r#"{"@type":"BreadcrumbList","@context":"c","itemListElement":[
            {"@type":"ListItem","item":"i","position":1}]}"#;
        let LdJson::BreadcrumbList(list) =
            serde_json::from_str::<LdJson>(list).expect("a nameless crumb is legal here")
        else {
            panic!("expected a BreadcrumbList");
        };
        assert!(list.item_list_element[0].name.is_none());
    }
}
