use html_extractor::serde_json::Value;

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

    /// Absent on the few accessory pages that carry no image at all, e.g. the Sport X20 eartips
    /// kits (`a3968z21-94`).
    pub image: Option<String>,

    pub url: String,
    pub brand: Brand,

    pub offers: Offer,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    #[serde(rename = "@context")]
    pub context: String,
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

    pub email: Option<String>,

    pub description: Option<String>,

    pub telephone: Option<String>,
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
    pub context: String,
    #[serde(rename = "@type")]
    pub kind: String,

    /// `None` on the crumb the storefront renders for an unresolved collection, whose `item`
    /// points at `collections/undefined`.
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
    #[serde(rename = "@context")]
    pub context: String,
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
    pub context: String,
    #[serde(rename = "@type")]
    pub kind: String,
    /// A plain string, or a Lexical editor state the CMS serializes as-is; left untyped on
    /// purpose.
    pub text: Value,
}

#[cfg(test)]
mod tests {
    use super::{FaqAnswer, LdJson, LdJsonScript};
    use html_extractor::serde_json;

    #[test]
    fn the_island_is_one_script_holding_an_array() {
        let island = r#"{"data":[
            {"@type":"Corporation","@context":"c","logo":"l","name":"Anker","sameAs":[],"url":"u"},
            {"@type":"BreadcrumbList","@context":"c","itemListElement":[
                {"@context":"c","@type":"ListItem","item":"i","name":"Home","position":1}]}]}"#;

        let island: LdJsonScript = serde_json::from_str(island).expect("the array deserializes");
        assert_eq!(island.data.len(), 2);
        assert!(matches!(island.data[0], LdJson::Corporation(_)));
    }

    #[test]
    fn the_price_is_a_string_and_the_image_is_not_a_list() {
        let product = r#"{"@type":"Product","@context":"c",
            "brand":{"@context":"c","@type":"Brand","name":"Anker"},
            "description":"d","image":"i","name":"n","offers":{"@context":"c","@type":"Offer",
            "availability":"https://schema.org/InStock","price":"99.99","priceCurrency":"CAD"},
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
    fn an_imageless_product_reads() {
        let product = r#"{"@type":"Product","@context":"c",
            "brand":{"@context":"c","@type":"Brand","name":"Anker"},
            "description":"d","name":"n","offers":{"@context":"c","@type":"Offer",
            "availability":"https://schema.org/InStock","price":"9.99","priceCurrency":"CAD"},
            "url":"u"}"#;

        let LdJson::Product(product) =
            serde_json::from_str::<LdJson>(product).expect("the image-less Product deserializes")
        else {
            panic!("expected a Product");
        };
        assert!(product.image.is_none());
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
    fn an_unresolved_collection_crumb_has_no_name() {
        let list = r#"{"@type":"BreadcrumbList","@context":"c","itemListElement":[
            {"@context":"c","@type":"ListItem","position":2,
             "item":"https://www.anker.com/ca-fr/collections/undefined?ref=breadcrumb"}]}"#;

        let LdJson::BreadcrumbList(list) =
            serde_json::from_str::<LdJson>(list).expect("the nameless crumb deserializes")
        else {
            panic!("expected a BreadcrumbList");
        };
        assert!(list.item_list_element[0].name.is_none());
    }

    #[test]
    fn the_soundcore_corporation_tier_reads() {
        let soundcore = r#"{"@type":"Corporation","@context":"c","logo":"l","name":"soundcore",
            "sameAs":[],"url":"u","description":"d","telephone":"+1 (800) 988 7973"}"#;

        let LdJson::Corporation(corp) =
            serde_json::from_str::<LdJson>(soundcore).expect("the soundcore tier deserializes")
        else {
            panic!("expected a Corporation");
        };
        assert_eq!(corp.telephone.as_deref(), Some("+1 (800) 988 7973"));
        assert_eq!(corp.description.as_deref(), Some("d"));
    }

    #[test]
    fn an_answer_text_is_a_string_or_a_cms_tree() {
        let plain: FaqAnswer =
            serde_json::from_str(r#"{"@context":"c","@type":"Answer","text":"Yes."}"#)
                .expect("plain reads");
        let rich: FaqAnswer = serde_json::from_str(
            r#"{"@context":"c","@type":"Answer","text":{"root":{"type":"root"}}}"#,
        )
        .expect("the CMS tree reads");

        assert!(plain.text.is_string());
        assert!(rich.text.is_object());
    }
}
