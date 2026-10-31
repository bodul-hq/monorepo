use diesel::{RunQueryDsl, connection::SimpleConnection};
use diesel_migrations::MigrationHarness;
use poem::test::TestClient;
use retailer_sauce::offer_content;
use serde_json::json;
use uuid::Uuid;

use super::{inbox_test_database, inbox_test_snapshot as snapshot, run_queue};
use crate::{
    bootstrap::MIGRATIONS,
    offers::retailer_offer_id::RetailerOfferId,
    read_offer::repository::{PostgresRetailerOfferRepository, RetailerOfferRepository},
};

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn localized_migration_preserves_exact_values_and_pending_work_through_round_trip() {
    let (_schema, pool) = inbox_test_database();
    let mut connection = pool.get().unwrap();
    while connection
        .revert_last_migration(MIGRATIONS)
        .unwrap()
        .to_string()
        != "20261027000005"
    {}
    for (index, language, description) in [
        (1, None, None),
        (2, Some("EN-au"), Some("")),
        (3, Some("fr"), Some(" \t<p>ž &amp;</p>\n")),
    ] {
        diesel::sql_query("INSERT INTO read_offers(id,retailer_code,title,price_amount,currency,url,created_at,language,description) VALUES ($1,'ankerau',' Original charger ',10,'AUD',$2,now(),$3,$4)")
            .bind::<diesel::sql_types::Uuid,_>(Uuid::from_u128(index))
            .bind::<diesel::sql_types::Text,_>(format!("https://example.com/{index}"))
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>,_>(language)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>,_>(description)
            .execute(&mut connection).unwrap();
    }
    connection.batch_execute("INSERT INTO command_entries(id,command_type,payload) VALUES ('00000000-0000-0000-0000-000000000009','LegacyPending','{}'); INSERT INTO read_offer_images(offer_id,position,image_url) VALUES ('00000000-0000-0000-0000-000000000001',0,'https://example.com/image.png')").unwrap();
    let rows = || {
        snapshot(
            &pool,
            "SELECT jsonb_agg(to_jsonb(o) ORDER BY id) AS data FROM read_offers o",
        )
    };
    let neighbors = || {
        snapshot(
            &pool,
            "SELECT jsonb_build_object('commands',(SELECT jsonb_agg(to_jsonb(c)) FROM command_entries c),'images',(SELECT jsonb_agg(to_jsonb(i)) FROM read_offer_images i)) AS data",
        )
    };
    let before = rows();
    let auxiliary = neighbors();
    connection.run_pending_migrations(MIGRATIONS).unwrap();
    let migrated = rows();
    for (old, new) in before
        .as_array()
        .unwrap()
        .iter()
        .zip(migrated.as_array().unwrap())
    {
        assert_eq!(
            new["content"],
            json!([{"language":old["language"],"title":old["title"],"description":old["description"]}])
        );
        let mut scalars = new.clone();
        scalars.as_object_mut().unwrap().remove("content");
        assert_eq!(&scalars, old);
    }
    for sql in [
        "UPDATE read_offers SET content=NULL",
        "UPDATE read_offers SET content='null'",
        "UPDATE read_offers SET content='{}'",
        "UPDATE read_offers SET content='[]'",
    ] {
        assert!(connection.batch_execute(sql).is_err());
    }
    while connection
        .revert_last_migration(MIGRATIONS)
        .unwrap()
        .to_string()
        != "20261027000005"
    {}
    assert_eq!(rows(), before);
    connection.run_pending_migrations(MIGRATIONS).unwrap();
    assert_eq!(rows(), migrated);
    assert_eq!(neighbors(), auxiliary);
}

#[tokio::test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
async fn imports_merge_languages_and_publish_selection_only_snapshots() {
    let (_schema, pool) = inbox_test_database();
    let (kernel, state) = crate::start_mulac(pool.clone()).unwrap();
    let client = TestClient::new(crate::app_with_reporting_token(state, "test".into()));
    let mut input = json!({"retailer_code":"ankerau","title":"English charger","price_amount":100,"currency":"AUD","url":"https://shop.example/localized","localization":{"language":"en","target_market":null},"content":{"description":" <p>English ž &amp;</p> "}});
    for step in 0..5 {
        match step {
            1 => {
                input["title"] = json!("French charger");
                input["localization"]["language"] = json!("fr");
                input.as_object_mut().unwrap().remove("content");
            }
            2 => {
                input["title"] = json!("English charger");
                input["localization"]["language"] = json!("en");
            }
            3 => {
                input.as_object_mut().unwrap().remove("localization");
            }
            4 => {
                input["content"] = json!({"description":null});
            }
            _ => {}
        }
        client
            .post("/api/tmp-import")
            .body_json(&input)
            .send()
            .await
            .assert_status(poem::http::StatusCode::ACCEPTED);
        run_queue(&client, "inbox", 100, 1).await;
        run_queue(&client, "commands", 100, 1).await;
        run_queue(&client, "events", 100, [1, 1, 1, 0, 1][step]).await;
        let row = snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o");
        if step == 1 {
            assert!(row["description"].is_null());
            assert_eq!(row["language"], "fr");
        }
        if step == 2 || step == 3 {
            assert_eq!(row["description"], " <p>English ž &amp;</p> ");
        }
    }
    let list = json!([
        {"language":"en","title":"English charger","description":null},
        {"language":"fr","title":"French charger","description":null}
    ]);
    let before = snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o");
    assert_eq!(before["content"], list);
    assert_eq!(before["language"], "en");
    assert_eq!(before["title"], "English charger");
    assert!(before["description"].is_null());
    let outbox = snapshot(
        &pool,
        "SELECT jsonb_agg(jsonb_build_object('type',meta->>'event_type','sequence',(meta->>'sequence')::bigint,'payload',payload::jsonb) ORDER BY (meta->>'sequence')::bigint) AS data FROM outbox_entries",
    );
    assert_eq!(outbox.as_array().unwrap().len(), 4);
    assert_eq!(outbox[0]["type"], "OfferCreated");
    assert_eq!(outbox[0]["payload"]["content"].as_array().unwrap().len(), 1);
    for index in 1..4 {
        assert_eq!(outbox[index]["type"], "OfferContentsChanged");
        assert_eq!(outbox[index]["sequence"], index + 1);
    }
    assert_eq!(
        outbox[1]["payload"]["content"],
        outbox[2]["payload"]["content"]
    );
    assert_eq!(outbox[1]["payload"]["language"], "fr");
    assert_eq!(outbox[2]["payload"]["language"], "en");
    // Replaying the most recent snapshot is idempotent and cannot duplicate outbox rows.
    pool.get().unwrap().batch_execute("UPDATE event_entries SET status=0,attempts=0,scheduled_at=now() WHERE (meta->>'sequence')::bigint=4").unwrap();
    run_queue(&client, "events", 100, 1).await;
    assert_eq!(
        snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o"),
        before
    );
    assert_eq!(
        snapshot(
            &pool,
            "SELECT to_jsonb(count(*)) AS data FROM outbox_entries"
        ),
        4
    );
    kernel.shutdown();
}

#[test]
#[ignore = "requires DATABASE_URL_TEST with CREATE SCHEMA permission"]
fn full_and_legacy_projection_updates_keep_content_and_selected_scalars_atomic() {
    let (_schema, pool) = inbox_test_database();
    pool.get().unwrap().batch_execute("INSERT INTO read_offers(id,retailer_code,title,price_amount,currency,url,created_at,language,description,content) VALUES ('00000000-0000-0000-0000-000000000001','ankerau','English',10,'AUD','https://example.com/atomic',now(),'en','old','[{\"language\":\"en\",\"title\":\"English\",\"description\":\"old\"},{\"language\":\"fr\",\"title\":\"French\",\"description\":null}]')").unwrap();
    let repository = PostgresRetailerOfferRepository::new(pool.clone());
    let id = RetailerOfferId::new(Uuid::from_u128(1).to_string());
    let row = || snapshot(&pool, "SELECT to_jsonb(o) AS data FROM read_offers o");
    let before = row();
    let content: Vec<offer_content::Content> =
        serde_json::from_value(before["content"].clone()).unwrap();
    pool.get()
        .unwrap()
        .batch_execute(
            "ALTER TABLE read_offers ADD CONSTRAINT force_content_rollback CHECK (language='en')",
        )
        .unwrap();
    assert!(
        repository
            .update_contents(&id, &content, &Some("fr".into()))
            .is_err()
    );
    assert_eq!(row(), before);
    pool.get().unwrap().batch_execute("ALTER TABLE read_offers DROP CONSTRAINT force_content_rollback; ALTER TABLE read_offers ADD CONSTRAINT force_market_rollback CHECK (target_market IS NULL)").unwrap();
    let localization = retailer_sauce::localization::Localization {
        language: Some("fr".into()),
        target_market: Some(serde_json::from_value(json!({"kind":"country","code":"FR"})).unwrap()),
    };
    assert!(repository.update_localization(&id, &localization).is_err());
    assert_eq!(row(), before);
    pool.get()
        .unwrap()
        .batch_execute("ALTER TABLE read_offers DROP CONSTRAINT force_market_rollback")
        .unwrap();
    repository
        .update_contents(&id, &content, &Some("fr".into()))
        .unwrap();
    assert_eq!(row()["title"], "French");
    assert!(row()["description"].is_null());
    repository.update_title(&id, "Latest French").unwrap();
    let description = retailer_sauce::structuralized::content::Content {
        description: Some(" \tž &amp;\n".into()),
    };
    repository.update_content(&id, &description).unwrap();
    let localization = retailer_sauce::localization::Localization {
        language: Some("en".into()),
        target_market: None,
    };
    repository.update_localization(&id, &localization).unwrap();
    let after = row();
    assert_eq!(
        after["content"],
        json!([{"language":"en","title":"Latest French","description":" \tž &amp;\n"}])
    );
    assert_eq!(after["title"], "Latest French");
    assert_eq!(after["description"], " \tž &amp;\n");
    for invalid in [
        json!([]),
        json!([{"language":null,"title":"x","description":null}]),
    ] {
        let invalid: Vec<offer_content::Content> = serde_json::from_value(invalid).unwrap();
        assert!(
            repository
                .update_contents(&id, &invalid, &Some("en".into()))
                .is_err()
        );
        assert_eq!(row(), after);
    }
}
