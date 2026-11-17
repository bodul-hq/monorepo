use crate::app_with_reporting_token;
use crate::offers::command::CREATE_FROM_OFFER_DETAILS_COMMAND;
use crate::offers::command::ImportOfferDetailsCommand;
use crate::reporting::DEFAULT_REPORTING_TOKEN;
use crate::reporting::ReportingApi;
use crate::start_mulac;
use crate::tests::assert_batch_response;
use crate::tests::assert_error_response;
use chrono::DateTime;
use chrono::Utc;
use inbox::prelude::testing::invalid_submissions;
use money::Currency;
use money::Money;
use poem::Route;
use poem_openapi::OpenApiService;
use serde_json::from_str;
use serde_json::to_string as serde_json_to_string3;
use shared::retailer::RetailerCode as SharedRetailerRetailerCode;
use std::collections::HashSet;
use std::env::var;
use url::Url;

fn reporting_now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-12-31T23:59:59Z")
        .unwrap()
        .with_timezone(&Utc)
}

use diesel::{Connection, PgConnection, connection::SimpleConnection, r2d2::ConnectionManager};
use diesel_migrations::MigrationHarness;
use poem::{http::StatusCode, test::TestClient};
use serde_json::{Value, json};
use testimony::prelude::EventStore;
use testimony_infra::prelude::DbPool;
use uuid::Uuid;

mod content;
mod facts;

use crate::{
    bootstrap::MIGRATIONS,
    infra::repository::OfferEventStore,
    offers::{event::RetailerOfferEvent, retailer_offer_id::RetailerOfferId},
};

pub(crate) struct TestSchema {
    connection: PgConnection,
    name: String,
}

impl Drop for TestSchema {
    fn drop(&mut self) {
        self.connection
            .batch_execute(&format!("DROP SCHEMA {} CASCADE", self.name))
            .expect("drop isolated test schema");
    }
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn tmp_import_stores_command_in_inbox_and_reports_storage_failure() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app_with_reporting_token(state, "test".into()));
    let input = json!({
        "retailer_code": "ankerau", "title": "USB charger", "price_amount": 1999,
        "currency": "AUD", "url": "https://example.com/charger?color=red"
    });
    let response = client
        .post("/api/tmp-import")
        .body_json(&input)
        .send()
        .await;
    response.assert_status(StatusCode::ACCEPTED);
    let body: Value = response.json().await.value().deserialize();
    let message_id = Uuid::parse_str(body["message_id"].as_str().unwrap()).unwrap();

    // The import is only stored: no command, event or offer exists yet.
    let stored = inbox_test_snapshot(&pool, "SELECT to_jsonb(i) AS data FROM inbox_entries i");
    assert_eq!(stored["id"], message_id.to_string());
    assert_eq!(
        from_str::<Value>(stored["payload"].as_str().unwrap()).unwrap(),
        json!({
            "retailer_code":"ankerau", "title":"USB charger",
            "price":{"amount":1999,"currency":"AUD"},
            "url":"https://example.com/charger?color=red", "images":[]
        })
    );
    assert_eq!(
        stored["meta"],
        json!({
            "message_id":message_id,"correlation_id":null,"source":"offers-api",
            "routing_key":CREATE_FROM_OFFER_DETAILS_COMMAND
        })
    );
    let counts = || {
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_build_object('commands', (SELECT count(*) FROM command_entries), 'events', (SELECT count(*) FROM event_store)) AS data",
        )
    };
    assert_eq!(counts(), json!({"commands":0,"events":0}));

    // Running the inbox queues the command; running commands saves the offer.
    run_queue(&client, "inbox", 10, 1).await;
    assert_eq!(counts(), json!({"commands":1,"events":0}));
    run_queue(&client, "commands", 10, 1).await;
    let stored_id = inbox_test_snapshot(&pool, "SELECT to_jsonb(agg_id) AS data FROM event_store");
    let id = stored_id.as_str().unwrap();
    let events = OfferEventStore::new(pool.clone())
        .load(RetailerOfferId::new(id))
        .unwrap();
    assert_eq!(events.len(), 1);
    let RetailerOfferEvent::OfferCreated(event) = events[0].event() else {
        panic!("expected creation event");
    };
    assert_eq!(event.title, "USB charger");
    assert_eq!(event.url, input["url"].as_str().unwrap());
    assert_eq!(event.price, Money::new(1999, Currency::AUD));
    assert_eq!(
        event.retailer_code,
        SharedRetailerRetailerCode::from_str("ankerau").unwrap()
    );

    // Inbox storage failures hide database diagnostics.
    pool.get()
        .unwrap()
        .batch_execute("DROP TABLE inbox_entries")
        .unwrap();
    let response = client
        .post("/api/tmp-import")
        .body_json(&input)
        .send()
        .await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    assert_error_response(response).await;
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn reporting_returns_schema_statistics_and_exact_queue_counts() {
    let database_url = var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).expect("connect to test database"),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut url = Url::parse(&database_url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(url.to_string()))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    let api = ReportingApi::with_clock(pool.clone(), DEFAULT_REPORTING_TOKEN.into(), reporting_now);
    let client =
        TestClient::new(Route::new().nest("/api", OpenApiService::new(api, "Reporting", "test")));
    let token = format!("Bearer {}", DEFAULT_REPORTING_TOKEN);
    let response = client
        .get("/api/reporting/database-tables")
        .header("authorization", &token)
        .send()
        .await;
    response.assert_status_is_ok();
    assert_eq!(response.0.headers()["cache-control"], "no-store");
    let report: Value = response.json().await.value().deserialize();
    assert_eq!(report["schema_name"], schema.name);
    assert_eq!(report["row_count_kind"], "estimated");
    let tables = report["tables"].as_array().unwrap();
    assert!(
        tables
            .iter()
            .any(|table| table["table_name"] == "event_store")
    );
    let names: Vec<_> = tables
        .iter()
        .map(|table| table["table_name"].as_str().unwrap())
        .collect();
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    for table in tables {
        assert!(table["row_count"].as_u64().is_some());
        assert_eq!(
            table["total_bytes"].as_u64().unwrap(),
            table["table_bytes"].as_u64().unwrap() + table["index_bytes"].as_u64().unwrap()
        );
    }
    let response = client
        .get("/api/reporting/entry-statuses")
        .header("authorization", &token)
        .send()
        .await;
    response.assert_status_is_ok();
    let empty: Value = response.json().await.value().deserialize();
    assert_eq!(empty["generated_at"], "2026-12-31T23:59:59Z");
    assert_eq!(empty["window_start"], "2026-12-31T00:00:00Z");
    assert_eq!(empty["window_end"], "2027-01-01T00:00:00Z");
    assert_eq!(empty["command_entries"], json!([]));
    assert_eq!(empty["event_entries"], json!([]));
    pool.get()
        .unwrap()
        .batch_execute(
            "INSERT INTO command_entries (id, command_type, payload, status, scheduled_at) VALUES
         ('00000000-0000-0000-0000-000000000001','test','{}',0,'2026-12-31T12:00:00Z'),
         ('00000000-0000-0000-0000-000000000002','test','{}',0,'2026-12-31T12:00:00Z'),
         ('00000000-0000-0000-0000-000000000003','test','{}',99,'2026-12-31T12:00:00Z');
         INSERT INTO event_entries (id, event_type, payload, status, scheduled_at) VALUES
         ('00000000-0000-0000-0000-000000000004','test','{}',5,'2026-12-31T12:00:00Z');",
        )
        .unwrap();
    let response = client
        .get("/api/reporting/entry-statuses")
        .header("authorization", &token)
        .send()
        .await;
    response.assert_status_is_ok();
    let report: Value = response.json().await.value().deserialize();
    assert_eq!(
        report["command_entries"],
        json!([
            {"status":0,"status_text":"received","count":2},
            {"status":99,"status_text":"unknown","count":1}
        ])
    );
    assert_eq!(
        report["event_entries"],
        json!([{"status":5,"status_text":"completed","count":1}])
    );
    pool.get()
        .unwrap()
        .batch_execute("DROP TABLE event_entries")
        .unwrap();
    client
        .get("/api/reporting/entry-statuses")
        .header("authorization", &token)
        .send()
        .await
        .assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    client
        .get("/api/reporting/database-tables")
        .header("authorization", &token)
        .send()
        .await
        .assert_status_is_ok();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn identifiers_insert_preserves_identity_and_rejects_duplicates() {
    use diesel::sql_query;
    use std::env::var as std_env_var;
    use url::Url as UrlUrl;

    use crate::infra::repository::PostgresRetailerOfferIdentifiersRepository;
    use crate::offers::handler::RetailerOfferIdentifiersRepository;
    use diesel::{QueryableByName, RunQueryDsl, sql_types::Text};
    use shared::retailer::RetailerCode;

    #[derive(QueryableByName)]
    struct Identity {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        retailer_code: String,
        #[diesel(sql_type = Text)]
        url: String,
    }

    let database_url = std_env_var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).expect("connect to test database"),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut database_url = UrlUrl::parse(&database_url).unwrap();
    database_url
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(
            database_url.to_string(),
        ))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    let repository = PostgresRetailerOfferIdentifiersRepository::new(pool.clone());
    let id = RetailerOfferId::new(Uuid::new_v4().to_string());
    let retailer = RetailerCode::from_str("ankerau").unwrap();
    let url = "https://example.com/Charger?color=red#details";
    assert!(repository.find(retailer, url).unwrap().is_none());
    repository.insert(&id, retailer, url).unwrap();
    let found = repository.find(retailer, url).unwrap().unwrap();
    assert_eq!(found.id, id);
    assert_eq!(found.retailer_code, retailer);
    assert_eq!(found.url, url);
    assert!(
        repository
            .find(retailer, &url.to_lowercase())
            .unwrap()
            .is_none()
    );
    assert!(
        repository
            .find(RetailerCode::AnkerCom, url)
            .unwrap()
            .is_none()
    );
    assert!(
        repository
            .find(retailer, &format!("{url}/"))
            .unwrap()
            .is_none()
    );

    // Renaming in either direction must preserve existing identities.
    {
        let mut connection = pool.get().unwrap();
        // Revert later repairs too, so this still reaches the rename migration.
        while connection
            .revert_last_migration(MIGRATIONS)
            .unwrap()
            .to_string()
            != "20261019000003"
        {}
        let previous: Identity =
            sql_query("SELECT id::text, retailer_code, url FROM offer_identities")
                .get_result(&mut connection)
                .unwrap();
        assert_eq!(previous.id, id.value());
        assert_eq!(previous.retailer_code, "ankerau");
        assert_eq!(previous.url, url);
        connection.run_pending_migrations(MIGRATIONS).unwrap();
    }
    let row: Identity = sql_query("SELECT id::text, retailer_code, url FROM read_offer_identities")
        .get_result(&mut pool.get().unwrap())
        .unwrap();
    assert_eq!(row.id, id.value());
    assert_eq!(row.retailer_code, "ankerau");
    assert_eq!(row.url, url);
    assert!(
        repository
            .insert(&id, retailer, "https://example.com/other")
            .is_err()
    );
    assert!(
        repository
            .insert(
                &RetailerOfferId::new(Uuid::new_v4().to_string()),
                retailer,
                url
            )
            .is_err()
    );
    assert!(
        repository
            .insert(&RetailerOfferId::new("not-a-uuid"), retailer, url)
            .is_err()
    );
    repository
        .insert(
            &RetailerOfferId::new(Uuid::new_v4().to_string()),
            retailer,
            "https://example.com/charger?color=red#details",
        )
        .unwrap();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn read_repositories_insert_contracts() {
    use crate::read_offer::repository::PostgresRetailerOfferRepository as ReadOfferRepositoryPostgresRetailerOfferRepository;
    use crate::read_offer::repository::tests::assert_insert_contract;
    use crate::read_offers_identifiers::repository::PostgresRetailerOfferIdentifiersRepository as ReadOffersIdentifiersRepositoryPostgresRetailerOfferIdentifiersRepository;
    use crate::read_offers_identifiers::repository::tests::assert_insert_contract as repository_tests_assert_insert_contract;
    use chrono::DateTime as ChronoDateTime;
    use chrono::Utc as ChronoUtc;
    use diesel::sql_query as diesel_sql_query2;
    use std::env::var as std_env_var2;
    use url::Url as UrlUrl2;

    let database_url = std_env_var2("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).expect("connect to test database"),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut database_url = UrlUrl2::parse(&database_url).unwrap();
    database_url
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(
            database_url.to_string(),
        ))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();

    // Verify a fresh schema can roll back completely and be rebuilt.
    {
        use diesel::sql_query as diesel_sql_query;

        let mut connection = pool.get().unwrap();
        connection.revert_all_migrations(MIGRATIONS).unwrap();
        assert!(connection.applied_migrations().unwrap().is_empty());
        use diesel::RunQueryDsl;
        #[derive(diesel::QueryableByName)]
        struct TableName {
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
        }
        let tables = |connection: &mut PgConnection| {
            diesel_sql_query(
                "SELECT tablename::text AS name FROM pg_tables WHERE schemaname = current_schema() AND tablename <> '__diesel_schema_migrations' ORDER BY tablename",
            ).load::<TableName>(connection).unwrap().into_iter().map(|row| row.name).collect::<Vec<_>>()
        };
        assert!(tables(&mut connection).is_empty());
        connection.run_pending_migrations(MIGRATIONS).unwrap();
        assert_eq!(
            tables(&mut connection),
            vec![
                "command_entries",
                "event_entries",
                "event_store",
                "inbox_entries",
                "outbox_entries",
                "read_offer_identities",
                "read_offer_images",
                "read_offers",
            ]
        );
    }
    assert_insert_contract(&ReadOfferRepositoryPostgresRetailerOfferRepository::new(
        pool.clone(),
    ));
    repository_tests_assert_insert_contract(
        &ReadOffersIdentifiersRepositoryPostgresRetailerOfferIdentifiersRepository::new(
            pool.clone(),
        ),
    );
    #[derive(diesel::QueryableByName)]
    struct StoredOffer {
        #[diesel(sql_type = diesel::sql_types::Text)]
        title: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        price_amount: i64,
        #[diesel(sql_type = diesel::sql_types::Text)]
        currency: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: ChronoDateTime<ChronoUtc>,
    }
    use diesel::RunQueryDsl;
    let rows =
        diesel_sql_query2("SELECT title, price_amount, currency, created_at FROM read_offers")
            .load::<StoredOffer>(&mut pool.get().unwrap())
            .unwrap();
    assert_eq!(rows.len(), 2);
    for row in rows {
        assert_eq!(row.title, "Charger");
        assert_eq!(row.price_amount, 1999);
        assert_eq!(row.currency, "AUD");
        assert_eq!(
            row.created_at,
            ChronoDateTime::from_timestamp(1_700_000_000, 123_000_000).unwrap()
        );
    }
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn registered_projectors_create_and_update_read_models() {
    use crate::app_with_reporting_token as imported_app_with_reporting_token;
    use crate::read_offer::repository::PostgresRetailerOfferRepository as ReadOfferRepositoryPostgresRetailerOfferRepository2;
    use crate::start_mulac as imported_start_mulac;
    use crate::subscribers::customer_facing::CustomerFacingSubscriber as SubscribersCustomerFacingCustomerFacingSubscriber;
    use diesel::sql_query as diesel_sql_query3;
    use diesel::sql_types::Jsonb as DieselSqlTypesJsonb;
    use diesel::sql_types::Text as DieselSqlTypesText;
    use diesel::sql_types::Uuid as DieselSqlTypesUuid;
    use kernel::NewEventEnvelope;
    use money::Currency as MoneyCurrency;
    use money::Money as MoneyMoney;
    use serde_json::from_value;
    use serde_json::to_string as serde_json_to_string;
    use std::env::var as std_env_var3;
    use url::Url as UrlUrl3;

    let database_url = std_env_var3("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).expect("connect to test database"),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut database_url = UrlUrl3::parse(&database_url).unwrap();
    database_url
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(
            database_url.to_string(),
        ))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();

    use crate::offers::event::{OfferPriceChangedEvent, OfferTitleChangedEvent};
    use diesel::RunQueryDsl;
    use kernel::ApplicationEvent;
    let (kernel, state) = imported_start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(imported_app_with_reporting_token(state, "test".to_owned()));
    let response = client
        .post("/api/tmp-import")
        .body_json(&json!({
            "retailer_code": "ankerau", "title": "Charger", "price_amount": 1999,
            "currency": "AUD", "url": "https://example.com/Charger?q=red#details"
        }))
        .send()
        .await;
    response.assert_status(StatusCode::ACCEPTED);
    run_queue(&client, "inbox", 10, 1).await;
    run_queue(&client, "commands", 10, 1).await;
    run_queue(&client, "events", 10, 1).await;
    run_queue(&client, "events", 10, 0).await;
    #[derive(diesel::QueryableByName)]
    struct Projection {
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        data: Value,
    }
    let read = || {
        diesel_sql_query3("SELECT to_jsonb(o) AS data FROM read_offers o JOIN read_offer_identities i USING (id, retailer_code, url)")
        .get_result::<Projection>(&mut pool.get().unwrap()).unwrap().data
    };
    let before = read();
    let id = RetailerOfferId::new(before["id"].as_str().unwrap());
    assert_eq!(before["id"], id.value());
    assert_eq!(before["title"], "Charger");
    assert_eq!(before["price_amount"], 1999);
    assert_eq!(before["currency"], "AUD");
    assert_eq!(before["retailer_code"], "ankerau");
    assert_eq!(before["url"], "https://example.com/Charger?q=red#details");
    let title = OfferTitleChangedEvent {
        id: id.clone(),
        title: "Updated charger".to_owned(),
    };
    let price = OfferPriceChangedEvent {
        id: id.clone(),
        price: MoneyMoney::new(1299, MoneyCurrency::EUR),
    };
    for (event_type, payload) in [
        (title.event_type(), serde_json_to_string(&title).unwrap()),
        (price.event_type(), serde_json_to_string(&price).unwrap()),
    ] {
        let event_id = Uuid::new_v4();
        diesel_sql_query3(
            "INSERT INTO event_entries (id, event_type, payload, meta) VALUES ($1, $2, $3, $4)",
        )
        .bind::<DieselSqlTypesUuid, _>(event_id)
        .bind::<DieselSqlTypesText, _>(event_type)
        .bind::<DieselSqlTypesText, _>(payload)
        .bind::<DieselSqlTypesJsonb, _>(json!({
            "event_id": event_id,
            "correlation_id": Uuid::new_v4(),
            "causation_id": Uuid::new_v4(),
            "source": "offers-test"
        }))
        .execute(&mut pool.get().unwrap())
        .unwrap();
        run_queue(&client, "events", 10, 1).await;
    }
    let mut expected = before;
    expected["title"] = json!("Updated charger");
    expected["content"][0]["title"] = json!("Updated charger");
    expected["price_amount"] = json!(1299);
    expected["currency"] = json!("EUR");
    assert_eq!(read(), expected);
    // Registration covers all offer events, preserving their wire payloads and metadata.
    use kernel::EventSubscriberPort;
    let subscriber = SubscribersCustomerFacingCustomerFacingSubscriber::new(pool.clone());
    let events = diesel_sql_query3("SELECT to_jsonb(e) AS data FROM event_entries e")
        .load::<Projection>(&mut pool.get().unwrap())
        .unwrap();
    assert_eq!(events.len(), 3);
    for event in events {
        let envelope = NewEventEnvelope {
            event_type: event.data["event_type"].as_str().unwrap().to_owned(),
            payload: event.data["payload"].as_str().unwrap().to_owned(),
            metadata: Some(from_value(event.data["meta"].clone()).unwrap()),
        };
        let read_outbox = || {
            diesel_sql_query3("SELECT to_jsonb(o) AS data FROM outbox_entries o WHERE id = $1")
                .bind::<DieselSqlTypesUuid, _>(envelope.metadata.as_ref().unwrap().event_id)
                .get_result::<Projection>(&mut pool.get().unwrap())
                .unwrap()
                .data
        };
        let before = read_outbox();
        assert_eq!(before["payload"], event.data["payload"]);
        assert_eq!(before["meta"]["event_type"], event.data["event_type"]);
        assert_eq!(before["meta"]["routing_key"], "customer_facing");
        assert_eq!(before["meta"]["content_type"], "application/json");
        for key in ["event_id", "correlation_id", "causation_id", "source"] {
            assert_eq!(before["meta"][key], event.data["meta"][key]);
        }
        assert_eq!(before["status"], 0);
        subscriber.handle(&envelope).unwrap();
        assert_eq!(read_outbox(), before);
    }
    #[derive(diesel::QueryableByName)]
    struct OutboxCount {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }
    let count = diesel_sql_query3("SELECT count(*) AS count FROM outbox_entries")
        .get_result::<OutboxCount>(&mut pool.get().unwrap())
        .unwrap()
        .count;
    assert_eq!(count, 3);
    use crate::read_offer::repository::RetailerOfferRepository;
    let repository = ReadOfferRepositoryPostgresRetailerOfferRepository2::new(pool);
    let missing = RetailerOfferId::new(Uuid::new_v4().to_string());
    assert!(repository.update_title(&missing, "missing").is_err());
    assert!(repository.update_price(&missing, &price.price).is_err());
    kernel.shutdown();
}

pub(crate) async fn run_queue(
    client: &TestClient<Route>,
    queue: &str,
    limit: usize,
    processed: usize,
) {
    let response = client
        .post(format!("/api/{queue}/run"))
        .body_json(&json!({"limit": limit}))
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(json!({"processed": processed, "errors": []}))
        .await;
}

pub(crate) fn inbox_test_database() -> (TestSchema, DbPool) {
    let database_url = var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).unwrap(),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut url = Url::parse(&database_url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(url.to_string()))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    (schema, pool)
}

#[derive(diesel::QueryableByName)]
struct InboxTestSnapshot {
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    data: Value,
}

pub(crate) fn inbox_test_snapshot(pool: &DbPool, sql: &str) -> Value {
    use diesel::sql_query as diesel_sql_query4;

    use diesel::RunQueryDsl;
    diesel_sql_query4(sql)
        .get_result::<InboxTestSnapshot>(&mut pool.get().unwrap())
        .unwrap()
        .data
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn inbox_submission_preserves_payload_metadata_and_duplicate_rows() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app_with_reporting_token(state, "test".into()));
    let message_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let input = json!({
        "payload":{"nested":[null,true,42,"é",{"key":"value"}]},
        "routing_key":" unknown.command ",
        "message_id":message_id,
        "correlation_id":correlation_id,
        "source":"external-source"
    });
    let response = client.post("/api/inbox").body_json(&input).send().await;
    response.assert_status(StatusCode::ACCEPTED);
    response.assert_json(json!({"message_id":message_id})).await;
    let sql = "SELECT to_jsonb(i) AS data FROM inbox_entries i";
    let original = inbox_test_snapshot(&pool, sql);
    assert_eq!(original["id"], message_id.to_string());
    assert_eq!(
        from_str::<Value>(original["payload"].as_str().unwrap()).unwrap(),
        input["payload"]
    );
    assert_eq!(
        original["meta"],
        json!({
            "message_id":message_id,"correlation_id":correlation_id,
            "source":"external-source","routing_key":" unknown.command "
        })
    );
    assert_eq!(original["status"], 0);
    assert_eq!(original["attempts"], 0);
    assert!(original["reservation_id"].is_null());
    assert!(original["processed_at"].is_null());

    // A changed envelope with the same ID is successful and changes no column.
    let response = client
        .post("/api/inbox")
        .body_json(&json!({
            "message_id":message_id,"payload":"replacement","routing_key":"different",
            "source":"different","correlation_id":Uuid::new_v4()
        }))
        .send()
        .await;
    response.assert_status(StatusCode::ACCEPTED);
    response.assert_json(json!({"message_id":message_id})).await;
    assert_eq!(inbox_test_snapshot(&pool, sql), original);

    // Rejected envelopes must not insert messages.
    for body in invalid_submissions() {
        client
            .post("/api/inbox")
            .body_json(&body)
            .send()
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
    assert_eq!(inbox_test_snapshot(&pool, sql), original);

    let mut ids = HashSet::from([message_id]);
    for payload in [
        json!(0),
        json!([1, "two"]),
        json!("text"),
        json!(42),
        json!(true),
    ] {
        use diesel::sql_query as diesel_sql_query5;
        use diesel::sql_types::Uuid as DieselSqlTypesUuid2;
        use serde_json::from_str as serde_json_from_str;

        let response = client
            .post("/api/inbox")
            .body_json(&json!({"payload":payload,"routing_key":"unknown","source":" crawler "}))
            .send()
            .await;
        response.assert_status(StatusCode::ACCEPTED);
        let body: Value = response.json().await.value().deserialize();
        let id = Uuid::parse_str(body["message_id"].as_str().unwrap()).unwrap();
        assert_eq!(body.as_object().unwrap().len(), 1);
        assert!(!id.is_nil());
        assert!(ids.insert(id));
        use diesel::RunQueryDsl;
        let stored =
            diesel_sql_query5("SELECT to_jsonb(i) AS data FROM inbox_entries i WHERE id = $1")
                .bind::<DieselSqlTypesUuid2, _>(id)
                .get_result::<InboxTestSnapshot>(&mut pool.get().unwrap())
                .unwrap()
                .data;
        assert_eq!(
            serde_json_from_str::<Value>(stored["payload"].as_str().unwrap()).unwrap(),
            payload
        );
        assert_eq!(
            stored["meta"],
            json!({"message_id":id,"correlation_id":null,"source":" crawler ","routing_key":"unknown"})
        );
    }
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_build_object('inbox', (SELECT count(*) FROM inbox_entries), 'commands', (SELECT count(*) FROM command_entries), 'events', (SELECT count(*) FROM event_entries), 'offers', (SELECT count(*) FROM read_offers)) AS data"
        ),
        json!({"inbox":6,"commands":0,"events":0,"offers":0})
    );

    // An actual SQL failure must also hide database diagnostics.
    pool.get()
        .unwrap()
        .batch_execute("DROP TABLE inbox_entries")
        .unwrap();
    let response = client.post("/api/inbox").body_json(&input).send().await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    assert_error_response(response).await;
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn inbox_submission_processes_offer_through_all_http_stages() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app_with_reporting_token(state, "test".into()));
    let command = ImportOfferDetailsCommand {
        was_price: Default::default(),
        availability: Default::default(),
        item_condition: Default::default(),
        content: None,
        gtin: None,
        sku: None,
        external_identifiers: Default::default(),
        localization: None,
        scraped_at: None,
        images: Vec::new(),
        retailer_code: SharedRetailerRetailerCode::AnkerAu,
        title: "USB charger".into(),
        price: Money::new(1999, Currency::AUD),
        url: "https://example.com/Charger?q=red#details".into(),
    };
    let response = client
        .post("/api/inbox")
        .body_json(&json!({
            "routing_key":CREATE_FROM_OFFER_DETAILS_COMMAND,
            "source":"crawler",
            "payload": {
                "retailer_code":"AnkerAu", "title":"USB charger",
                "price":{"amount":1999,"currency":"AUD"},
                "url":"https://example.com/Charger?q=red#details"
            }
        }))
        .send()
        .await;
    response.assert_status(StatusCode::ACCEPTED);
    let counts = || {
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_build_object('commands', (SELECT count(*) FROM command_entries), 'events', (SELECT count(*) FROM event_entries), 'offers', (SELECT count(*) FROM read_offers)) AS data",
        )
    };
    assert_eq!(counts(), json!({"commands":0,"events":0,"offers":0}));
    run_queue(&client, "inbox", 10, 1).await;
    assert_eq!(counts(), json!({"commands":1,"events":0,"offers":0}));
    run_queue(&client, "commands", 10, 1).await;
    assert_eq!(counts(), json!({"commands":1,"events":1,"offers":0}));
    run_queue(&client, "events", 10, 1).await;
    let projection = inbox_test_snapshot(
        &pool,
        "SELECT to_jsonb(o) AS data FROM read_offers o JOIN read_offer_identities i USING (id, retailer_code, url)",
    );
    Uuid::parse_str(projection["id"].as_str().unwrap()).unwrap();
    assert_eq!(projection["title"], command.title);
    assert_eq!(projection["retailer_code"], "ankerau");
    assert_eq!(projection["url"], command.url);
    assert_eq!(projection["price_amount"], 1999);
    assert_eq!(projection["currency"], "AUD");
    for queue in ["inbox", "commands", "events"] {
        run_queue(&client, queue, 10, 0).await;
    }
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn queue_endpoints_process_one_stage_and_preserve_unavailable_outbox() {
    use crate::app_with_reporting_token as imported_app_with_reporting_token2;
    use crate::offers::command::CREATE_FROM_OFFER_DETAILS_COMMAND as OffersCommandCREATE_FROM_OFFER_DETAILS_COMMAND;
    use crate::offers::command::ImportOfferDetailsCommand as OffersCommandImportOfferDetailsCommand;
    use crate::start_mulac as imported_start_mulac2;
    use diesel::sql_query as diesel_sql_query6;
    use diesel::sql_types::Integer;
    use money::Currency as MoneyCurrency2;
    use money::Money as MoneyMoney2;
    use serde_json::to_string as serde_json_to_string2;
    use shared::retailer::RetailerCode as SharedRetailerRetailerCode2;
    use std::env::var as std_env_var4;
    use url::Url as UrlUrl4;

    let database_url = std_env_var4("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).unwrap(),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut url = UrlUrl4::parse(&database_url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(url.to_string()))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    let (kernel, state) = imported_start_mulac2(pool.clone()).unwrap();
    let client = TestClient::new(imported_app_with_reporting_token2(state, "test".into()));
    use diesel::{
        RunQueryDsl,
        sql_types::{Jsonb, Text, Uuid as SqlUuid},
    };
    for number in 0..3 {
        let command = OffersCommandImportOfferDetailsCommand {
            was_price: Default::default(),
            availability: Default::default(),
            item_condition: Default::default(),
            content: None,
            gtin: None,
            sku: None,
            external_identifiers: Default::default(),
            localization: None,
            scraped_at: None,
            images: Vec::new(),
            retailer_code: SharedRetailerRetailerCode2::AnkerAu,
            title: "Charger".into(),
            price: MoneyMoney2::new(1999, MoneyCurrency2::AUD),
            url: format!("https://example.com/charger/{number}"),
        };
        diesel_sql_query6("INSERT INTO inbox_entries (id, payload, meta, scheduled_at) VALUES ($1, $2, $3, NOW() + $4 * INTERVAL '1 day')")
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .bind::<Text, _>(serde_json_to_string2(&command).unwrap())
            .bind::<Jsonb, _>(json!({"routing_key": OffersCommandCREATE_FROM_OFFER_DETAILS_COMMAND}))
            .bind::<Integer, _>(if number == 2 { 1 } else { 0 })
            .execute(&mut pool.get().unwrap()).unwrap();
    }
    // Each endpoint advances only its own queue, and respects the batch limit.
    run_queue(&client, "commands", 10, 0).await;
    run_queue(&client, "inbox", 1, 1).await;
    run_queue(&client, "events", 10, 0).await;
    run_queue(&client, "commands", 10, 1).await;
    run_queue(&client, "commands", 10, 0).await;
    run_queue(&client, "events", 10, 1).await;
    run_queue(&client, "inbox", 10, 1).await;
    run_queue(&client, "inbox", 10, 0).await;
    run_queue(&client, "commands", 1, 1).await;
    run_queue(&client, "events", 1, 1).await;
    run_queue(&client, "events", 10, 0).await;

    #[derive(diesel::QueryableByName)]
    struct Snapshot {
        #[diesel(sql_type = Jsonb)]
        data: Value,
    }
    let snapshot = |sql: &str| {
        diesel_sql_query6(sql)
            .get_result::<Snapshot>(&mut pool.get().unwrap())
            .unwrap()
            .data
    };
    let projections = snapshot(
        "SELECT to_jsonb(count(*)) AS data FROM read_offers JOIN read_offer_identities USING (id, retailer_code, url)",
    );
    assert_eq!(projections, 2);

    pool.get()
        .unwrap()
        .batch_execute(
            "INSERT INTO inbox_entries (id, payload, meta) VALUES (gen_random_uuid(), '{}', '{}');
         INSERT INTO outbox_entries (id, payload, meta) VALUES (gen_random_uuid(), '{}', '{}');",
        )
        .unwrap();
    let response = client
        .post("/api/inbox/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await;
    response.assert_status_is_ok();
    let report: Value = response.json().await.value().deserialize();
    assert_eq!(report["processed"], Value::Null);
    assert_eq!(report["errors"].as_array().unwrap().len(), 1);
    let failed = snapshot(
        "SELECT jsonb_build_object('attempts', attempts, 'reserved', reservation_id IS NOT NULL, 'scheduled_later', scheduled_at > NOW(), 'processed', processed_at IS NOT NULL) AS data FROM inbox_entries WHERE meta = '{}'::jsonb",
    );
    assert_eq!(
        failed,
        json!({"attempts":1, "reserved":false, "scheduled_later":true, "processed":false})
    );
    run_queue(&client, "inbox", 10, 0).await;

    let outbox_sql = "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM outbox_entries o";
    let before = snapshot(outbox_sql);
    client
        .post("/api/outbox/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await
        .assert_status(StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(snapshot(outbox_sql), before);
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn outbox_delivery_completes_only_confirmed_items_and_retries_failures() {
    use crate::app_with_reporting_token as imported_app_with_reporting_token3;
    use crate::offers::event::OFFER_CREATED_EVENT;
    use crate::start_mulac as imported_start_mulac3;
    use crate::subscribers::customer_facing::CustomerFacingSubscriber as SubscribersCustomerFacingCustomerFacingSubscriber2;
    use diesel::sql_query as diesel_sql_query7;
    use kernel::NewEventEnvelope as KernelNewEventEnvelope;
    use kernel::NewEventMetadata;
    use outbox::prelude::AsyncOutboxService;
    use outbox::prelude::poem::OutboxAdapter;
    use std::env::var as std_env_var5;
    use url::Url as UrlUrl5;

    let database_url = std_env_var5("DATABASE_URL_TEST").expect("DATABASE_URL_TEST is required");
    let mut schema = TestSchema {
        connection: PgConnection::establish(&database_url).unwrap(),
        name: format!("offers_test_{}", Uuid::new_v4().simple()),
    };
    schema
        .connection
        .batch_execute(&format!("CREATE SCHEMA {}", schema.name))
        .unwrap();
    let mut url = UrlUrl5::parse(&database_url).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={}", schema.name));
    let pool = DbPool::builder()
        .max_size(2)
        .build(ConnectionManager::<PgConnection>::new(url.to_string()))
        .unwrap();
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();

    use diesel::RunQueryDsl;
    use outbox::prelude::{OutboundMessageEnvelope, OutboxError, OutboxPublisherPort};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    struct Publisher(AtomicBool);
    impl OutboxPublisherPort for Publisher {
        fn publish(&self, message: OutboundMessageEnvelope) -> Result<(), OutboxError> {
            assert_eq!(message.metadata.routing_key, "customer_facing");
            if self.0.load(Ordering::SeqCst) {
                Ok(())
            } else {
                Err(err_test_delivery())
            }
        }
    }
    fn err_test_delivery() -> OutboxError {
        OutboxError::Transport("test broker unavailable".into())
    }
    let publisher = Arc::new(Publisher(AtomicBool::new(false)));
    let (kernel, mut state) = imported_start_mulac3(pool.clone()).unwrap();
    state.outbox = OutboxAdapter::new(AsyncOutboxService::from_postgres(
        pool.clone(),
        publisher.clone(),
    ));
    let client = TestClient::new(imported_app_with_reporting_token3(state, "test".into()));
    use kernel::EventSubscriberPort;
    let subscriber = SubscribersCustomerFacingCustomerFacingSubscriber2::new(pool.clone());
    for _ in 0..2 {
        subscriber
            .handle(&KernelNewEventEnvelope {
                event_type: OFFER_CREATED_EVENT.into(),
                payload: "{}".into(),
                metadata: Some(NewEventMetadata {
                    sequence: None,
                    event_id: Uuid::new_v4(),
                    correlation_id: None,
                    causation_id: None,
                    source: Some("offers-test".into()),
                }),
            })
            .unwrap();
    }
    // Existing rows used null IDs; the migration makes them consumable too.
    pool.get()
        .unwrap()
        .batch_execute(
            "UPDATE outbox_entries SET meta = jsonb_set(meta, '{message_id}', 'null'::jsonb)",
        )
        .unwrap();
    pool.get()
        .unwrap()
        .batch_execute(include_str!(
            "../migrations/2026-10-21-000001_outbox_message_ids/up.sql"
        ))
        .unwrap();
    let response = client
        .post("/api/outbox/run")
        .body_json(&json!({"limit":1}))
        .send()
        .await;
    response.assert_status_is_ok();
    let body: Value = response.json().await.value().deserialize();
    assert!(body["processed"].is_null());
    assert_eq!(body["errors"].as_array().unwrap().len(), 1);
    #[derive(diesel::QueryableByName)]
    struct Snapshot {
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        data: Value,
    }
    let failed: Snapshot = diesel_sql_query7("SELECT jsonb_build_object('attempts', attempts, 'reserved', reservation_id IS NOT NULL, 'scheduled_later', scheduled_at > NOW(), 'processed', processed_at IS NOT NULL) AS data FROM outbox_entries WHERE attempts = 1").get_result(&mut pool.get().unwrap()).unwrap();
    assert_eq!(
        failed.data,
        json!({"attempts":1,"reserved":false,"scheduled_later":true,"processed":false})
    );
    publisher.0.store(true, Ordering::SeqCst);
    run_queue(&client, "outbox", 1, 1).await;
    run_queue(&client, "outbox", 10, 0).await;
    pool.get()
        .unwrap()
        .batch_execute("UPDATE outbox_entries SET scheduled_at = NOW() WHERE processed_at IS NULL")
        .unwrap();
    run_queue(&client, "outbox", 1, 1).await;
    run_queue(&client, "outbox", 10, 0).await;
    let completed: Snapshot = diesel_sql_query7("SELECT to_jsonb(count(*)) AS data FROM outbox_entries WHERE processed_at IS NOT NULL AND reservation_id IS NULL").get_result(&mut pool.get().unwrap()).unwrap();
    assert_eq!(completed.data, 2);
    kernel.shutdown();
}

fn insert_command(pool: &DbPool, id: Uuid, command_type: &str, payload: &str) {
    use diesel::sql_query as diesel_sql_query8;

    use diesel::{
        RunQueryDsl,
        sql_types::{Text, Uuid as SqlUuid},
    };
    diesel_sql_query8(
        "INSERT INTO command_entries (id, command_type, payload) VALUES ($1, $2, $3)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(command_type)
    .bind::<Text, _>(payload)
    .execute(&mut pool.get().unwrap())
    .unwrap();
}

fn command_snapshot(pool: &DbPool, id: Uuid) -> Value {
    use diesel::sql_query as diesel_sql_query9;
    use diesel::sql_types::Uuid as DieselSqlTypesUuid3;

    use diesel::RunQueryDsl;
    diesel_sql_query9(
        "SELECT jsonb_build_object('status', status, 'attempts', attempts, 'reserved', reservation_id IS NOT NULL, 'scheduled_later', scheduled_at > NOW(), 'processed', processed_at IS NOT NULL) AS data FROM command_entries WHERE id = $1",
    )
    .bind::<DieselSqlTypesUuid3, _>(id)
    .get_result::<InboxTestSnapshot>(&mut pool.get().unwrap())
    .unwrap()
    .data
}

async fn run_commands_expecting_errors(client: &TestClient<Route>, pool: &DbPool, errors: Value) {
    let response = client
        .post("/api/commands/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await;
    response.assert_status_is_ok();
    assert_batch_response(response, errors.as_array().unwrap().len()).await;
    let history = inbox_test_snapshot(
        pool,
        "SELECT jsonb_agg(extra_info) AS data FROM command_entries WHERE status = 4",
    )
    .to_string();
    for error in errors.as_array().unwrap() {
        assert!(history.contains(error.as_str().unwrap()), "{history}");
    }
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn commands_run_stops_retrying_after_six_failures() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app_with_reporting_token(state, "test".into()));
    let id = Uuid::new_v4();
    insert_command(&pool, id, "unregistered", "{}");
    let not_found = json!(["no handler registered for command type 'unregistered'"]);

    run_commands_expecting_errors(&client, &pool, not_found.clone()).await;
    assert_eq!(
        command_snapshot(&pool, id),
        json!({"status":4, "attempts":1, "reserved":false, "scheduled_later":true, "processed":false})
    );
    // The backoff keeps the failed command out of the next batch.
    run_queue(&client, "commands", 10, 0).await;

    for attempts in 2..=6 {
        pool.get()
            .unwrap()
            .batch_execute("UPDATE command_entries SET scheduled_at = NOW()")
            .unwrap();
        run_commands_expecting_errors(&client, &pool, not_found.clone()).await;
        let status = 4;
        assert_eq!(
            command_snapshot(&pool, id),
            json!({"status":status, "attempts":attempts, "reserved":false, "scheduled_later":true, "processed":false})
        );
    }
    // Exhausted commands stay failed and are never reserved again.
    pool.get()
        .unwrap()
        .batch_execute("UPDATE command_entries SET scheduled_at = NOW()")
        .unwrap();
    run_queue(&client, "commands", 10, 0).await;
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM event_entries"
        ),
        0
    );
    kernel.shutdown();
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn commands_run_keeps_successful_commands_of_a_failed_batch() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app_with_reporting_token(state, "test".into()));
    let command = ImportOfferDetailsCommand {
        was_price: Default::default(),
        availability: Default::default(),
        item_condition: Default::default(),
        content: None,
        gtin: None,
        sku: None,
        external_identifiers: Default::default(),
        localization: None,
        scraped_at: None,
        images: Vec::new(),
        retailer_code: SharedRetailerRetailerCode::AnkerAu,
        title: "Charger".into(),
        price: Money::new(1999, Currency::AUD),
        url: "https://example.com/charger".into(),
    };
    let valid = Uuid::new_v4();
    let unregistered = Uuid::new_v4();
    insert_command(
        &pool,
        valid,
        CREATE_FROM_OFFER_DETAILS_COMMAND,
        &serde_json_to_string3(&command).unwrap(),
    );
    insert_command(&pool, unregistered, "unregistered", "{}");

    run_commands_expecting_errors(
        &client,
        &pool,
        json!(["no handler registered for command type 'unregistered'"]),
    )
    .await;
    assert_eq!(
        command_snapshot(&pool, valid),
        json!({"status":5, "attempts":1, "reserved":false, "scheduled_later":false, "processed":true})
    );
    assert_eq!(command_snapshot(&pool, unregistered)["status"], 4);
    // The completed command's event is recorded but not dispatched.
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_build_object('events', (SELECT count(*) FROM event_entries WHERE status = 0), 'offers', (SELECT count(*) FROM read_offers)) AS data"
        ),
        json!({"events":1,"offers":0})
    );
    kernel.shutdown();
}

fn event_snapshot(pool: &DbPool, id: Uuid) -> Value {
    use diesel::sql_query as diesel_sql_query10;
    use diesel::sql_types::Uuid as DieselSqlTypesUuid4;

    use diesel::RunQueryDsl;
    diesel_sql_query10(
        "SELECT jsonb_build_object('status', status, 'attempts', attempts, 'reserved', reservation_id IS NOT NULL, 'scheduled_later', scheduled_at > NOW(), 'processed', processed_at IS NOT NULL) AS data FROM event_entries WHERE id = $1",
    )
    .bind::<DieselSqlTypesUuid4, _>(id)
    .get_result::<InboxTestSnapshot>(&mut pool.get().unwrap())
    .unwrap()
    .data
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn events_run_reschedules_subscriber_failures_and_keeps_completed_events() {
    use crate::app_with_reporting_token as imported_app_with_reporting_token4;
    use crate::start_mulac as imported_start_mulac4;
    use crate::tests::assert_batch_response as tests_assert_batch_response;
    use diesel::sql_query as diesel_sql_query11;
    use diesel::sql_types::Uuid as DieselSqlTypesUuid5;

    use diesel::RunQueryDsl;
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = imported_start_mulac4(pool.clone()).unwrap();
    let client = TestClient::new(imported_app_with_reporting_token4(state, "test".into()));
    client
        .post("/api/tmp-import")
        .body_json(&json!({
            "retailer_code": "ankerau", "title": "Charger", "price_amount": 1999,
            "currency": "AUD", "url": "https://example.com/charger"
        }))
        .send()
        .await
        .assert_status(StatusCode::ACCEPTED);
    run_queue(&client, "inbox", 10, 1).await;
    run_queue(&client, "commands", 10, 1).await;
    // CustomerFacingSubscriber rejects an OfferCreated event without metadata.
    pool.get()
        .unwrap()
        .batch_execute("UPDATE event_entries SET meta = NULL")
        .unwrap();
    let failing = inbox_test_snapshot(&pool, "SELECT to_jsonb(id) AS data FROM event_entries");
    let failing = Uuid::parse_str(failing.as_str().unwrap()).unwrap();
    // An event type without subscribers completes and is counted.
    let unsubscribed = Uuid::new_v4();
    diesel_sql_query11(
        "INSERT INTO event_entries (id, event_type, payload) VALUES ($1, 'unsubscribed', '{}')",
    )
    .bind::<DieselSqlTypesUuid5, _>(unsubscribed)
    .execute(&mut pool.get().unwrap())
    .unwrap();

    let response = client
        .post("/api/events/run")
        .body_json(&json!({"limit":10}))
        .send()
        .await;
    response.assert_status_is_ok();
    tests_assert_batch_response(response, 1).await;
    assert_eq!(
        event_snapshot(&pool, failing),
        json!({"status":4, "attempts":1, "reserved":false, "scheduled_later":true, "processed":false})
    );
    assert_eq!(event_snapshot(&pool, unsubscribed)["status"], 5);
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM outbox_entries"
        ),
        0
    );
    // The backoff keeps the failed event out of the next batch.
    run_queue(&client, "events", 10, 0).await;
    kernel.shutdown();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn image_projection_orders_and_rejects_conflicts_with_memory_parity() {
    use crate::offers::event::OfferFacts;
    use crate::read_offer::entity::RetailerOffer;
    use chrono::Utc as ChronoUtc2;
    use money::Currency as MoneyCurrency3;
    use money::Money as MoneyMoney3;
    use retailer_sauce::prelude::offer_content::singleton;
    use shared::retailer::RetailerCode as SharedRetailerRetailerCode3;

    use crate::offers::event::OfferImageAddedEvent;
    use crate::read_offer::repository::{
        InMemoryRetailerOfferRepository, PostgresRetailerOfferRepository, RetailerOfferRepository,
    };
    let (_schema, pool) = inbox_test_database();
    let repositories: Vec<Box<dyn RetailerOfferRepository>> = vec![
        Box::new(PostgresRetailerOfferRepository::new(pool)),
        Box::new(InMemoryRetailerOfferRepository::new()),
    ];
    for repository in repositories {
        let entity = RetailerOffer {
            facts: OfferFacts {
                content: singleton("Charger".into(), None, None),
                ..Default::default()
            },
            identifiers: Default::default(),
            localization: Default::default(),
            scraped_at: None,
            images: Vec::new(),
            id: RetailerOfferId::new(Uuid::now_v7().to_string()),
            retailer_code: SharedRetailerRetailerCode3::AnkerAu,
            title: "Charger".into(),
            price: MoneyMoney3::new(1999, MoneyCurrency3::AUD),
            url: "https://shop.example/charger".into(),
            created_at: ChronoUtc2::now(),
        };
        let a = OfferImageAddedEvent {
            id: entity.id.clone(),
            image_url: "https://shop.example/A".into(),
            position: 0,
        };
        let b = OfferImageAddedEvent {
            id: entity.id.clone(),
            image_url: "https://shop.example/B".into(),
            position: 1,
        };
        assert!(repository.add_image(&b).is_err());
        repository.insert(&entity).unwrap();
        assert!(repository.images(&entity.id).unwrap().is_empty());
        repository.add_image(&b).unwrap();
        repository.add_image(&a).unwrap();
        repository.add_image(&a).unwrap();
        for event in [
            OfferImageAddedEvent {
                position: 2,
                ..a.clone()
            },
            OfferImageAddedEvent {
                image_url: "https://shop.example/conflict".into(),
                ..a.clone()
            },
        ] {
            assert!(repository.add_image(&event).is_err());
        }
        repository.update_title(&entity.id, "updated").unwrap();
        repository
            .update_price(&entity.id, &MoneyMoney3::new(42, MoneyCurrency3::EUR))
            .unwrap();
        assert_eq!(
            repository.images(&entity.id).unwrap(),
            [a.image_url, b.image_url]
        );
    }
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn image_imports_emit_append_only_events_through_existing_handler() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app_with_reporting_token(state, "test".into()));
    let a = "https://shop.example/A";
    let b = "https://shop.example/B";
    let c = "https://shop.example/C";
    for (images, events) in [
        (json!([a, a, b, "/bad"]), 1),
        (json!([b, c, a, c]), 1),
        (json!([]), 0),
        (json!([c, a, b]), 0),
    ] {
        let input = json!({"retailer_code":"ankerau","title":"charger","price_amount":1999,"currency":"AUD","url":"https://shop.example/charger","images":images});
        client
            .post("/api/tmp-import")
            .body_json(&input)
            .send()
            .await
            .assert_status(StatusCode::ACCEPTED);
        run_queue(&client, "inbox", 100, 1).await;
        run_queue(&client, "commands", 100, 1).await;
        run_queue(&client, "events", 100, events).await;
    }
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_agg(image_url ORDER BY position) AS data FROM read_offer_images"
        ),
        json!([a, b, c])
    );
    assert_eq!(
        inbox_test_snapshot(&pool, "SELECT to_jsonb(count(*)) AS data FROM event_store"),
        2
    );
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM outbox_entries"
        ),
        2
    );
    let created = inbox_test_snapshot(
        &pool,
        "SELECT payload::jsonb AS data FROM outbox_entries WHERE meta->>'event_type'='OfferCreated'",
    );
    assert_eq!(created["images"], json!([a, b]));
    let added = inbox_test_snapshot(
        &pool,
        "SELECT payload::jsonb AS data FROM outbox_entries WHERE meta->>'event_type'='OfferImageAdded'",
    );
    assert_eq!(added["image_url"], c);
    assert_eq!(added["position"], 2);
    kernel.shutdown();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn image_migration_preserves_preexisting_offer_rows() {
    let (_schema, pool) = inbox_test_database();
    let mut connection = pool.get().unwrap();
    connection
        .batch_execute(include_str!(
            "../migrations/2026-10-25-000001_offer_images/down.sql"
        ))
        .unwrap();
    connection.batch_execute("INSERT INTO read_offers (id,retailer_code,url,title,price_amount,currency,created_at,content) VALUES (gen_random_uuid(),'ankerau','https://shop.example/old','Old offer',123,'AUD',now(),'[{\"language\":null,\"title\":\"Old offer\",\"description\":null}]')").unwrap();
    let before = inbox_test_snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o");
    connection
        .batch_execute(include_str!(
            "../migrations/2026-10-25-000001_offer_images/up.sql"
        ))
        .unwrap();
    assert_eq!(
        inbox_test_snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o"),
        before
    );
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM read_offer_images"
        ),
        0
    );
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn creation_projection_rolls_back_parent_and_images_together() {
    use crate::offers::event::OfferFacts as OffersEventOfferFacts;
    use crate::read_offer::entity::RetailerOffer as ReadOfferEntityRetailerOffer;
    use chrono::Utc as ChronoUtc3;
    use money::Currency as MoneyCurrency4;
    use money::Money as MoneyMoney4;
    use retailer_sauce::prelude::offer_content::singleton as retailer_sauce_offer_content_singleton;
    use shared::retailer::RetailerCode as SharedRetailerRetailerCode4;

    use crate::read_offer::repository::{PostgresRetailerOfferRepository, RetailerOfferRepository};
    let (_schema, pool) = inbox_test_database();
    let entity = ReadOfferEntityRetailerOffer {
        facts: OffersEventOfferFacts {
            content: retailer_sauce_offer_content_singleton("Charger".into(), None, None),
            ..Default::default()
        },
        identifiers: Default::default(),
        localization: Default::default(),
        scraped_at: None,
        id: RetailerOfferId::new(Uuid::now_v7().to_string()),
        retailer_code: SharedRetailerRetailerCode4::AnkerAu,
        title: "Charger".into(),
        url: "https://shop.example/atomic".into(),
        price: MoneyMoney4::new(1999, MoneyCurrency4::AUD),
        created_at: ChronoUtc3::now(),
        images: vec![
            "https://shop.example/A".into(),
            "https://shop.example/B".into(),
        ],
    };
    let repository = PostgresRetailerOfferRepository::new(pool.clone());
    pool.get()
        .unwrap()
        .batch_execute(
            "ALTER TABLE read_offer_images ADD CONSTRAINT fail_second_image CHECK (position <> 1)",
        )
        .unwrap();
    assert!(repository.insert(&entity).is_err());
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_build_object('offers',(SELECT count(*) FROM read_offers),'images',(SELECT count(*) FROM read_offer_images)) AS data"
        ),
        json!({"offers":0,"images":0})
    );
    pool.get()
        .unwrap()
        .batch_execute("ALTER TABLE read_offer_images DROP CONSTRAINT fail_second_image")
        .unwrap();
    repository.insert(&entity).unwrap();
    assert_eq!(repository.images(&entity.id).unwrap(), entity.images);
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn scrape_only_import_persists_history_projection_and_outbox() {
    let (_schema, pool) = inbox_test_database();
    let (_kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app_with_reporting_token(state, "test".into()));
    let mut input = json!({"retailer_code":"ankerau","title":"Charger","price_amount":100,"currency":"AUD","url":"https://shop.example/scrape", "scraped_at":"2026-10-26T14:00:00.123456+02:00"});
    for (time, events) in [
        ("2026-10-26T14:00:00.123456+02:00", 1),
        ("2026-10-26T12:00:01.123456Z", 1),
        ("2026-10-26T12:00:00.123456Z", 0),
    ] {
        input["scraped_at"] = json!(time);
        client
            .post("/api/tmp-import")
            .body_json(&input)
            .send()
            .await
            .assert_status(StatusCode::ACCEPTED);
        run_queue(&client, "inbox", 100, 1).await;
        run_queue(&client, "commands", 100, 1).await;
        run_queue(&client, "events", 100, events).await;
    }
    let row = inbox_test_snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o");
    assert_eq!(row["scraped_at"], "2026-10-26T12:00:01.123456+00:00");
    let count = inbox_test_snapshot(&pool, "SELECT to_jsonb(count(*)) AS data FROM event_store");
    assert_eq!(count, 2);
    let event = inbox_test_snapshot(
        &pool,
        "SELECT payload::jsonb AS data FROM outbox_entries WHERE meta->>'event_type'='OfferScrapedAtUpdated'",
    );
    assert_eq!(event["scraped_at"], "2026-10-26T12:00:01.123456Z");
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn every_event_keeps_its_testimony_revision_through_outbox_replay() {
    let (_schema, pool) = inbox_test_database();
    let (_kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app_with_reporting_token(state, "test".into()));
    let mut input = json!({"retailer_code":"ankerau","title":"Charger","price_amount":100,"currency":"AUD","url":"https://shop.example/revision"});
    for update in [false, true, true] {
        if update {
            input["title"] = json!("New charger");
            input["price_amount"] = json!(200);
            input["images"] = json!(["https://shop.example/a.png", "https://shop.example/b.png"]);
        }
        client
            .post("/api/tmp-import")
            .body_json(&input)
            .send()
            .await
            .assert_status(StatusCode::ACCEPTED);
        run_queue(&client, "inbox", 100, 1).await;
        run_queue(&client, "commands", 100, 1).await;
        let response = client
            .post("/api/events/run")
            .body_json(&json!({"limit":100}))
            .send()
            .await;
        response.assert_status_is_ok();
    }
    let sequences = inbox_test_snapshot(
        &pool,
        "SELECT jsonb_agg(sequence ORDER BY sequence) AS data FROM event_store",
    );
    assert_eq!(sequences, json!([1, 2, 3, 4, 5]));
    for table in ["event_entries", "outbox_entries"] {
        let actual = inbox_test_snapshot(
            &pool,
            &format!(
                "SELECT jsonb_agg((meta->>'sequence')::bigint ORDER BY (meta->>'sequence')::bigint) AS data FROM {table}"
            ),
        );
        assert_eq!(actual, sequences);
        assert_eq!(
            inbox_test_snapshot(
                &pool,
                &format!(
                    "SELECT to_jsonb(count(*)) AS data FROM {table} WHERE payload::jsonb ? 'sequence'"
                )
            ),
            0
        );
    }
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM event_store WHERE sequence <> (metadata->>'sequence')::bigint"
        ),
        0
    );
    let before = inbox_test_snapshot(
        &pool,
        "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM outbox_entries o",
    );
    pool.get()
        .unwrap()
        .batch_execute("UPDATE event_entries SET status=0,attempts=0,scheduled_at=now() WHERE event_type <> 'OfferCreated'")
        .unwrap();
    run_queue(&client, "events", 100, 4).await;
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM outbox_entries o"
        ),
        before
    );
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn localization_and_identifier_imports_preserve_identity_and_legacy_enrichment() {
    let (_schema, pool) = inbox_test_database();
    let (_kernel, state) = start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(app_with_reporting_token(state, "test".into()));
    let mut input = json!({"retailer_code":"ankerau","title":"Charger","price_amount":100,"currency":"AUD","url":"https://shop.example/facts"});
    for step in 0..4 {
        if step == 1 {
            input["gtin"] = json!("0012345678905");
            input["sku"] = json!("A1");
            input["localization"] = json!({"target_market":{"kind":"region","label":"European Union"},"language":"en-AU"});
        }
        if step == 2 {
            input.as_object_mut().unwrap().remove("gtin");
            input.as_object_mut().unwrap().remove("sku");
            input.as_object_mut().unwrap().remove("localization");
        }
        if step == 3 {
            input["sku"] = json!("A2");
            input["localization"] = json!({"target_market":null,"language":null});
        }
        client
            .post("/api/tmp-import")
            .body_json(&input)
            .send()
            .await
            .assert_status(StatusCode::ACCEPTED);
        run_queue(&client, "inbox", 100, 1).await;
        run_queue(&client, "commands", 100, 1).await;
        run_queue(&client, "events", 100, [1, 3, 0, 3][step]).await;
    }
    let row = inbox_test_snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o");
    assert_eq!(row["gtin"], "0012345678905");
    assert_eq!(row["sku"], "A2");
    assert!(row["language"].is_null());
    assert_eq!(
        row["content"],
        json!([
            {"language":null,"title":"Charger","description":null},
            {"language":"en-AU","title":"Charger","description":null}
        ])
    );
    assert_eq!(
        inbox_test_snapshot(&pool, "SELECT to_jsonb(count(*)) AS data FROM read_offers"),
        1
    );
    assert_eq!(
        inbox_test_snapshot(
            &pool,
            "SELECT jsonb_agg((meta->>'sequence')::bigint ORDER BY (meta->>'sequence')::bigint) AS data FROM outbox_entries"
        ),
        json!([1, 2, 3, 4, 5, 6, 7])
    );
}
