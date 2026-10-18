use super::raw;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum NormalizeError {
    #[error("ld+json offer price `{value}` is not a number: {source}")]
    OfferPriceNotANumber {
        value: String,
        source: std::num::ParseFloatError,
    },

    #[error("ld+json offer price `{amount}` `{currency}` is not valid money: {source}")]
    OfferPriceNotMoney {
        amount: f64,
        currency: String,
        source: money::MoneyError,
    },

    #[error("publishedAt `{value}` is not an RFC 3339 instant: {source}")]
    PublishedAtNotAnInstant {
        value: String,
        source: chrono::ParseError,
    },

    #[error("no `Product` ld+json island on the document")]
    MissingProductIsland,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkerCaNormalizedDocument {
    pub ld_json: ldjson::LdJson,
    pub product: product::ProductRecord,
}

impl AnkerCaNormalizedDocument {
    pub fn from_raw(raw: raw::AnkerCaRawDocument) -> Result<Self, NormalizeError> {
        let ld_json = ldjson::LdJson::from_raw(raw.ld_json)?;
        let product = product::ProductRecord::from_raw(raw.flight.product)?;

        Ok(AnkerCaNormalizedDocument { ld_json, product })
    }
}

pub mod ldjson {

    use super::NormalizeError;
    use super::raw;
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
        pub name: String,
        pub description: String,

        /// A single image URL despite the plural field name. `None` on pages with no image.
        pub images: Option<String>,

        pub url: String,
        pub brand: String,

        pub offer: Offer,
    }

    impl Product {
        pub(crate) fn from_raw(raw: raw::ldjsonitem::Product) -> Result<Product, NormalizeError> {
            let offer = Offer::from_raw(raw.offers)?;

            Ok(Product {
                name: raw.name,
                description: raw.description,
                images: raw.image,
                url: raw.url,
                brand: raw.brand.name,
                offer,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Offer {
        pub availability: retailer_sauce::availability::Availability,

        pub price: money::Money,
    }

    impl Offer {
        fn from_raw(raw: raw::ldjsonitem::Offer) -> Result<Offer, NormalizeError> {
            let price = price(&raw.price, &raw.price_currency)?;

            Ok(Offer {
                availability: retailer_sauce::availability::Availability::from_schema_org(
                    raw.availability,
                ),
                price,
            })
        }
    }

    fn price(amount: &str, currency: &str) -> Result<money::Money, NormalizeError> {
        let amount =
            amount
                .parse::<f64>()
                .map_err(|source| NormalizeError::OfferPriceNotANumber {
                    value: amount.to_string(),
                    source,
                })?;

        money::Money::from_raw(amount, currency).map_err(|source| {
            NormalizeError::OfferPriceNotMoney {
                amount,
                currency: currency.to_string(),
                source,
            }
        })
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

        pub email: Option<String>,
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
                email: raw.email,
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
        pub name: Option<String>,

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
        pub answer: Value,
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
}

pub mod product {
    use super::NormalizeError;
    use super::raw;
    use chrono::{DateTime, Utc};
    use money::Money;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ProductRecord {
        pub id: String,
        pub handle: String,
        pub title: String,
        pub vendor: String,
        pub product_type: String,
        pub description: String,
        // Commented out: the same copy with markup; adds no value for now.
        // pub description_html: String,
        pub available_for_sale: bool,
        pub created_at: String,
        pub updated_at: String,
        pub published_at: DateTime<Utc>,
        // Commented out: not needed for now.
        // pub tags: Vec<String>,
        pub seo: ProductSeo,

        pub price: Money,
        /// `None` when the storefront's zero sentinel indicates no markdown.
        pub compare_at_price: Option<Money>,
        /// `None` on pages with no image.
        pub featured_image: Option<ProductImage>,
        pub images: Vec<ProductImage>,

        // Commented out: `images` already carries the gallery, mirroring `media` one for one, and
        // video entries are not of interest right now.
        // pub media: Vec<ProductMedia>,
        pub options: Vec<ProductOption>,
        pub variants: Vec<ProductVariant>,
        pub selling_plan_groups: Vec<SellingPlanGroup>,
        // Commented out: an untyped, per-namespace CMS/storefront-configuration blob - see
        // AGENTS.md and `raw::flight::ProductMetafields`. Individual facts worth having (specs,
        // navigation, ...) should be lifted out by name rather than carrying the whole thing.
        // pub metafields: ProductMetafields,
    }

    impl ProductRecord {
        pub fn from_raw(raw: raw::flight::Product) -> Result<Self, NormalizeError> {
            let published_at = published_at(&raw.published_at)?;

            Ok(ProductRecord {
                id: raw.id,
                handle: raw.handle,
                title: raw.title,
                vendor: raw.vendor,
                product_type: raw.product_type,
                description: raw.description,
                // description_html: raw.description_html,
                available_for_sale: raw.available_for_sale,
                created_at: raw.created_at,
                updated_at: raw.updated_at,
                published_at,
                // tags: raw.tags,
                seo: ProductSeo::from_raw(raw.seo),
                price: money(raw.price),
                compare_at_price: compare_at_price(raw.compare_at_price),
                featured_image: raw.featured_image.map(ProductImage::from_raw),
                images: raw.images.into_iter().map(ProductImage::from_raw).collect(),
                // media: raw.media.into_iter().map(ProductMedia::from_raw).collect(),
                options: raw
                    .options
                    .into_iter()
                    .map(ProductOption::from_raw)
                    .collect(),
                variants: raw
                    .variants
                    .into_iter()
                    .map(ProductVariant::from_raw)
                    .collect(),
                selling_plan_groups: raw
                    .selling_plan_groups
                    .into_iter()
                    .map(SellingPlanGroup::from_raw)
                    .collect(),
                // metafields: ProductMetafields::from_raw(raw.metafields),
            })
        }
    }

    fn published_at(raw: &str) -> Result<DateTime<Utc>, NormalizeError> {
        let parsed = DateTime::parse_from_rfc3339(raw).map_err(|source| {
            NormalizeError::PublishedAtNotAnInstant {
                value: raw.to_string(),
                source,
            }
        })?;

        Ok(parsed.with_timezone(&Utc))
    }

    fn money(raw: raw::flight::Price) -> Money {
        let amount = raw.amount.parse::<f64>().unwrap();

        Money::from_raw(amount, &raw.currency_code).unwrap()
    }

    // `0.0` is the storefront's sentinel for "not marked down", rather than a real zero price. The
    // product renders it as that zero; a variant omits the key entirely, so both spellings of the
    // same absence go through here.
    fn compare_at_price(raw: raw::flight::Price) -> Option<Money> {
        let amount = raw.amount.parse::<f64>().unwrap();

        (amount != 0.0).then(|| Money::from_raw(amount, &raw.currency_code).unwrap())
    }

    // `0.0` is what an unweighed variant renders rather than a weightless one.
    fn weight(raw: f64) -> Option<f64> {
        (raw != 0.0).then_some(raw)
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ProductImage {
        pub url: String,
        // Commented out: not needed for now.
        // pub alt_text: Option<String>,
        // pub width: u32,
        // pub height: u32,
    }

    impl ProductImage {
        fn from_raw(raw: raw::flight::Image) -> ProductImage {
            ProductImage {
                url: raw.url,
                // alt_text: raw.alt_text,
                // width: raw.width,
                // height: raw.height,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ProductSeo {
        pub title: Option<String>,
        pub description: Option<String>,
    }

    impl ProductSeo {
        fn from_raw(raw: raw::flight::Seo) -> ProductSeo {
            ProductSeo {
                title: raw.title,
                description: raw.description,
            }
        }
    }

    /* Media normalization is disabled: `images` already carries the gallery, and video entries
    are not of interest right now.
    #[derive(Debug, Clone, PartialEq, Serialize)]
    #[serde(tag = "media_content_type")]
    pub enum ProductMedia {
        Image { id: String, image: MediaImage },
        Video { id: String, sources: Vec<MediaSource> },
    }

    impl ProductMedia {
        fn from_raw(raw: raw::flight::Media) -> ProductMedia {
            match raw {
                raw::flight::Media::Image { id, image } => ProductMedia::Image {
                    id,
                    image: MediaImage::from_raw(image),
                },
                raw::flight::Media::Video { id, sources } => ProductMedia::Video {
                    id,
                    sources: sources.into_iter().map(MediaSource::from_raw).collect(),
                },
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct MediaImage {
        pub url: String,
        pub alt_text: Option<String>,
    }

    impl MediaImage {
        fn from_raw(raw: raw::flight::MediaImage) -> MediaImage {
            MediaImage {
                url: raw.url,
                alt_text: raw.alt_text,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct MediaSource {
        pub url: String,
        pub format: String,
        pub mime_type: String,
        pub width: u32,
        pub height: u32,
    }

    impl MediaSource {
        fn from_raw(raw: raw::flight::MediaSource) -> MediaSource {
            MediaSource {
                url: raw.url,
                format: raw.format,
                mime_type: raw.mime_type,
                width: raw.width,
                height: raw.height,
            }
        }
    }
    */

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ProductOption {
        pub id: String,
        pub name: String,
        pub values: Vec<String>,
    }

    impl ProductOption {
        fn from_raw(raw: raw::flight::ProductOption) -> ProductOption {
            ProductOption {
                id: raw.id,
                name: raw.name,
                values: raw.values,
            }
        }
    }

    /* Product metafields normalization is disabled: an untyped, per-namespace CMS/storefront
    blob - see AGENTS.md and `raw::flight::ProductMetafields`.
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct ProductMetafields {
        pub global: Option<Value>,
        pub component: Option<Value>,
        pub original_payload: Option<Value>,
        pub seo: Option<Value>,
        pub product: Option<Value>,
    }

    impl ProductMetafields {
        fn from_raw(raw: raw::flight::ProductMetafields) -> ProductMetafields {
            ProductMetafields {
                global: raw.global,
                component: raw.component,
                original_payload: raw.original_payload,
                seo: raw.seo,
                product: raw.product,
            }
        }
    }
    */

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ProductVariant {
        pub id: String,
        pub title: String,
        pub sku: String,
        pub available_for_sale: bool,
        pub requires_shipping: bool,
        pub quantity_available: i64,
        pub price: Money,
        pub compare_at_price: Option<Money>,
        /// Probably pounds: the payload states no unit, but the same class of item reads heavier
        /// here than on the kilogram storefronts - a 3 ft Lightning cable reads `0.0860`. `None`
        /// where the variant renders the `0.0` an unweighed one does.
        pub weight: Option<f64>,
        /// `None` where the product has no image.
        pub image: Option<ProductImage>,
        pub selected_options: Vec<SelectedOption>,
        pub selling_plan_allocations: Vec<SellingPlanAllocation>,
        // Commented out: an untyped, per-namespace CMS/storefront-configuration blob - see
        // AGENTS.md and `raw::flight::VariantMetafields`.
        // pub metafields: VariantMetafields,
    }

    impl ProductVariant {
        pub fn from_raw(raw: raw::flight::Variant) -> ProductVariant {
            ProductVariant {
                id: raw.id,
                title: raw.title,
                sku: raw.sku,
                available_for_sale: raw.available_for_sale,
                requires_shipping: raw.requires_shipping,
                quantity_available: raw.quantity_available,
                price: money(raw.price),
                compare_at_price: raw.compare_at_price.and_then(compare_at_price),
                weight: weight(raw.weight),
                image: raw.image.map(ProductImage::from_raw),
                selected_options: raw
                    .selected_options
                    .into_iter()
                    .map(SelectedOption::from_raw)
                    .collect(),
                selling_plan_allocations: raw
                    .selling_plan_allocations
                    .into_iter()
                    .map(SellingPlanAllocation::from_raw)
                    .collect(),
                // metafields: VariantMetafields::from_raw(raw.metafields),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SelectedOption {
        pub name: String,
        pub value: String,
    }

    impl SelectedOption {
        fn from_raw(raw: raw::flight::SelectedOption) -> SelectedOption {
            SelectedOption {
                name: raw.name,
                value: raw.value,
            }
        }
    }

    /* Variant metafields normalization is disabled: an untyped, per-namespace CMS/storefront
    blob - see AGENTS.md and `raw::flight::VariantMetafields`.
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct VariantMetafields {
        pub global: Option<Value>,
        pub discount_code_app: Option<Value>,
        pub component: Option<Value>,
        pub original_payload: Option<Value>,
        pub filter: Option<Value>,
        pub discounts_function: Option<Value>,
        pub variant: Option<Value>,
    }

    impl VariantMetafields {
        fn from_raw(raw: raw::flight::VariantMetafields) -> VariantMetafields {
            VariantMetafields {
                global: raw.global,
                discount_code_app: raw.discount_code_app,
                component: raw.component,
                original_payload: raw.original_payload,
                filter: raw.filter,
                discounts_function: raw.discounts_function,
                variant: raw.variant,
            }
        }
    }
    */

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SellingPlanAllocation {
        pub selling_plan: AllocatedSellingPlan,
    }

    impl SellingPlanAllocation {
        // The `priceAdjustments` each allocation carries are empty everywhere in the corpus - see
        // [`raw::flight::EmptyAdjustment`] - so they carry no information forward here.
        fn from_raw(raw: raw::flight::SellingPlanAllocation) -> SellingPlanAllocation {
            SellingPlanAllocation {
                selling_plan: AllocatedSellingPlan::from_raw(raw.selling_plan),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AllocatedSellingPlan {
        pub id: String,
        pub name: String,
        pub options: Vec<SellingPlanOption>,
    }

    impl AllocatedSellingPlan {
        fn from_raw(raw: raw::flight::AllocatedSellingPlan) -> AllocatedSellingPlan {
            AllocatedSellingPlan {
                id: raw.id,
                name: raw.name,
                options: raw
                    .options
                    .into_iter()
                    .map(SellingPlanOption::from_raw)
                    .collect(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SellingPlanGroup {
        pub name: String,
        pub selling_plans: Vec<SellingPlan>,
    }

    impl SellingPlanGroup {
        fn from_raw(raw: raw::flight::SellingPlanGroup) -> SellingPlanGroup {
            SellingPlanGroup {
                name: raw.name,
                selling_plans: raw
                    .selling_plans
                    .into_iter()
                    .map(SellingPlan::from_raw)
                    .collect(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SellingPlan {
        pub id: String,
        pub name: String,
        pub options: Vec<SellingPlanOption>,
    }

    impl SellingPlan {
        fn from_raw(raw: raw::flight::SellingPlan) -> SellingPlan {
            SellingPlan {
                id: raw.id,
                name: raw.name,
                options: raw
                    .options
                    .into_iter()
                    .map(SellingPlanOption::from_raw)
                    .collect(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SellingPlanOption {
        pub name: String,
        pub value: String,
    }

    impl SellingPlanOption {
        fn from_raw(raw: raw::flight::SellingPlanOption) -> SellingPlanOption {
            SellingPlanOption {
                name: raw.name,
                value: raw.value,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizeError, ldjson};
    use html_extractor::serde_json;

    #[test]
    fn a_document_without_a_product_island_fails_to_normalize() {
        let scripts = scripts(
            r#"[{"data":[{"@type":"Corporation","@context":"c","logo":"l","name":"Anker",
            "sameAs":[],"url":"u"}]}]"#,
        );

        let error = ldjson::LdJson::from_raw(scripts)
            .expect_err("a document with no Product island is not a product page");

        assert_eq!(error, NormalizeError::MissingProductIsland);
        assert_eq!(
            error.to_string(),
            "no `Product` ld+json island on the document"
        );
    }

    #[test]
    fn the_first_product_island_wins_and_the_others_are_ignored() {
        let scripts = scripts(
            r#"[{"data":[
            {"@type":"Corporation","@context":"c","logo":"l","name":"Anker","sameAs":[],"url":"u"},
            {"@type":"Product","@context":"c","brand":{"@context":"c","@type":"Brand","name":"Anker"},
             "description":"first","image":"i","name":"n","url":"u",
             "offers":{"@context":"c","@type":"Offer","availability":"https://schema.org/InStock",
             "price":"99.99","priceCurrency":"CAD"}},
            {"@type":"Product","@context":"c","brand":{"@context":"c","@type":"Brand","name":"Anker"},
             "description":"second","image":"i","name":"n","url":"u",
             "offers":{"@context":"c","@type":"Offer","availability":"https://schema.org/InStock",
             "price":"1.00","priceCurrency":"CAD"}}]}]"#,
        );

        let ld_json = ldjson::LdJson::from_raw(scripts).expect("the island reads");

        assert_eq!(ld_json.product.description, "first");
    }

    fn scripts(json: &str) -> Vec<super::raw::ldjsonitem::LdJsonScript> {
        serde_json::from_str(json).expect("the fixture is a list of ld+json scripts")
    }
}
