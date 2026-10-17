#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonItem {
    pub data: LdJson,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum LdJson {
    Product(Product),
    Organization(Organization),
    BreadcrumbList(BreadcrumbList),
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

    #[serde(rename = "aggregateRating")]
    pub aggregate_rating: Option<AggregateRating>,

    pub review: Option<Vec<Review>>,
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
    pub url: String,

    pub sku: Option<String>,
    /// Present only on a marked-down variant, as its strikethrough price.
    #[serde(rename = "priceSpecification")]
    pub price_specification: Option<PriceSpecification>,
    #[serde(rename = "hasMerchantReturnPolicy")]
    pub has_merchant_return_policy: MerchantReturnPolicy,
    #[serde(rename = "shippingDetails")]
    pub shipping_details: ShippingDetails,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceSpecification {
    #[serde(rename = "@type")]
    pub kind: String,
    pub price: f64,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    #[serde(rename = "priceType")]
    pub price_type: String,
    #[serde(rename = "valueAddedTaxIncluded")]
    pub value_added_tax_included: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MerchantReturnPolicy {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "applicableCountry")]
    pub applicable_country: String,
    #[serde(rename = "returnPolicyCountry")]
    pub return_policy_country: String,
    #[serde(rename = "returnPolicyCategory")]
    pub return_policy_category: String,
    #[serde(rename = "merchantReturnDays")]
    pub merchant_return_days: i64,
    #[serde(rename = "returnMethod")]
    pub return_method: String,
    #[serde(rename = "returnFees")]
    pub return_fees: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShippingDetails {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "shippingRate")]
    pub shipping_rate: ShippingRate,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShippingRate {
    #[serde(rename = "@type")]
    pub kind: String,
    pub value: f64,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateRating {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "ratingValue")]
    pub rating_value: f64,
    #[serde(rename = "reviewCount")]
    pub review_count: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    #[serde(rename = "@type")]
    pub kind: String,
    pub author: Author,
    #[serde(rename = "datePublished")]
    pub date_published: String,
    #[serde(rename = "reviewBody")]
    pub review_body: String,
    #[serde(rename = "reviewRating")]
    pub review_rating: ReviewRating,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRating {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "ratingValue")]
    pub rating_value: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    pub name: String,
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

    pub item: Option<String>,
    pub position: i64,
}

#[cfg(test)]
mod tests {
    use super::LdJson;
    use html_extractor::serde_json;

    #[test]
    fn the_final_crumb_carries_no_item() {
        let trail = r#"{"@type":"BreadcrumbList","@context":"c","itemListElement":[
            {"@type":"ListItem","item":"i","name":"Home","position":1},
            {"@type":"ListItem","name":"This page","position":2}]}"#;

        let LdJson::BreadcrumbList(trail) =
            serde_json::from_str::<LdJson>(trail).expect("a linkless final crumb is the norm here")
        else {
            panic!("expected a BreadcrumbList");
        };
        assert_eq!(trail.item_list_element[0].item.as_deref(), Some("i"));
        assert!(trail.item_list_element[1].item.is_none());
    }

    #[test]
    fn the_organisation_is_not_spelled_corporation() {
        let org = r#"{"@type":"Organization","@context":"c","logo":"l","name":"Anker",
            "sameAs":["s"],"url":"u"}"#;
        assert!(matches!(
            serde_json::from_str::<LdJson>(org).expect("Organization dispatches"),
            LdJson::Organization(_)
        ));

        let corp = org.replace("Organization", "Corporation");
        assert!(
            serde_json::from_str::<LdJson>(&corp).is_err(),
            "this storefront does not render a Corporation"
        );
    }

    #[test]
    fn the_aggregate_rating_reads_as_both_a_float_and_an_integer() {
        for (spelling, expected) in [("4.5", 4.5_f64), ("5", 5.0_f64)] {
            let product = PRODUCT.replace(
                r#""ratingValue":4.5"#,
                &format!(r#""ratingValue":{spelling}"#),
            );
            let LdJson::Product(product) =
                serde_json::from_str::<LdJson>(&product).expect("both spellings read")
            else {
                panic!("expected a Product");
            };
            let rating = product.aggregate_rating.expect("this fixture has one");
            assert_eq!(rating.rating_value, expected, "rating spelled {spelling}");
        }
    }

    #[test]
    fn the_offer_carries_its_return_policy_and_shipping_rate() {
        let LdJson::Product(product) =
            serde_json::from_str::<LdJson>(PRODUCT).expect("the Product deserializes")
        else {
            panic!("expected a Product");
        };
        let offer = &product.offers[0];
        assert_eq!(offer.has_merchant_return_policy.merchant_return_days, 30);
        assert_eq!(offer.shipping_details.shipping_rate.currency, "JPY");
        assert_eq!(offer.sku.as_deref(), Some("A1"));
        assert_eq!(product.review.expect("reviews")[0].author.name, "A");
    }

    #[test]
    fn an_offer_without_a_sku_still_deserializes() {
        let without = PRODUCT.replace(r#""sku":"A1","#, "");
        let LdJson::Product(product) =
            serde_json::from_str::<LdJson>(&without).expect("sku is optional")
        else {
            panic!("expected a Product");
        };
        assert!(product.offers[0].sku.is_none());
    }

    #[test]
    fn a_marked_down_offer_carries_its_strikethrough_price() {
        let marked_down = PRODUCT.replace(
            r#""sku":"A1","#,
            r#""sku":"A1","priceSpecification":{"@type":"UnitPriceSpecification","price":2990.0,
                "priceCurrency":"JPY","priceType":"https://schema.org/StrikethroughPrice",
                "valueAddedTaxIncluded":true},"#,
        );
        let LdJson::Product(product) =
            serde_json::from_str::<LdJson>(&marked_down).expect("priceSpecification reads")
        else {
            panic!("expected a Product");
        };
        let specification = product.offers[0]
            .price_specification
            .as_ref()
            .expect("this offer is marked down");
        assert_eq!(specification.price, 2990.0);
        assert_eq!(
            specification.price_type,
            "https://schema.org/StrikethroughPrice"
        );

        let LdJson::Product(product) =
            serde_json::from_str::<LdJson>(PRODUCT).expect("the fixture deserializes")
        else {
            panic!("expected a Product");
        };
        assert!(product.offers[0].price_specification.is_none());
    }

    const PRODUCT: &str = r#"{"@type":"Product","@context":"https://schema.org",
        "brand":{"@type":"Brand","name":"Anker"},"description":"d","image":["i"],"name":"n",
        "url":"u",
        "aggregateRating":{"@type":"AggregateRating","ratingValue":4.5,"reviewCount":10},
        "review":[{"@type":"Review","author":{"@type":"Person","name":"A"},
            "datePublished":"2024-01-01","reviewBody":"b",
            "reviewRating":{"@type":"Rating","ratingValue":5}}],
        "offers":[{"@type":"Offer","availability":"https://schema.org/InStock","price":1980.0,
            "priceCurrency":"JPY","sku":"A1","url":"u",
            "hasMerchantReturnPolicy":{"@type":"MerchantReturnPolicy","applicableCountry":"JP",
                "returnPolicyCountry":"JP","returnPolicyCategory":"c","merchantReturnDays":30,
                "returnMethod":"m","returnFees":"f"},
            "shippingDetails":{"@type":"OfferShippingDetails",
                "shippingRate":{"@type":"MonetaryAmount","value":0.0,"currency":"JPY"}}}]}"#;
}
