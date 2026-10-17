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
    ImageObject(ImageObject),
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

    pub url: String,
    pub brand: Brand,

    pub offers: Vec<Offer>,
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

    pub price: f64,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,

    #[serde(rename = "highPrice")]
    pub high_price: String,

    #[serde(rename = "lowPrice")]
    pub low_price: String,
    #[serde(rename = "offerCount")]
    pub offer_count: i64,
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
pub struct ImageObject {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "contentUrl")]
    pub content_url: String,
    pub license: String,
    #[serde(rename = "acquireLicensePage")]
    pub acquire_license_page: String,
    #[serde(rename = "creditText")]
    pub credit_text: String,
    pub creator: Creator,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Creator {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaqPage {
    #[serde(rename = "@context")]
    pub context: String,

    #[serde(rename = "mainEntity")]
    pub main_entity: Option<Vec<FaqQuestion>>,
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
        "brand":{"@type":"Brand","name":"Anker"},"description":"d","image":["i"],"name":"n",
        "offers":[{"@type":"Offer","highPrice":"2","lowPrice":"1","offerCount":2,"price":1.5,
        "priceCurrency":"EUR"}],"url":"u"}"#;

    #[test]
    fn the_offers_are_a_range_spelled_two_ways() {
        let LdJson::Product(product) =
            serde_json::from_str::<LdJson>(PRODUCT).expect("the Product island deserializes")
        else {
            panic!("expected a Product");
        };
        assert_eq!(product.offers.len(), 1);
        assert_eq!(product.offers[0].price, 1.5);
        assert_eq!(product.offers[0].high_price, "2");
        assert_eq!(product.offers[0].low_price, "1");
        assert_eq!(product.offers[0].offer_count, 2);
    }

    #[test]
    fn a_single_offer_object_is_not_accepted() {
        let single = PRODUCT
            .replace("\"offers\":[", "\"offers\":")
            .replace("}],\"url\"", "},\"url\"");
        assert!(serde_json::from_str::<LdJson>(&single).is_err());
    }

    #[test]
    fn the_image_object_dispatches_and_carries_its_creator() {
        let island = r#"{"@type":"ImageObject","@context":"https://schema.org",
            "acquireLicensePage":"a","contentUrl":"c",
            "creator":{"@type":"Organization","name":"Anker"},"creditText":"t","license":"l"}"#;

        let LdJson::ImageObject(image) =
            serde_json::from_str::<LdJson>(island).expect("the ImageObject dispatches")
        else {
            panic!("expected an ImageObject");
        };
        assert_eq!(image.creator.name, "Anker");
        assert_eq!(image.creator.kind, "Organization");
    }

    #[test]
    fn an_unknown_type_is_rejected_rather_than_ignored() {
        assert!(
            serde_json::from_str::<LdJson>(r#"{"@type":"VideoObject","@context":"c"}"#).is_err()
        );
    }

    #[test]
    fn an_empty_faq_island_still_deserializes() {
        let bare = r#"{"@type":"FAQPage","@context":"https://schema.org"}"#;
        let LdJson::FaqPage(faq) =
            serde_json::from_str::<LdJson>(bare).expect("an empty FAQ island is legal here")
        else {
            panic!("expected a FaqPage");
        };
        assert!(faq.main_entity.is_none());
    }
}
