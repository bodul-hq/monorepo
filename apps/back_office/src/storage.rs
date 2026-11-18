use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::{BigInt, Jsonb, Text};
use diesel::{OptionalExtension, PgConnection, QueryableByName, RunQueryDsl, sql_query};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use poem::{Error, Result, http::StatusCode, session::SessionStorage};
use serde_json::{Value, from_value, to_value};
use std::{collections::BTreeMap, time::Duration};
use tokio::task::spawn_blocking;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
#[derive(Clone)]
pub struct Storage(Pool<ConnectionManager<PgConnection>>);
fn unavailable() -> Error {
    Error::from_status(StatusCode::SERVICE_UNAVAILABLE)
}
impl Storage {
    pub fn connect(url: &str) -> Result<Self> {
        let pool = Pool::builder()
            .connection_timeout(Duration::from_secs(5))
            .build(ConnectionManager::<PgConnection>::new(url))
            .map_err(|_| unavailable())?;
        pool.get()
            .map_err(|_| unavailable())?
            .run_pending_migrations(MIGRATIONS)
            .map_err(|_| unavailable())?;
        Ok(Self(pool))
    }
    pub async fn cleanup(&self) -> Result<()> {
        let pool = self.0.clone();
        spawn_blocking(move || {
            sql_query("DELETE FROM sessions WHERE expires_at <= now()")
                .execute(&mut pool.get().map_err(|_| unavailable())?)
                .map_err(|_| unavailable())?;
            Ok(())
        })
        .await
        .map_err(|_| unavailable())?
    }
}
#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Jsonb)]
    data: Value,
}
impl SessionStorage for Storage {
    async fn load_session(&self, id: &str) -> Result<Option<BTreeMap<String, Value>>> {
        let pool = self.0.clone();
        let id = id.to_owned();
        spawn_blocking(move || {
            let row = sql_query("SELECT data FROM sessions WHERE id = $1 AND expires_at > now()")
                .bind::<Text, _>(id)
                .get_result::<Row>(&mut pool.get().map_err(|_| unavailable())?)
                .optional()
                .map_err(|_| unavailable())?;
            row.map(|r| from_value(r.data).map_err(|_| unavailable()))
                .transpose()
        })
        .await
        .map_err(|_| unavailable())?
    }
    async fn update_session(
        &self,
        id: &str,
        entries: &BTreeMap<String, Value>,
        expires: Option<Duration>,
    ) -> Result<()> {
        let pool = self.0.clone();
        let id = id.to_owned();
        let data = to_value(entries).map_err(|_| unavailable())?;
        let seconds = expires.unwrap_or(Duration::from_secs(28800)).as_secs() as i64;
        spawn_blocking(move || {
            sql_query("INSERT INTO sessions(id,data,expires_at) VALUES ($1,$2,now()+$3 * interval '1 second') ON CONFLICT(id) DO UPDATE SET data=EXCLUDED.data")
                .bind::<Text,_>(id).bind::<Jsonb,_>(data).bind::<BigInt,_>(seconds)
                .execute(&mut pool.get().map_err(|_| unavailable())?).map_err(|_| unavailable())?;
            Ok(())
        }).await.map_err(|_| unavailable())?
    }
    async fn remove_session(&self, id: &str) -> Result<()> {
        let pool = self.0.clone();
        let id = id.to_owned();
        spawn_blocking(move || {
            sql_query("DELETE FROM sessions WHERE id=$1")
                .bind::<Text, _>(id)
                .execute(&mut pool.get().map_err(|_| unavailable())?)
                .map_err(|_| unavailable())?;
            Ok(())
        })
        .await
        .map_err(|_| unavailable())?
    }
}

#[cfg(test)]
mod postgres_tests {
    use super::*;
    use diesel::Connection;
    use rand::{RngExt, rng};
    use std::env::var;

    #[tokio::test]
    #[ignore = "requires DATABASE_URL_TEST"]
    async fn persistence_expiry_and_revocation() {
        let url = var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST");
        let schema = format!("back_office_test_{}", rng().random::<u64>());
        let mut connection = PgConnection::establish(&url).unwrap();
        sql_query(format!("CREATE SCHEMA {schema}"))
            .execute(&mut connection)
            .unwrap();
        let separator = if url.contains('?') { '&' } else { '?' };
        let scoped = format!("{url}{separator}options=-csearch_path%3D{schema}");
        let first = Storage::connect(&scoped).unwrap();
        let mut entries = BTreeMap::new();
        entries.insert("user".into(), Value::String("bodul".into()));
        first
            .update_session("persistent", &entries, Some(Duration::from_secs(28800)))
            .await
            .unwrap();
        drop(first);
        let second = Storage::connect(&scoped).unwrap();
        assert_eq!(
            second.load_session("persistent").await.unwrap(),
            Some(entries.clone())
        );
        second.remove_session("persistent").await.unwrap();
        assert!(second.load_session("persistent").await.unwrap().is_none());
        second
            .update_session("expired", &entries, Some(Duration::ZERO))
            .await
            .unwrap();
        assert!(second.load_session("expired").await.unwrap().is_none());
        second.cleanup().await.unwrap();
        drop(second);
        sql_query(format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&mut connection)
            .unwrap();
    }
}
