use html_extractor::serde_json::{self, Value};
use serde::de::DeserializeOwned;

/// One `script[type="application/ld+json"]`. The SEO app marks its script with `data-seoant`
/// attributes (274 of 274 pages) and the reviews app with an `id` (126 pages); the rest carry
/// only `type`. Each attribute is optional, any other one fails the page.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LdJsonScript {
    #[serde(rename = "type")]
    pub script_type: String,
    #[serde(rename = "data-seoant")]
    pub seoant: Option<String>,
    #[serde(rename = "data-seoant-version")]
    pub seoant_version: Option<String>,
    pub id: Option<String>,
    pub data: LdJson,
}

/// The island types seen in the corpus. Every page carries 3 to 5 islands written by four
/// different apps, and `@type` alone cannot tell them apart: three of them are a `Product`. So
/// the island is told apart by what only it has, and each kind then deserializes strictly into
/// its own struct:
///
/// - `@graph` and no `@type`: the SEO app's [`Graph`] envelope (274 pages).
/// - `Organization`: the theme's (274).
/// - `ProductGroup`: the theme's, for a product with a variant list (248).
/// - `Product` with `review`: the reviews app's [`ReviewedProduct`] (126).
/// - `Product` with `@id`: the theme's, for a product without one (24).
/// - Any other `Product`: the merchant feed app's [`FeedProduct`] (274).
///
/// Not derived as an untagged enum: that reports every failure as "did not match any variant",
/// where this names the island's own offending field.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(try_from = "Value")]
pub enum LdJson {
    Graph(Graph),
    Organization(Organization),
    ProductGroup(ProductGroup),
    Product(Product),
    /// Boxed, with [`LdJson::ReviewedProduct`]: these two are several times the size of the rest.
    FeedProduct(Box<FeedProduct>),
    ReviewedProduct(Box<ReviewedProduct>),
}

impl TryFrom<Value> for LdJson {
    type Error = String;

    fn try_from(island: Value) -> Result<LdJson, String> {
        fn strict<T: DeserializeOwned>(island: Value) -> Result<T, String> {
            serde_json::from_value(island).map_err(|e| e.to_string())
        }

        let Some(object) = island.as_object() else {
            return Err("an ld+json island is not an object".to_string());
        };

        if object.contains_key("@graph") {
            return strict(island).map(LdJson::Graph);
        }

        match object.get("@type").and_then(Value::as_str) {
            Some("Organization") => strict(island).map(LdJson::Organization),
            Some("ProductGroup") => strict(island).map(LdJson::ProductGroup),
            Some("Product") if object.contains_key("review") => {
                strict(island).map(|product| LdJson::ReviewedProduct(Box::new(product)))
            }
            Some("Product") if object.contains_key("@id") => strict(island).map(LdJson::Product),
            Some("Product") => strict(island).map(|product| LdJson::FeedProduct(Box::new(product))),
            Some(other) => Err(format!("unknown variant `{other}`, an unseen @type")),
            None => Err("an ld+json island has neither `@graph` nor `@type`".to_string()),
        }
    }
}

/// The SEO app's envelope, which has no `@type` of its own. Its members restate the page: a
/// `ProductGroup` (248) or `Product` (26) with the variants' offers, a `WebPage`, a
/// `BreadcrumbList`, an `Organization` and, on 34 pages, a `VideoObject`. They are app-authored
/// and differ in shape from member to member and page to page (a variant's `color`, `size` and
/// `material` come and go, and its prices are floats, unlike every other island's strings), so
/// they are carried as raw JSON. Nothing reads them: every fact in them is stated by the record
/// or by the theme's island, which is checked against the record.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@graph")]
    pub graph: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@type")]
    pub island_type: String,
    pub name: String,
    pub logo: String,
    pub url: String,
}

/// The theme's island for a product with a variant list: 248 pages, 48 of them with a single
/// variant. The product of the 2 pages that has no price (`s5-all-flash-nas`) has none.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductGroup {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub island_type: String,
    pub brand: Brand,
    pub category: String,
    pub description: String,
    #[serde(rename = "hasVariant")]
    pub has_variant: Vec<GroupVariant>,
    pub name: String,
    #[serde(rename = "productGroupID")]
    pub product_group_id: String,
    /// The product page with its locale prefix (`/de`), percent-encoded. The prefix is not
    /// reliable: `de-products-kabeltasche.html`, a `de_DE` page, has none.
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupVariant {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub variant_type: String,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    /// Absent where the variant has no SKU (18 of the 1360 variants in a group).
    pub sku: Option<String>,
}

/// The theme's island for a product that is not a variant list: 24 pages, each a single variant.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub island_type: String,
    pub brand: Brand,
    pub category: String,
    pub description: String,
    pub image: String,
    pub name: String,
    pub offers: Offer,
    pub sku: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    #[serde(rename = "@type")]
    pub brand_type: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub offer_type: String,
    pub availability: String,
    /// A decimal string in major units, such as `129.00`.
    pub price: String,
    #[serde(rename = "priceCurrency")]
    pub price_currency: String,
    /// The product URL with a `variant` query parameter.
    pub url: String,
}

/// The merchant feed app's `Product`, on every page, describing one variant of the product (the
/// one whose SKU it carries, which is the first on 262 pages) in the shape Google's merchant
/// listings take: shipping, returns and a manufacturer, all of them constant across the corpus
/// and so typed. It is not a source of facts here: its `offers.price` is that variant's, not the
/// product's, and on 18 pages it is malformed (see [`FeedOffer::price`]).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeedProduct {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@type")]
    pub island_type: String,
    pub name: String,
    pub description: String,
    pub image: String,
    pub brand: Brand,
    pub manufacturer: Manufacturer,
    /// `null` on 2 pages: the 2 whose product has no variant SKU (`s5-all-flash-nas`).
    pub sku: Option<String>,
    /// `null` on the same 2 pages, otherwise equal to `sku`.
    pub mpn: Option<String>,
    pub offers: FeedOffer,
    /// Empty on all 274 pages, so the shape of an element is unknown and one fails the page.
    pub additional_property: Vec<UnseenProperty>,
    /// `null` on all 274 pages.
    pub category: Option<String>,
    pub audience: Audience,
}

/// An `additionalProperty` element; none was ever seen, and this type has no value, so
/// deserializing one is an error.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub enum UnseenProperty {}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manufacturer {
    #[serde(rename = "@type")]
    pub manufacturer_type: String,
    pub name: String,
}

/// `audienceType` is `null` on all 274 pages.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Audience {
    #[serde(rename = "@type")]
    pub audience_type_name: String,
    pub audience_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeedOffer {
    #[serde(rename = "@type")]
    pub offer_type: String,
    pub url: String,
    pub price_currency: String,
    /// The price in major units, a decimal string such as `129.00` on 256 pages. On 18 (9 per
    /// locale) the app writes a thousands separator before the decimal point as well, as in
    /// `4.159.00` for 4159 euros, so it is not a decimal number and is kept as text.
    pub price: String,
    pub price_valid_until: String,
    pub availability: String,
    pub item_condition: String,
    pub seller: Seller,
    pub shipping_details: ShippingDetails,
    pub has_merchant_return_policy: ReturnPolicy,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seller {
    #[serde(rename = "@type")]
    pub seller_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShippingDetails {
    #[serde(rename = "@type")]
    pub details_type: String,
    pub shipping_destination: ShippingDestination,
    pub delivery_time: DeliveryTime,
}

/// 25 countries, the same on all 274 pages.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShippingDestination {
    #[serde(rename = "@type")]
    pub destination_type: String,
    pub address_country: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryTime {
    #[serde(rename = "@type")]
    pub delivery_type: String,
    pub handling_time: DayRange,
    pub transit_time: DayRange,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DayRange {
    #[serde(rename = "@type")]
    pub range_type: String,
    pub min_value: u32,
    pub max_value: u32,
    pub unit_code: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReturnPolicy {
    #[serde(rename = "@type")]
    pub policy_type: String,
    pub applicable_country: String,
    pub return_policy_category: String,
    pub merchant_return_days: u32,
    pub return_method: String,
}

/// The reviews app's `Product`, on 126 pages: only the reviews matter to it, but it restates
/// the product and an offer. `aggregateRating` is absent on the 16 of those pages whose
/// reviews carry no count.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewedProduct {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@type")]
    pub island_type: String,
    pub name: String,
    pub description: String,
    pub image: String,
    pub brand: Brand,
    pub sku: String,
    /// `null` on all 126 pages.
    pub mpn: Option<String>,
    pub aggregate_rating: Option<AggregateRating>,
    pub review: Vec<Review>,
    pub offers: ReviewedOffer,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AggregateRating {
    #[serde(rename = "@type")]
    pub rating_type: String,
    pub rating_value: String,
    pub best_rating: String,
    pub review_count: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Review {
    #[serde(rename = "@type")]
    pub review_type: String,
    pub review_rating: ReviewRating,
    pub author: Author,
    pub review_body: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewRating {
    #[serde(rename = "@type")]
    pub rating_type: String,
    pub rating_value: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
    #[serde(rename = "@type")]
    pub author_type: String,
    pub name: String,
}

/// The reviews app's offer: the same keys as the feed app's, less shipping, returns and the
/// seller's `url`. Its `price` has the same format problem as the feed's on the pages that have
/// both, and is read by nothing.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewedOffer {
    #[serde(rename = "@type")]
    pub offer_type: String,
    pub url: String,
    pub price_currency: String,
    pub price: String,
    pub price_valid_until: String,
    pub item_condition: String,
    pub availability: String,
    pub seller: ReviewedSeller,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedSeller {
    #[serde(rename = "@type")]
    pub seller_type: String,
    pub name: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{LdJson, LdJsonScript};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn organization() -> Value {
        json!({
            "@context": "http://schema.org",
            "@type": "Organization",
            "name": "Minisforum EU",
            "logo": "https://minisforumpc.eu/cdn/shop/files/Logo.svg?v=1786004388&width=500",
            "url": "https://minisforumpc.eu"
        })
    }

    fn group_variant(id: u64, price: &str, sku: &str, kind: &str, availability: &str) -> Value {
        json!({
            "@id": format!("/products/minisforum-u700-refurbished?variant={id}#variant"),
            "@type": "Product",
            "image": "https://minisforumpc.eu/cdn/shop/files/image13.png?v=1770796122&width=1920",
            "name": format!("MINISFORUM U700 Refurbished - Intel® Core™ i5-5257U Processor / {kind}"),
            "offers": {
                "@id": format!("/products/minisforum-u700-refurbished?variant={id}#offer"),
                "@type": "Offer",
                "availability": availability,
                "price": price,
                "priceCurrency": "EUR",
                "url": format!("https://minisforumpc.eu/products/minisforum-u700-refurbished?variant={id}")
            },
            "sku": sku
        })
    }

    /// The theme's island of `super::productjson::tests::product()`.
    pub(crate) fn product_group() -> Value {
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/minisforum-u700-refurbished#product",
            "@type": "ProductGroup",
            "brand": {"@type": "Brand", "name": "Refurbished Mini PC"},
            "category": "Hard Drives",
            "description": "",
            "hasVariant": [
                group_variant(53296398172526, "129.00", "U70082REU", "8GB RAM + 256GB SSD", "http://schema.org/InStock"),
                group_variant(53753234751854, "99.00", "U70000REU", "Barebone", "http://schema.org/OutOfStock")
            ],
            "name": "MINISFORUM U700 Refurbished",
            "productGroupID": "14959267250542",
            "url": "https://minisforumpc.eu/products/minisforum-u700-refurbished"
        })
    }

    /// The theme's island for a product that is not a variant list, with the first variant's offer.
    pub(crate) fn product() -> Value {
        let group = product_group();
        json!({
            "@context": "http://schema.org/",
            "@id": "/products/minisforum-u700-refurbished#product",
            "@type": "Product",
            "brand": group["brand"],
            "category": "Hard Drives",
            "description": "",
            "image": group["hasVariant"][0]["image"],
            "name": group["name"],
            "offers": group["hasVariant"][0]["offers"],
            "sku": "U70082REU",
            "url": group["url"]
        })
    }

    /// The merchant feed app's island, as rendered for the first variant.
    pub(crate) fn feed_product() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "Product",
            "name": "MINISFORUM U700 Refurbished",
            "description": "",
            "image": "//minisforumpc.eu/cdn/shop/files/image13_grande.png?v=1770796122",
            "brand": {"@type": "Brand", "name": "Minisforum"},
            "manufacturer": {"@type": "Organization", "name": "Minisforum"},
            "sku": "U70082REU",
            "mpn": "U70082REU",
            "offers": {
                "@type": "Offer",
                "url": "https://minisforumpc.eu/products/minisforum-u700-refurbished",
                "priceCurrency": "EUR",
                "price": "129.00",
                "priceValidUntil": "2026-11-14",
                "availability": "https://schema.org/InStock",
                "itemCondition": "https://schema.org/NewCondition",
                "seller": {
                    "@type": "Organization",
                    "name": "Minisforum EU",
                    "url": "https://minisforumpc.eu"
                },
                "shippingDetails": {
                    "@type": "OfferShippingDetails",
                    "shippingDestination": {
                        "@type": "DefinedRegion",
                        "addressCountry": ["DE", "AT", "FR"]
                    },
                    "deliveryTime": {
                        "@type": "ShippingDeliveryTime",
                        "handlingTime": {"@type": "QuantitativeValue", "minValue": 1, "maxValue": 2, "unitCode": "DAY"},
                        "transitTime": {"@type": "QuantitativeValue", "minValue": 3, "maxValue": 7, "unitCode": "DAY"}
                    }
                },
                "hasMerchantReturnPolicy": {
                    "@type": "MerchantReturnPolicy",
                    "applicableCountry": "DE",
                    "returnPolicyCategory": "https://schema.org/MerchantReturnFiniteReturnWindow",
                    "merchantReturnDays": 30,
                    "returnMethod": "https://schema.org/ReturnByMail"
                }
            },
            "additionalProperty": [],
            "category": null,
            "audience": {"@type": "Audience", "audienceType": null}
        })
    }

    /// The SEO app's envelope, trimmed to its WebPage, BreadcrumbList and Organization.
    pub(crate) fn graph() -> Value {
        json!({
            "@context": "https://schema.org",
            "@graph": [
                {
                    "@type": "WebPage",
                    "@id": "https://minisforumpc.eu/products/minisforum-u700-refurbished#webpage",
                    "mainEntity": {"@id": "https://minisforumpc.eu/products/minisforum-u700-refurbished#product-group"},
                    "url": "https://minisforumpc.eu/products/minisforum-u700-refurbished",
                    "breadcrumb": {"@id": "https://minisforumpc.eu/products/minisforum-u700-refurbished#breadcrumb"}
                },
                {
                    "@type": "BreadcrumbList",
                    "@id": "https://minisforumpc.eu/products/minisforum-u700-refurbished#breadcrumb",
                    "itemListElement": [
                        {"@type": "ListItem", "position": 1, "name": "Home", "item": "https://minisforumpc.eu"}
                    ]
                },
                {
                    "@type": "Organization",
                    "@id": "https://minisforumpc.eu/#organization",
                    "name": "Minisforum EU",
                    "url": "https://minisforumpc.eu"
                }
            ]
        })
    }

    /// The reviews app's island.
    pub(crate) fn reviewed_product() -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "Product",
            "name": "MINISFORUM U700 Refurbished",
            "description": "",
            "image": "//minisforumpc.eu/cdn/shop/files/image13.png?v=1770796122",
            "brand": {"@type": "Brand", "name": "Minisforum"},
            "sku": "U70082REU",
            "mpn": null,
            "aggregateRating": {
                "@type": "AggregateRating",
                "ratingValue": "5.0",
                "bestRating": "5",
                "reviewCount": 1
            },
            "review": [{
                "@type": "Review",
                "reviewRating": {"@type": "Rating", "ratingValue": 5},
                "author": {"@type": "Person", "name": "Jan"},
                "reviewBody": "Gut."
            }],
            "offers": {
                "@type": "Offer",
                "url": "https://minisforumpc.eu/products/minisforum-u700-refurbished",
                "priceCurrency": "EUR",
                "price": "129.00",
                "priceValidUntil": "2026-11-14",
                "itemCondition": "https://schema.org/NewCondition",
                "availability": "https://schema.org/InStock",
                "seller": {"@type": "Organization", "name": "Minisforum EU"}
            }
        })
    }

    fn deserialize(island: &Value) -> Result<LdJson, serde_json::Error> {
        serde_json::from_value(island.clone())
    }

    #[test]
    fn every_island_kind_deserializes_into_its_own_variant() {
        assert!(matches!(deserialize(&graph()), Ok(LdJson::Graph(_))));
        assert!(matches!(
            deserialize(&organization()),
            Ok(LdJson::Organization(_))
        ));
        assert!(matches!(
            deserialize(&product_group()),
            Ok(LdJson::ProductGroup(_))
        ));
        assert!(matches!(deserialize(&product()), Ok(LdJson::Product(_))));
        assert!(matches!(
            deserialize(&feed_product()),
            Ok(LdJson::FeedProduct(_))
        ));
        assert!(matches!(
            deserialize(&reviewed_product()),
            Ok(LdJson::ReviewedProduct(_))
        ));
    }

    #[test]
    fn the_feed_app_may_leave_the_identifiers_null() {
        let mut island = feed_product();
        island["sku"] = Value::Null;
        island["mpn"] = Value::Null;

        assert!(matches!(deserialize(&island), Ok(LdJson::FeedProduct(_))));
    }

    /// As on `products-minisforum-ms-s1-max-mini-pc.html`.
    #[test]
    fn a_feed_price_with_a_thousands_separator_is_kept_as_text() {
        let mut island = feed_product();
        island["offers"]["price"] = json!("4.159.00");

        let Ok(LdJson::FeedProduct(product)) = deserialize(&island) else {
            panic!("a feed product");
        };
        assert_eq!(product.offers.price, "4.159.00");
    }

    #[test]
    fn a_review_island_without_an_aggregate_rating_deserializes() {
        let mut island = reviewed_product();
        island
            .as_object_mut()
            .expect("an object")
            .remove("aggregateRating");

        assert!(matches!(
            deserialize(&island),
            Ok(LdJson::ReviewedProduct(product)) if product.aggregate_rating.is_none()
        ));
    }

    #[test]
    fn an_unseen_island_type_fails() {
        for island_type in ["BreadcrumbList", "FAQPage"] {
            let error = deserialize(&json!({"@type": island_type}))
                .expect_err("the island type is not typed");
            assert!(
                error
                    .to_string()
                    .contains(&format!("unknown variant `{island_type}`")),
                "{error}"
            );
        }
    }

    #[test]
    fn an_island_without_a_type_or_a_graph_fails() {
        assert!(deserialize(&json!({"@context": "https://schema.org"})).is_err());
        assert!(deserialize(&json!(["Product"])).is_err());
    }

    #[test]
    fn an_unknown_key_in_an_island_fails_naming_the_key() {
        for island in [
            graph(),
            organization(),
            product_group(),
            product(),
            feed_product(),
            reviewed_product(),
        ] {
            let mut island = island;
            island["gtin"] = json!("1");

            let error = deserialize(&island).expect_err("an unknown key");
            assert!(
                error.to_string().contains("unknown field `gtin`"),
                "{error}"
            );
        }
    }

    #[test]
    fn an_unknown_key_below_an_island_fails() {
        let mut island = feed_product();
        island["offers"]["shippingDetails"]["deliveryTime"]["handlingTime"]["extra"] = json!(1);
        assert!(deserialize(&island).is_err());

        let mut island = product_group();
        island["hasVariant"][0]["offers"]["itemCondition"] = json!("new");
        assert!(deserialize(&island).is_err());
    }

    #[test]
    fn an_additional_property_fails() {
        let mut island = feed_product();
        island["additionalProperty"] = json!([{"@type": "PropertyValue", "name": "a"}]);

        assert!(deserialize(&island).is_err());
    }

    #[test]
    fn the_graph_keeps_its_members_as_they_are() {
        let Ok(LdJson::Graph(graph)) = deserialize(&graph()) else {
            panic!("a graph");
        };
        assert_eq!(graph.graph.len(), 3);
        assert_eq!(graph.graph[0]["@type"], json!("WebPage"));
    }

    #[test]
    fn the_script_attributes_are_optional_but_closed() {
        let script = |extra: Value| {
            let mut script = json!({"type": "application/ld+json", "data": organization()});
            for (key, value) in extra.as_object().expect("an object") {
                script[key] = value.clone();
            }
            serde_json::from_value::<LdJsonScript>(script)
        };

        assert!(script(json!({})).is_ok());
        assert!(
            script(json!({"data-seoant": "1", "data-seoant-version": "v2"})).is_ok(),
            "the SEO app's attributes"
        );
        assert!(script(json!({"id": "a2r-03hu34xd-uhj7389-x"})).is_ok());
        assert!(script(json!({"nonce": "x"})).is_err());
    }
}
