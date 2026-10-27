use diesel::{
    RunQueryDsl,
    connection::SimpleConnection,
    sql_types::{Nullable, Text, Uuid as SqlUuid},
};
use diesel_migrations::MigrationHarness;
use poem::{http::StatusCode, test::TestClient};
use retailer_sauce::identifier::ProductIdentifiers;
use serde_json::{Value, json};
use testimony_infra::prelude::DbPool;
use uuid::Uuid;

mod queued_work;

use super::{
    entity::RetailerOfferIdentifiers,
    repository::{PostgresRetailerOfferIdentifiersRepository, RetailerOfferIdentifiersRepository},
};
use crate::{
    bootstrap::MIGRATIONS,
    offers::{
        handler::RetailerOfferIdentifiersRepository as Lookup, retailer_offer_id::RetailerOfferId,
    },
    postgres_tests::{inbox_test_database, inbox_test_snapshot as snapshot, run_queue},
};

fn before_identifier_migration(pool: &DbPool) {
    let mut connection = pool.get().unwrap();
    loop {
        let reverted = connection.revert_last_migration(MIGRATIONS).unwrap();
        if reverted.to_string() == "20261027000001" {
            break;
        }
    }
}

fn rows(pool: &DbPool, table: &str) -> Value {
    snapshot(
        pool,
        &format!(
            "SELECT coalesce(jsonb_agg(jsonb_build_object('id',id,'retailer_code',retailer_code,'url',url,'gtin',gtin,'sku',sku,'shopify_product_gid',shopify_product_gid,'shopify_product_variation_gid',shopify_product_variation_gid,'shopify_handle',shopify_handle) ORDER BY id),'[]') AS data FROM {table}"
        ),
    )
}

fn table_snapshot(pool: &DbPool, table: &str) -> Value {
    snapshot(
        pool,
        &format!("SELECT coalesce(jsonb_agg(to_jsonb(r) ORDER BY id),'[]') AS data FROM {table} r"),
    )
}

fn migration_columns(pool: &DbPool) -> Value {
    snapshot(
        pool,
        "SELECT coalesce(jsonb_agg(column_name ORDER BY column_name),'[]') AS data FROM information_schema.columns WHERE table_schema=current_schema() AND table_name='read_offer_identities' AND column_name IN ('gtin','sku','shopify_product_gid','shopify_product_variation_gid','shopify_handle')",
    )
}

fn seed_legacy_rows(pool: &DbPool) {
    for (id, gtin, sku, product, variant, handle) in [
        (
            1,
            Some("0012345678905"),
            Some(" SKU 001 "),
            Some("gid://shopify/Product/001"),
            Some("002"),
            Some(" Ž \\\" "),
        ),
        (2, None, Some("partial"), None, Some("0002"), None),
        (3, None, None, None, None, None),
    ] {
        diesel::sql_query("INSERT INTO read_offers (id,retailer_code,url,title,price_amount,currency,created_at,gtin,sku,shopify_product_gid,shopify_product_variation_gid,shopify_handle) VALUES ($1,'ankerau',$2,'Charger',1999,'AUD',now(),$3,$4,$5,$6,$7)")
            .bind::<SqlUuid,_>(Uuid::from_u128(id)).bind::<Text,_>(format!("https://example.com/{id}"))
            .bind::<Nullable<Text>,_>(gtin).bind::<Nullable<Text>,_>(sku).bind::<Nullable<Text>,_>(product)
            .bind::<Nullable<Text>,_>(variant).bind::<Nullable<Text>,_>(handle)
            .execute(&mut pool.get().unwrap()).unwrap();
    }
    pool.get().unwrap().batch_execute("INSERT INTO read_offer_identities(id,retailer_code,url) SELECT id,retailer_code,url FROM read_offers; INSERT INTO read_offer_identities(id,retailer_code,url) VALUES ('00000000-0000-0000-0000-000000000004','ankerau','https://example.com/unmatched'); INSERT INTO command_entries(id,command_type,payload) VALUES ('00000000-0000-0000-0000-000000000011','LegacyPending','{}')").unwrap();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn backfill_preserves_values_unmatched_identities_and_pending_work_through_round_trip() {
    let (_schema, pool) = inbox_test_database();
    before_identifier_migration(&pool);
    seed_legacy_rows(&pool);
    pool.get().unwrap().batch_execute(
        "INSERT INTO read_offer_identities(id,retailer_code,url) VALUES ('00000000-0000-0000-0000-000000000005','ankerau','https://example.com/different-id'); INSERT INTO read_offers(id,retailer_code,url,title,price_amount,currency,created_at,gtin,sku) VALUES ('00000000-0000-0000-0000-000000000006','ankerau','https://example.com/source-only','Source only',1999,'AUD',now(),'0000000000006','do-not-copy'), ('00000000-0000-0000-0000-000000000007','ankerau','https://example.com/different-id','Different ID',1999,'AUD',now(),'0000000000007','do-not-remap')"
    ).unwrap();
    let originals = rows(&pool, "read_offers");
    let source_rows = table_snapshot(&pool, "read_offers");
    let identity_keys = table_snapshot(&pool, "read_offer_identities");
    let work = table_snapshot(&pool, "command_entries");
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    let populated = rows(&pool, "read_offer_identities");
    assert_eq!(
        &populated.as_array().unwrap()[..3],
        &originals.as_array().unwrap()[..3]
    );
    assert_eq!(populated.as_array().unwrap().len(), 5);
    assert_eq!(rows(&pool, "read_offers"), originals);
    for row in &populated.as_array().unwrap()[3..] {
        for name in [
            "gtin",
            "sku",
            "shopify_product_gid",
            "shopify_product_variation_gid",
            "shopify_handle",
        ] {
            assert!(row[name].is_null());
        }
    }
    // Check the rows counted by the migration; Diesel suppresses PostgreSQL NOTICEs.
    assert_eq!(
        snapshot(
            &pool,
            "SELECT jsonb_agg(i.id ORDER BY i.id) AS data FROM read_offer_identities i WHERE NOT EXISTS (SELECT 1 FROM read_offers o WHERE o.id=i.id)"
        ),
        json!([Uuid::from_u128(4), Uuid::from_u128(5)])
    );
    before_identifier_migration(&pool);
    assert_eq!(table_snapshot(&pool, "read_offers"), source_rows);
    assert_eq!(
        table_snapshot(&pool, "read_offer_identities"),
        identity_keys
    );
    assert_eq!(migration_columns(&pool), json!([]));
    assert_eq!(table_snapshot(&pool, "command_entries"), work);
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    assert_eq!(rows(&pool, "read_offer_identities"), populated);
    assert_eq!(rows(&pool, "read_offers"), originals);
    assert_eq!(table_snapshot(&pool, "command_entries"), work);
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn backfill_rejects_conflicting_identity_keys_and_empty_identifiers_atomically() {
    for mutation in [
        "UPDATE read_offer_identities SET url='https://example.com/different' WHERE id='00000000-0000-0000-0000-000000000001'",
        "UPDATE read_offer_identities SET retailer_code='ankercom' WHERE id='00000000-0000-0000-0000-000000000001'",
        "UPDATE read_offers SET gtin='' WHERE id='00000000-0000-0000-0000-000000000001'",
        "UPDATE read_offers SET sku='' WHERE id='00000000-0000-0000-0000-000000000001'",
        "UPDATE read_offers SET shopify_product_gid='' WHERE id='00000000-0000-0000-0000-000000000001'",
        "UPDATE read_offers SET shopify_product_variation_gid='' WHERE id='00000000-0000-0000-0000-000000000001'",
        "UPDATE read_offers SET shopify_handle='' WHERE id='00000000-0000-0000-0000-000000000001'",
    ] {
        let (_schema, pool) = inbox_test_database();
        before_identifier_migration(&pool);
        seed_legacy_rows(&pool);
        pool.get().unwrap().batch_execute(mutation).unwrap();
        let identities = table_snapshot(&pool, "read_offer_identities");
        let source = table_snapshot(&pool, "read_offers");
        let work = table_snapshot(&pool, "command_entries");
        let migrations = pool.get().unwrap().applied_migrations().unwrap();
        assert!(
            pool.get()
                .unwrap()
                .run_pending_migrations(MIGRATIONS)
                .is_err()
        );
        assert_eq!(
            table_snapshot(&pool, "read_offer_identities"),
            identities,
            "{mutation}"
        );
        assert_eq!(table_snapshot(&pool, "read_offers"), source, "{mutation}");
        assert_eq!(table_snapshot(&pool, "command_entries"), work, "{mutation}");
        assert_eq!(
            pool.get().unwrap().applied_migrations().unwrap(),
            migrations
        );
        assert_eq!(migration_columns(&pool), json!([]));
    }
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn identity_repositories_preserve_exact_values_replace_nulls_and_keep_url_matching() {
    let (_schema, pool) = inbox_test_database();
    let writer = PostgresRetailerOfferIdentifiersRepository::new(pool.clone());
    let lookup =
        crate::infra::identifiers_repository::PostgresRetailerOfferIdentifiersRepository::new(
            pool.clone(),
        );
    let identifiers: ProductIdentifiers = serde_json::from_value(json!({"gtin":"0012345678905","sku":" Sku ","external_identifiers":{"shopify_product_gid":"001","shopify_product_variation_gid":"002","shopify_handle":" Ž \\\" "}})).unwrap();
    let entity = RetailerOfferIdentifiers {
        identifiers: identifiers.clone(),
        id: RetailerOfferId::new(Uuid::new_v4().to_string()),
        retailer_code: bodul_shared::retailer::RetailerCode::AnkerAu,
        url: "https://example.com/Charger?q=Red#details".into(),
    };
    writer.insert(&entity).unwrap();
    assert_eq!(
        lookup.find(entity.retailer_code, &entity.url).unwrap(),
        Some(entity.clone())
    );
    assert!(
        lookup
            .find(entity.retailer_code, &entity.url.to_lowercase())
            .unwrap()
            .is_none()
    );
    let other = RetailerOfferIdentifiers {
        id: RetailerOfferId::new(Uuid::new_v4().to_string()),
        url: "https://example.com/other".into(),
        ..entity.clone()
    };
    writer.insert(&other).unwrap(); // Product identifiers are not unique identity keys.
    let mut invalid = identifiers;
    invalid.gtin.as_mut().unwrap().value.clear();
    assert!(writer.update_identifiers(&entity.id, &invalid).is_err());
    assert_eq!(
        lookup.find(entity.retailer_code, &entity.url).unwrap(),
        Some(entity.clone())
    );
    let replacement: ProductIdentifiers =
        serde_json::from_value(json!({"sku":"replacement"})).unwrap();
    for _ in 0..2 {
        writer.update_identifiers(&entity.id, &replacement).unwrap();
    }
    assert_eq!(
        lookup
            .find(entity.retailer_code, &entity.url)
            .unwrap()
            .unwrap()
            .identifiers,
        replacement
    );
    writer
        .update_identifiers(&entity.id, &ProductIdentifiers::default())
        .unwrap();
    assert_eq!(
        lookup
            .find(entity.retailer_code, &entity.url)
            .unwrap()
            .unwrap()
            .identifiers,
        ProductIdentifiers::default()
    );
    assert!(
        writer
            .update_identifiers(
                &RetailerOfferId::new(Uuid::new_v4().to_string()),
                &replacement
            )
            .is_err()
    );
    assert!(
        writer
            .update_identifiers(&RetailerOfferId::new("invalid"), &replacement)
            .is_err()
    );
    assert_eq!(
        lookup.find(other.retailer_code, &other.url).unwrap(),
        Some(other)
    );
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn registered_identity_projector_tracks_creation_and_identifier_only_imports() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    let mut payload = json!({"retailer_code":"AnkerAu","title":"Charger","price":{"amount":1999,"currency":"AUD"},"url":"https://example.com/Charger","gtin":"0012345678905","sku":"original","external_identifiers":{"shopify_product_gid":"001","shopify_product_variation_gid":"002","shopify_handle":"handle"}});
    for sku in ["original", "updated"] {
        payload["sku"] = json!(sku);
        client.post("/api/inbox").body_json(&json!({"message_id":Uuid::new_v4(),"source":"test","routing_key":"ImportOfferDetails","payload":payload})).send().await.assert_status(StatusCode::ACCEPTED);
        run_queue(&client, "inbox", 100, 1).await;
        run_queue(&client, "commands", 100, 1).await;
        run_queue(&client, "events", 100, 1).await;
        let identities = rows(&pool, "read_offer_identities");
        assert_eq!(identities, rows(&pool, "read_offers"));
        assert_eq!(identities[0]["sku"], sku);
        assert_eq!(identities[0]["gtin"], "0012345678905");
    }
    assert_eq!(
        snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM read_offer_identities"
        ),
        1
    );
    kernel.shutdown();
}
