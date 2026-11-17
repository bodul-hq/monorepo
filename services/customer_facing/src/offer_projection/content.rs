//! Content reconciliation stays inside the inbox receipt transaction.
use crate::stored_content::StoredContent;
use diesel::QueryResult;
use diesel::result::Error;
use diesel::sql_query;
use offer_content::singleton;
use offer_content::sort;
use retailer_sauce::prelude::Localization;
use serde_json::from_value;
use serde_json::to_value;
use std::io::Error as StdIoError;

use diesel::{
    PgConnection, RunQueryDsl,
    sql_types::{Jsonb, Uuid as SqlUuid},
};
use retailer_sauce::prelude::{
    OfferContent as Content, StructuralizedContent as LegacyContent, offer_content,
};
use serde_json::Value;
use uuid::Uuid;

pub(super) fn normalize_creation(payload: &mut Value) -> Result<(), String> {
    if payload.get("content").is_some_and(Value::is_array) {
        let mut list: Vec<Content> =
            from_value(payload["content"].clone()).map_err(|cause| cause.to_string())?;
        sort(&mut list);
        payload["content"] = to_value(list).expect("serializable content");
        return Ok(());
    }
    let description = match payload.get("content") {
        None => None,
        Some(value) if value.is_object() => {
            from_value::<LegacyContent>(value.clone())
                .map_err(|error| error.to_string())?
                .description
        }
        Some(_) => return Err("content must be an object or array".into()),
    };
    let title = payload
        .get("title")
        .and_then(Value::as_str)
        .ok_or("title is required")?
        .to_owned();
    let localization: Localization = match payload.get("localization") {
        None => Default::default(),
        Some(value) => from_value(value.clone()).map_err(|error| error.to_string())?,
    };
    payload["content"] = to_value(singleton(title, localization.language, description))
        .expect("serializable content");
    Ok(())
}

pub(super) struct Current {
    pub content: Vec<Content>,
    pub language: Option<String>,
}

#[derive(diesel::QueryableByName)]
struct Record {
    #[diesel(sql_type = Jsonb)]
    content: Value,
}

pub(super) fn lock(connection: &mut PgConnection, id: Uuid) -> QueryResult<Current> {
    let row = sql_query("SELECT content FROM offers WHERE id=$1 FOR UPDATE")
        .bind::<SqlUuid, _>(id)
        .get_result::<Record>(connection)?;
    let stored: StoredContent =
        from_value(row.content).map_err(|cause| error(cause.to_string()))?;
    stored.guard().map_err(error)?;
    Ok(Current {
        content: stored.contents(),
        language: stored.selected_language,
    })
}

pub(super) fn write(
    connection: &mut PgConnection,
    id: Uuid,
    content: &[Content],
    language: &Option<String>,
) -> QueryResult<usize> {
    write_inner(connection, id, content, language, false)
}

pub(super) fn write_language(
    connection: &mut PgConnection,
    id: Uuid,
    content: &[Content],
    language: &Option<String>,
) -> QueryResult<usize> {
    write_inner(connection, id, content, language, true)
}
fn write_inner(
    connection: &mut PgConnection,
    id: Uuid,
    content: &[Content],
    language: &Option<String>,
    renamed: bool,
) -> QueryResult<usize> {
    let row = sql_query("SELECT content FROM offers WHERE id=$1 FOR UPDATE")
        .bind::<SqlUuid, _>(id)
        .get_result::<Record>(connection)?;
    let stored: StoredContent = from_value(row.content).map_err(|e| error(e.to_string()))?;
    let mut list = content.to_vec();
    sort(&mut list);
    let stored = stored
        .reconcile(
            &list,
            language,
            renamed && stored.selected_language != *language,
        )
        .map_err(error)?;
    sql_query("UPDATE offers SET content=$2 WHERE id=$1")
        .bind::<SqlUuid, _>(id)
        .bind::<Jsonb, _>(to_value(stored).expect("serializable content"))
        .execute(connection)
}

pub(super) fn error(message: String) -> Error {
    Error::DeserializationError(StdIoError::other(message).into())
}

#[cfg(test)]
mod tests {
    use offer_content::singleton as offer_content_singleton;

    use super::super::Change;
    use super::*;
    use serde_json::json;

    #[test]
    fn snapshot_requires_explicit_selection_and_complete_unique_entries() {
        let id = Uuid::now_v7();
        let entry = json!({"language":null,"title":"Unknown ž","description":" \t<p>&amp;</p>\n"});
        let valid = json!({"id":id,"content":[entry],"language":null});
        assert!(Change::parse("OfferContentsChanged", &valid.to_string()).is_ok());
        for bad in [
            json!({"id":id,"content":[entry]}),
            json!({"id":id,"content":[entry],"language":"fr"}),
            json!({"id":id,"content":[],"language":null}),
            json!({"id":id,"content":[entry,entry],"language":null}),
            json!({"id":id,"content":[{"title":"Title","description":null}],"language":null}),
            json!({"id":id,"content":[{"language":null,"title":"Title"}],"language":null}),
            json!({"id":id,"content":[{"language":null,"title":" ","description":null}],"language":null}),
            json!({"id":id,"content":{"description":null},"language":null}),
        ] {
            assert!(
                Change::parse("OfferContentsChanged", &bad.to_string()).is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn creation_adapts_legacy_content_and_sorts_new_lists_without_reinterpreting_language() {
        let mut value = json!({"id":Uuid::now_v7(),"retailer_code":"AnkerAu","title":"Selected","price":{"amount":10,"currency":"AUD"},"url":"https://example.com","created_at":"2026-10-27T00:00:00Z","localization":{"language":"EN-au"}});
        for description in [None, Some(""), Some(" \t<p>ž &amp;</p>\n")] {
            value["content"] = json!({"description":description});
            let Change::Created(offer) = Change::parse("OfferCreated", &value.to_string()).unwrap()
            else {
                panic!()
            };
            assert_eq!(
                offer.content,
                offer_content_singleton(
                    "Selected".into(),
                    Some("EN-au".into()),
                    description.map(str::to_owned)
                )
            );
        }
        value["content"] = json!([
            {"language":"fr","title":"French","description":null},
            {"language":"EN-au","title":"Selected","description":null},
            {"language":null,"title":"Unknown","description":null}
        ]);
        let Change::Created(offer) = Change::parse("OfferCreated", &value.to_string()).unwrap()
        else {
            panic!()
        };
        assert_eq!(
            offer
                .content
                .iter()
                .map(|item| item.language.as_deref())
                .collect::<Vec<_>>(),
            [None, Some("EN-au"), Some("fr")]
        );
        value["title"] = json!("Does not match");
        assert!(Change::parse("OfferCreated", &value.to_string()).is_err());
    }
}
