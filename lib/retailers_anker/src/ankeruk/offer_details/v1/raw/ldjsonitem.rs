#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    pub data: Vec<LdJson>,
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

    /// Absent on a few pages, e.g. some spare parts.
    pub image: Option<String>,

    pub url: String,
    pub brand: Brand,

    pub offers: Offer,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@context")]
    pub context: Option<String>,
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    #[serde(rename = "@context")]
    pub context: Option<String>,
    #[serde(rename = "@type")]
    pub kind: String,

    pub availability: String,

    pub price: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corporation {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
    pub url: String,
    pub logo: String,

    #[serde(rename = "sameAs")]
    pub same_as: Vec<String>,

    #[serde(rename = "@id")]
    pub id: Option<String>,

    #[serde(rename = "legalName")]
    pub legal_name: Option<String>,

    pub email: Option<StringOrList>,

    pub description: Option<String>,

    pub telephone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum StringOrList {
    One(String),
    Many(Vec<String>),
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
    #[serde(rename = "@context")]
    pub context: Option<String>,
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
    #[serde(rename = "@context")]
    pub context: Option<String>,
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
    #[serde(rename = "acceptedAnswer")]
    pub accepted_answer: FaqAnswer,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaqAnswer {
    #[serde(rename = "@context")]
    pub context: Option<String>,
    #[serde(rename = "@type")]
    pub kind: String,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::{LdJson, LdJsonScript};
    use html_extractor::serde_json;

    #[test]
    fn the_island_is_one_script_holding_an_array() {
        let island = r#"{"data":[
            {"@type":"Corporation","@context":"c","logo":"l","name":"Anker","sameAs":[],"url":"u"},
            {"@type":"BreadcrumbList","@context":"c","itemListElement":[
                {"@type":"ListItem","item":"i","name":"Home","position":1}]}]}"#;

        let island: LdJsonScript = serde_json::from_str(island).expect("the array deserializes");
        assert_eq!(island.data.len(), 2);
        assert!(matches!(island.data[0], LdJson::Corporation(_)));
    }

    #[test]
    fn the_price_is_a_string_and_the_image_is_not_a_list() {
        let product = r#"{"@type":"Product","@context":"c","brand":{"@type":"Brand","name":"Anker"},
            "description":"d","image":"i","name":"n","offers":{"@type":"Offer",
            "availability":"https://schema.org/InStock","price":"99.99","priceCurrency":"GBP"},
            "url":"u"}"#;

        let LdJson::Product(product) =
            serde_json::from_str::<LdJson>(product).expect("the Product deserializes")
        else {
            panic!("expected a Product");
        };
        assert_eq!(product.offers.price, "99.99");
        assert_eq!(product.image.as_deref(), Some("i"));
    }

    #[test]
    fn the_brand_and_offer_carry_a_context_and_the_image_may_be_absent() {
        let product = r#"{"@type":"Product","@context":"c",
            "brand":{"@context":"c","@type":"Brand","name":"Anker"},
            "description":"d","name":"n","offers":{"@context":"c","@type":"Offer",
            "availability":"https://schema.org/OutOfStock","price":"9.99","priceCurrency":"GBP"},
            "url":"u"}"#;

        let LdJson::Product(product) =
            serde_json::from_str::<LdJson>(product).expect("the Product deserializes")
        else {
            panic!("expected a Product");
        };
        assert_eq!(product.image, None);
        assert_eq!(product.brand.context.as_deref(), Some("c"));
    }

    #[test]
    fn the_lean_corporation_tier_reads() {
        let lean = r#"{"@type":"Corporation","@context":"c","logo":"l","name":"Anker",
            "sameAs":["s"],"url":"u"}"#;

        let LdJson::Corporation(corp) =
            serde_json::from_str::<LdJson>(lean).expect("the lean tier deserializes")
        else {
            panic!("expected a Corporation");
        };
        assert!(corp.id.is_none());
        assert!(corp.email.is_none());
    }

    #[test]
    fn the_email_is_a_string_on_some_documents_and_a_list_on_others() {
        use super::StringOrList;
        for (json, expected) in [
            (r#""a@b.c""#, StringOrList::One("a@b.c".into())),
            (
                r#"["a@b.c","d@e.f"]"#,
                StringOrList::Many(vec!["a@b.c".into(), "d@e.f".into()]),
            ),
        ] {
            let island = format!(
                r#"{{"@type":"Corporation","@context":"c","@id":"i","email":{json},
                   "legalName":"L","logo":"l","name":"Anker","sameAs":[],"url":"u"}}"#
            );
            let LdJson::Corporation(corp) =
                serde_json::from_str::<LdJson>(&island).expect("both spellings read")
            else {
                panic!("expected a Corporation");
            };
            assert_eq!(corp.email, Some(expected));
        }
    }
}
