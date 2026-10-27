//! Content reconciliation stays inside the inbox receipt transaction.
use diesel::{
    PgConnection, RunQueryDsl,
    sql_types::{Jsonb, Nullable, Text, Uuid as SqlUuid},
};
use retailer_sauce::{
    offer_content::{self, Content},
    structuralized::content::Content as LegacyContent,
};
use serde_json::Value;
use uuid::Uuid;

pub(super) fn normalize_creation(payload: &mut Value) -> Result<(), String> {
    if payload.get("content").is_some_and(Value::is_array) {
        let mut list: Vec<Content> = serde_json::from_value(payload["content"].clone())
            .map_err(|cause| cause.to_string())?;
        offer_content::sort(&mut list);
        payload["content"] = serde_json::to_value(list).expect("serializable content");
        return Ok(());
    }
    let description = match payload.get("content") {
        None => None,
        Some(value) if value.is_object() => {
            serde_json::from_value::<LegacyContent>(value.clone())
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
    let localization: retailer_sauce::localization::Localization = match payload.get("localization")
    {
        None => Default::default(),
        Some(value) => serde_json::from_value(value.clone()).map_err(|error| error.to_string())?,
    };
    payload["content"] = serde_json::to_value(offer_content::singleton(
        title,
        localization.language,
        description,
    ))
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
    #[diesel(sql_type = Nullable<Text>)]
    language: Option<String>,
}

pub(super) fn lock(connection: &mut PgConnection, id: Uuid) -> diesel::QueryResult<Current> {
    let row = diesel::sql_query("SELECT content, language FROM offers WHERE id=$1 FOR UPDATE")
        .bind::<SqlUuid, _>(id)
        .get_result::<Record>(connection)?;
    let content = serde_json::from_value(row.content).map_err(|cause| error(cause.to_string()))?;
    Ok(Current {
        content,
        language: row.language,
    })
}

pub(super) fn write(
    connection: &mut PgConnection,
    id: Uuid,
    content: &[Content],
    language: &Option<String>,
) -> diesel::QueryResult<usize> {
    offer_content::guard(content, language).map_err(error)?;
    let mut content = content.to_vec();
    offer_content::sort(&mut content);
    let selected = offer_content::selected(&content, language).map_err(error)?;
    diesel::sql_query(
        "UPDATE offers SET content=$2, title=$3, description=$4, language=$5 WHERE id=$1",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Jsonb, _>(serde_json::to_value(&content).expect("serializable content"))
    .bind::<Text, _>(&selected.title)
    .bind::<Nullable<Text>, _>(&selected.description)
    .bind::<Nullable<Text>, _>(language)
    .execute(connection)
}

pub(super) fn error(message: String) -> diesel::result::Error {
    diesel::result::Error::DeserializationError(std::io::Error::other(message).into())
}

#[cfg(test)]
mod tests {
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
                offer_content::singleton(
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
