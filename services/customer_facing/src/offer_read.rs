//! One bounded read path for REST and GraphQL, independent of queue execution.

use std::sync::Arc;

use async_graphql::{ComplexObject, ID, SimpleObject};
use chrono::{DateTime, SecondsFormat, Utc};
use diesel::{
    RunQueryDsl,
    sql_types::{BigInt, Text, Timestamptz, Uuid as SqlUuid},
};
use kernel::io::DbPool;
use poem_openapi::{
    ApiResponse, Object, OpenApi,
    param::Query,
    payload::{Json, PlainText},
};
use uuid::Uuid;

#[derive(Clone, Object, SimpleObject, serde::Serialize)]
pub struct OfferPrice {
    /// Exact signed i64 minor units, encoded as a decimal string.
    pub amount: String,
    pub currency: String,
}

#[derive(Clone, Object, SimpleObject, serde::Serialize)]
#[graphql(complex)]
pub struct Offer {
    #[graphql(skip)]
    pub id: String,
    pub retailer_code: String,
    pub title: String,
    pub price: OfferPrice,
    pub url: String,
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
}

struct PostgresRepository(DbPool);

#[derive(diesel::QueryableByName)]
struct Record {
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
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

impl Repository for PostgresRepository {
    fn load(&self, limit: i64, offset: i64) -> Result<Vec<Offer>, ()> {
        let mut connection = self.0.get().map_err(|_| ())?;
        let records = diesel::sql_query(
            "SELECT id, retailer_code, title, price_amount, currency, url, created_at
             FROM offers ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2",
        )
        .bind::<BigInt, _>(limit)
        .bind::<BigInt, _>(offset)
        .load::<Record>(&mut connection)
        .map_err(|_| ())?;
        Ok(records
            .into_iter()
            .map(|record| Offer {
                id: record.id.to_string(),
                retailer_code: record.retailer_code,
                title: record.title,
                price: OfferPrice {
                    amount: record.price_amount.to_string(),
                    currency: record.currency,
                },
                url: record.url,
                created_at: record
                    .created_at
                    .to_rfc3339_opts(SecondsFormat::AutoSi, true),
            })
            .collect())
    }
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
        if !(1..=100).contains(&limit) {
            return Err(ReadError::Invalid("limit must be between 1 and 100"));
        }
        if offset < 0 {
            return Err(ReadError::Invalid(
                "offset must be between 0 and 2147483647",
            ));
        }
        let repository = self.0.clone();
        let mut items = tokio::task::spawn_blocking(move || {
            repository.load(i64::from(limit) + 1, i64::from(offset))
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
    /// A page of offers, newest first. Concurrent inserts can shift later pages.
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
