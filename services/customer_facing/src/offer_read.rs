//! One bounded read path for REST and GraphQL, independent of queue execution.

use std::sync::Arc;

use async_graphql::{ComplexObject, ID, SimpleObject};
use chrono::{DateTime, SecondsFormat, Utc};
use diesel::{
    Connection, RunQueryDsl,
    connection::SimpleConnection,
    sql_types::{BigInt, Text, Timestamptz, Uuid as SqlUuid},
};
use kernel::io::DbPool;
use poem_openapi::{
    ApiResponse, Object, OpenApi,
    param::Query,
    payload::{Json, PlainText},
};
use uuid::Uuid;

mod facts;
pub use facts::{OfferAvailability, OfferContent, OfferItemCondition};

#[derive(Clone, Object, SimpleObject, serde::Serialize)]
pub struct OfferPrice {
    /// Exact signed i64 minor units, encoded as a decimal string.
    pub amount: String,
    pub currency: String,
}

#[derive(Clone, Object, SimpleObject, serde::Serialize)]
#[graphql(complex)]
pub struct Offer {
    pub was_price: Option<OfferPrice>,
    pub availability: Option<OfferAvailability>,
    pub item_condition: Option<OfferItemCondition>,
    pub content: Vec<OfferContent>,
    pub gtin: Option<String>,
    pub sku: Option<String>,
    pub external_identifiers: ExternalIdentifiers,
    pub localization: OfferLocalization,
    /// Latest known successful download, in UTC RFC3339 format.
    pub scraped_at: Option<String>,
    #[graphql(skip)]
    pub id: String,
    pub retailer_code: String,
    pub title: String,
    pub price: OfferPrice,
    pub url: String,
    pub images: Vec<String>,
    /// Original creation instant in UTC RFC3339 format.
    pub created_at: String,
}

#[ComplexObject]
impl Offer {
    async fn id(&self) -> ID {
        ID(self.id.clone())
    }
}

#[derive(Object, SimpleObject, serde::Serialize)]
pub struct OfferPage {
    pub items: Vec<Offer>,
    pub limit: i32,
    pub offset: i32,
    pub has_more: bool,
}

pub trait Repository: Send + Sync {
    fn load(&self, limit: i64, offset: i64) -> Result<Vec<Offer>, ()>;
    fn search(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<Offer>, ()>;
}

struct PostgresRepository(DbPool);

#[derive(diesel::QueryableByName)]
struct Record {
    #[diesel(sql_type = diesel::sql_types::Nullable<BigInt>)]
    was_price_amount: Option<i64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    was_price_currency: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
    availability: Option<serde_json::Value>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
    item_condition: Option<serde_json::Value>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    content: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    gtin: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    sku: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    external_identifiers: crate::identifier::Identifier,

    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
    target_market: Option<serde_json::Value>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    language: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Timestamptz>)]
    scraped_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    retailer_code: String,
    #[diesel(sql_type = Text)]
    title: String,
    #[diesel(sql_type = BigInt)]
    price_amount: i64,
    #[diesel(sql_type = Text)]
    currency: String,
    #[diesel(sql_type = Text)]
    url: String,
    #[diesel(sql_type = diesel::sql_types::Array<Text>)]
    images: Vec<String>,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

impl Repository for PostgresRepository {
    fn load(&self, limit: i64, offset: i64) -> Result<Vec<Offer>, ()> {
        let mut connection = self.0.get().map_err(|_| ())?;
        let records = diesel::sql_query(
            "SELECT page.*, ARRAY(SELECT image_url FROM offer_images WHERE offer_id = page.id ORDER BY position) AS images
             FROM (SELECT offers.*, offer_selected_content(content)->>'title' AS title, offer_selected_content(content)->>'description' AS description, offer_selected_content(content)->>'url' AS url, content->>'selected_language' AS language
             FROM offers ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2) page
             ORDER BY created_at DESC, id DESC",
        )
        .bind::<BigInt, _>(limit)
        .bind::<BigInt, _>(offset)
        .load::<Record>(&mut connection)
        .map_err(|_| ())?;
        map_records(records)
    }

    fn search(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<Offer>, ()> {
        let mut connection = self.0.get().map_err(|_| ())?;
        let records = connection.transaction::<_, diesel::result::Error, _>(|connection| {
            connection.batch_execute(
                "SET LOCAL pg_trgm.word_similarity_threshold = 0.5; SET LOCAL statement_timeout = '3s'",
            )?;
            diesel::sql_query(
                "SELECT page.*, ARRAY(SELECT image_url FROM offer_images WHERE offer_id = page.id ORDER BY position) AS images
                 FROM (SELECT offers.*, offer_selected_content(content)->>'title' AS title,
                    offer_selected_content(content)->>'description' AS description,
                    offer_selected_content(content)->>'url' AS url, content->>'selected_language' AS language,
                    5 * customer_facing_extensions.word_similarity(lower($1), lower(offer_selected_content(content)->>'title'))
                    + COALESCE(customer_facing_extensions.word_similarity(lower($1), lower(offer_selected_content(content)->>'description')),0) AS score
                 FROM offers WHERE lower($1) OPERATOR(customer_facing_extensions.<%) lower(offer_selected_content(content)->>'title')
                    OR lower($1) OPERATOR(customer_facing_extensions.<%) lower(offer_selected_content(content)->>'description')
                 ORDER BY score DESC, created_at DESC, id DESC LIMIT $2 OFFSET $3) page
                 ORDER BY score DESC, created_at DESC, id DESC",
            )
            .bind::<Text, _>(query)
            .bind::<BigInt, _>(limit)
            .bind::<BigInt, _>(offset)
            .load::<Record>(connection)
        }).map_err(|_| ())?;
        map_records(records)
    }
}

fn map_records(records: Vec<Record>) -> Result<Vec<Offer>, ()> {
    records
        .into_iter()
        .map(|record| {
            let content = facts::content(
                record.content,
                &record.language,
                &record.title,
                &record.description,
            )?;
            Ok(Offer {
                was_price: facts::was_price(
                    record.was_price_amount,
                    record.was_price_currency,
                    record.price_amount,
                    &record.currency,
                )?,
                availability: record
                    .availability
                    .map(OfferAvailability::from_record)
                    .transpose()?,
                item_condition: record
                    .item_condition
                    .map(OfferItemCondition::from_record)
                    .transpose()?,
                content,
                gtin: record.gtin,
                sku: record.sku,
                external_identifiers: ExternalIdentifiers {
                    shopify_product_gid: record
                        .external_identifiers
                        .0
                        .shopify_product_gid
                        .map(|value| value.value),
                    shopify_product_variation_gid: record
                        .external_identifiers
                        .0
                        .shopify_product_variation_gid
                        .map(|value| value.value),
                    shopify_handle: record
                        .external_identifiers
                        .0
                        .shopify_handle
                        .map(|value| value.value),
                },
                localization: OfferLocalization::from_record(
                    record.target_market,
                    record.language,
                )?,
                scraped_at: record
                    .scraped_at
                    .map(|time| time.to_rfc3339_opts(SecondsFormat::AutoSi, true)),
                id: record.id.to_string(),
                retailer_code: record.retailer_code,
                title: record.title,
                price: OfferPrice {
                    amount: record.price_amount.to_string(),
                    currency: record.currency,
                },
                url: record.url,
                images: record.images,
                created_at: record
                    .created_at
                    .to_rfc3339_opts(SecondsFormat::AutoSi, true),
            })
        })
        .collect::<Result<Vec<_>, ()>>()
}

#[derive(Clone)]
pub struct Offers(Arc<dyn Repository>);

#[derive(Debug)]
pub enum ReadError {
    Invalid(&'static str),
    Unavailable,
}

impl ReadError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Invalid(message) => message,
            Self::Unavailable => "offers could not be retrieved",
        }
    }
}

impl Offers {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self(repository)
    }

    pub fn postgres(pool: DbPool) -> Self {
        Self::new(Arc::new(PostgresRepository(pool)))
    }

    pub async fn list(&self, limit: i32, offset: i32) -> Result<OfferPage, ReadError> {
        self.read(None, limit, offset).await
    }

    pub async fn search(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<OfferPage, ReadError> {
        self.read(
            Some(normalize_query(query).map_err(ReadError::Invalid)?),
            limit,
            offset,
        )
        .await
    }

    async fn read(
        &self,
        query: Option<String>,
        limit: i32,
        offset: i32,
    ) -> Result<OfferPage, ReadError> {
        if !(1..=100).contains(&limit) {
            return Err(ReadError::Invalid("limit must be between 1 and 100"));
        }
        if offset < 0 {
            return Err(ReadError::Invalid(
                "offset must be between 0 and 2147483647",
            ));
        }
        let repository = self.0.clone();
        let mut items = tokio::task::spawn_blocking(move || match query {
            Some(query) => repository.search(&query, i64::from(limit) + 1, i64::from(offset)),
            None => repository.load(i64::from(limit) + 1, i64::from(offset)),
        })
        .await
        .map_err(|_| ReadError::Unavailable)?
        .map_err(|_| ReadError::Unavailable)?;
        let has_more = items.len() > limit as usize;
        items.truncate(limit as usize);
        Ok(OfferPage {
            items,
            limit,
            offset,
            has_more,
        })
    }
}

#[derive(ApiResponse)]
pub enum PageResponse {
    /// A page in browse or relevance order. Concurrent updates can shift later pages.
    #[oai(status = 200)]
    Ok(Json<OfferPage>),
    #[oai(status = 400)]
    Invalid(PlainText<&'static str>),
    #[oai(status = 500)]
    Unavailable(PlainText<&'static str>),
}

pub struct OffersApi(pub Offers);

#[OpenApi]
impl OffersApi {
    /// Search stored titles by relevance, then creation time and ID. No queues run.
    #[oai(path = "/offers/search", method = "get")]
    async fn search(
        &self,
        q: Query<String>,
        #[oai(
            default = "default_limit",
            validator(minimum(value = "1"), maximum(value = "100"))
        )]
        limit: Query<i32>,
        #[oai(
            default,
            validator(minimum(value = "0"), maximum(value = "2147483647"))
        )]
        offset: Query<i32>,
    ) -> PageResponse {
        match self.0.search(&q.0, limit.0, offset.0).await {
            Ok(page) => PageResponse::Ok(Json(page)),
            Err(error @ ReadError::Invalid(_)) => PageResponse::Invalid(PlainText(error.message())),
            Err(error) => PageResponse::Unavailable(PlainText(error.message())),
        }
    }

    /// Browse projected offers without executing any queues.
    #[oai(path = "/offers", method = "get")]
    async fn offers(
        &self,
        #[oai(
            default = "default_limit",
            validator(minimum(value = "1"), maximum(value = "100"))
        )]
        limit: Query<i32>,
        #[oai(
            default,
            validator(minimum(value = "0"), maximum(value = "2147483647"))
        )]
        offset: Query<i32>,
    ) -> PageResponse {
        match self.0.list(limit.0, offset.0).await {
            Ok(page) => PageResponse::Ok(Json(page)),
            Err(error @ ReadError::Invalid(_)) => PageResponse::Invalid(PlainText(error.message())),
            Err(error) => PageResponse::Unavailable(PlainText(error.message())),
        }
    }
}

fn default_limit() -> i32 {
    30
}

#[cfg(test)]
pub(crate) mod tests;

#[derive(Clone, Copy, poem_openapi::Enum, async_graphql::Enum, serde::Serialize, PartialEq, Eq)]
pub enum MarketKind {
    #[oai(rename = "country")]
    #[serde(rename = "country")]
    Country,
    #[oai(rename = "region")]
    #[serde(rename = "region")]
    Region,
}
#[derive(Clone, Object, SimpleObject, serde::Serialize)]
pub struct OfferMarket {
    pub kind: MarketKind,
    pub code: Option<String>,
    pub label: Option<String>,
}
#[derive(Clone, Default, Object, SimpleObject, serde::Serialize)]
pub struct OfferLocalization {
    pub target_market: Option<OfferMarket>,
    pub language: Option<String>,
}
impl OfferLocalization {
    fn from_record(
        market: Option<serde_json::Value>,
        language: Option<String>,
    ) -> Result<Self, ()> {
        use retailer_sauce::localization::{Localization, TargetMarket};
        let target_market = market
            .map(serde_json::from_value)
            .transpose()
            .map_err(|_| ())?;
        let localization = Localization {
            target_market,
            language,
        };
        localization.guard().map_err(|_| ())?;
        let target_market = localization.target_market.map(|market| match market {
            TargetMarket::Country { code } => OfferMarket {
                kind: MarketKind::Country,
                code: Some(code.as_str().into()),
                label: None,
            },
            TargetMarket::Region { label } => OfferMarket {
                kind: MarketKind::Region,
                code: None,
                label: Some(label),
            },
        });
        Ok(Self {
            target_market,
            language: localization.language,
        })
    }
}

#[derive(Clone, Default, Object, SimpleObject, serde::Serialize)]
pub struct ExternalIdentifiers {
    pub shopify_product_gid: Option<String>,
    pub shopify_product_variation_gid: Option<String>,
    pub shopify_handle: Option<String>,
}

/// Keep these boundary rules aligned with the customers app.
pub fn normalize_query(query: &str) -> Result<String, &'static str> {
    if query.chars().any(|c| c.is_control() && !c.is_whitespace()) {
        return Err("search text must not contain control characters");
    }
    let query = query.split_whitespace().collect::<Vec<_>>().join(" ");
    if !(1..=200).contains(&query.chars().count()) || !query.chars().any(char::is_alphanumeric) {
        return Err("enter 1–200 characters including a letter or number");
    }
    Ok(query)
}
