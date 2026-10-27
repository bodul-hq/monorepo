use inbox_api::io as inbox;
use mulac_poem::io::{InboxAdapter, InboxSubmission, SubmitInboxRequest};
use poem::{Error, Result, http::StatusCode};
use poem_openapi::{OpenApi, payload::Json};

pub(crate) use self::model::ImportOfferRequest;

const IMPORT_SOURCE: &str = "offers-api";

mod model {
    use poem_openapi::Object;

    #[derive(Object)]
    pub struct ImportOfferRequest {
        pub was_price:
            poem_openapi::types::MaybeUndefined<poem_openapi::types::Any<serde_json::Value>>,
        pub availability: poem_openapi::types::MaybeUndefined<
            poem_openapi::types::Any<retailer_sauce::availability::Availability>,
        >,
        pub item_condition: poem_openapi::types::MaybeUndefined<
            poem_openapi::types::Any<retailer_sauce::item_condition::ItemCondition>,
        >,
        pub content: poem_openapi::types::MaybeUndefined<
            poem_openapi::types::Any<retailer_sauce::structuralized::content::Content>,
        >,
        pub gtin: Option<String>,
        pub sku: Option<String>,
        pub external_identifiers: poem_openapi::types::MaybeUndefined<
            poem_openapi::types::Any<retailer_sauce::identifier::ExternalIdentifiers>,
        >,
        pub localization: poem_openapi::types::MaybeUndefined<
            poem_openapi::types::Any<retailer_sauce::localization::Localization>,
        >,
        pub scraped_at: Option<String>,
        #[oai(default)]
        pub images: Vec<String>,
        pub retailer_code: String,
        pub title: String,
        /// Price in minor currency units (cents for currencies with two decimal places).
        pub price_amount: i64,
        pub currency: String,
        pub url: String,
    }
}

mod conversion {
    use bodul_shared::retailer::RetailerCode;
    use money::Currency;
    use serde_json::json;

    use super::{IMPORT_SOURCE, ImportOfferRequest, inbox};
    use crate::offers::command::CREATE_FROM_OFFER_DETAILS_COMMAND;

    fn err_null_identifiers() -> String {
        "external_identifiers must be an object".into()
    }

    fn err_null_localization() -> String {
        "localization must be an object".into()
    }

    fn err_invalid_was_price(error: impl std::fmt::Display) -> String {
        format!("invalid was_price: {error}")
    }

    fn err_null_content() -> String {
        "content must be an object".into()
    }

    fn err_invalid_scraped_at(error: chrono::ParseError) -> String {
        format!("invalid scraped_at: {error}")
    }

    impl TryFrom<ImportOfferRequest> for inbox::SubmitInboxRequest {
        type Error = String;

        fn try_from(request: ImportOfferRequest) -> Result<Self, Self::Error> {
            let scraped_at = request
                .scraped_at
                .as_deref()
                .map(str::parse::<chrono::DateTime<chrono::Utc>>)
                .transpose()
                .map_err(err_invalid_scraped_at)?;
            let retailer_code = RetailerCode::from_str(&request.retailer_code)?;
            let currency: Currency = request.currency.parse()?;

            let mut payload = json!({
                "retailer_code": retailer_code,
                "title": request.title,
                "price": { "amount": request.price_amount, "currency": currency },
                "url": request.url,
                "images": request.images,
            });

            use poem_openapi::types::MaybeUndefined;
            match request.was_price {
                MaybeUndefined::Undefined => {}
                MaybeUndefined::Null => payload["was_price"] = serde_json::Value::Null,
                MaybeUndefined::Value(value) => {
                    let was_price: money::Money = retailer_sauce::observation::object(value.0)
                        .map_err(err_invalid_was_price)?;
                    crate::offers::event::guard_was_price(
                        &money::Money::new(request.price_amount, currency),
                        Some(&was_price),
                    )?;
                    payload["was_price"] = json!(was_price);
                }
            }
            match request.availability {
                MaybeUndefined::Undefined => {}
                MaybeUndefined::Null => payload["availability"] = serde_json::Value::Null,
                MaybeUndefined::Value(value) => {
                    value.0.guard()?;
                    payload["availability"] = json!(value.0);
                }
            }
            match request.item_condition {
                MaybeUndefined::Undefined => {}
                MaybeUndefined::Null => payload["item_condition"] = serde_json::Value::Null,
                MaybeUndefined::Value(value) => {
                    value.0.guard()?;
                    payload["item_condition"] = json!(value.0);
                }
            }
            match request.content {
                MaybeUndefined::Undefined => {}
                MaybeUndefined::Null => return Err(err_null_content()),
                MaybeUndefined::Value(value) => {
                    value.0.guard()?;
                    payload["content"] = json!(value.0);
                }
            }
            if let Some(scraped_at) = scraped_at {
                payload["scraped_at"] = json!(scraped_at);
            }
            match request.localization {
                poem_openapi::types::MaybeUndefined::Undefined => {}
                poem_openapi::types::MaybeUndefined::Null => return Err(err_null_localization()),
                poem_openapi::types::MaybeUndefined::Value(value) => {
                    payload["localization"] = json!(value.0);
                }
            }
            let external_identifiers = match request.external_identifiers {
                poem_openapi::types::MaybeUndefined::Undefined => Default::default(),
                poem_openapi::types::MaybeUndefined::Null => return Err(err_null_identifiers()),
                poem_openapi::types::MaybeUndefined::Value(value) => value.0,
            };
            let identifiers = retailer_sauce::identifier::ProductIdentifiers {
                gtin: request
                    .gtin
                    .map(|value| retailer_sauce::identifier::Gtin { value }),
                sku: request
                    .sku
                    .map(|value| retailer_sauce::identifier::Sku { value }),
                external_identifiers,
            };
            identifiers.guard()?;
            // Keep legacy form payloads unchanged unless identifiers were provided.
            if identifiers != Default::default() {
                payload["gtin"] = json!(identifiers.gtin);
                payload["sku"] = json!(identifiers.sku);
                payload["external_identifiers"] = json!(identifiers.external_identifiers);
            }
            let submission = Self {
                payload: Some(payload),
                routing_key: CREATE_FROM_OFFER_DETAILS_COMMAND.to_owned(),
                message_id: None,
                correlation_id: None,
                source: Some(IMPORT_SOURCE.to_owned()),
            };

            Ok(submission)
        }
    }
}

pub struct OffersApi {
    inbox: InboxAdapter,
}

impl OffersApi {
    pub fn new(inbox: InboxAdapter) -> Self {
        Self { inbox }
    }
}

#[OpenApi]
impl OffersApi {
    /// Validate an import and store it in the inbox as an `ImportOfferDetails`
    /// message; `/inbox/run` and `/commands/run` then create the offer.
    #[oai(path = "/tmp-import", method = "post")]
    async fn tmp_import(&self, request: Json<ImportOfferRequest>) -> Result<InboxSubmission> {
        let submission =
            inbox::SubmitInboxRequest::try_from(request.0).map_err(err_invalid_import_request)?;
        self.inbox
            .submit(Json(SubmitInboxRequest(submission)))
            .await
    }
}

fn err_invalid_import_request(message: String) -> Error {
    Error::from_string(message, StatusCode::BAD_REQUEST)
}

#[cfg(test)]
mod tests {
    use bodul_shared::retailer::RetailerCode;
    use money::{Currency, Money};
    use poem_openapi::types::ParseFromJSON;
    use retailer_sauce::observation::Observation;
    use serde_json::json;

    use super::*;
    use crate::offers::command::{CREATE_FROM_OFFER_DETAILS_COMMAND, ImportOfferDetailsCommand};

    fn import_request_json() -> serde_json::Value {
        json!({"retailer_code":"ankerau","title":"Charger","price_amount":1999,"currency":"AUD","url":"https://example.com/charger"})
    }

    fn submitted_payload(input: serde_json::Value) -> serde_json::Value {
        let request = ImportOfferRequest::parse_from_json(Some(input))
            .map_err(|error| error.into_message())
            .unwrap();
        inbox::SubmitInboxRequest::try_from(request)
            .unwrap()
            .payload
            .unwrap()
    }

    #[test]
    fn optional_facts_preserve_missing_null_and_observed_values() {
        let payload = submitted_payload(import_request_json());
        for field in ["was_price", "availability", "item_condition", "content"] {
            assert!(payload.get(field).is_none(), "{field}");
        }
        let command: ImportOfferDetailsCommand = serde_json::from_value(payload).unwrap();
        assert_eq!(command.was_price, Observation::Missing);
        assert_eq!(command.availability, Observation::Missing);
        assert_eq!(command.item_condition, Observation::Missing);
        assert_eq!(command.content, None);

        let mut input = import_request_json();
        for field in ["was_price", "availability", "item_condition"] {
            input[field] = serde_json::Value::Null;
        }
        input["content"] = json!({"description":null});
        let payload = submitted_payload(input);
        for field in ["was_price", "availability", "item_condition"] {
            assert_eq!(payload.get(field), Some(&serde_json::Value::Null));
        }
        let command: ImportOfferDetailsCommand = serde_json::from_value(payload).unwrap();
        assert_eq!(command.was_price, Observation::Observed(None));
        assert_eq!(command.availability, Observation::Observed(None));
        assert_eq!(command.item_condition, Observation::Observed(None));
        assert_eq!(command.content.unwrap().description, None);

        let mut input = import_request_json();
        let facts = json!({"was_price":{"amount":2499,"currency":"AUD"},"availability":{"other":"custom source value"},"item_condition":"new","content":{"description":"<b>Exact HTML ž</b>"}});
        input
            .as_object_mut()
            .unwrap()
            .extend(facts.as_object().unwrap().clone());
        let payload = submitted_payload(input);
        for (field, expected) in facts.as_object().unwrap() {
            assert_eq!(&payload[field], expected);
        }
        let command: ImportOfferDetailsCommand = serde_json::from_value(payload).unwrap();
        assert_eq!(
            command.was_price,
            Observation::Observed(Some(Money::new(2499, Currency::AUD)))
        );
        assert_eq!(
            command.availability,
            Observation::Observed(Some(retailer_sauce::availability::Availability::Other(
                "custom source value".into()
            )))
        );
        assert_eq!(
            command.item_condition,
            Observation::Observed(Some(retailer_sauce::item_condition::ItemCondition::New))
        );
        assert_eq!(
            command.content.unwrap().description.as_deref(),
            Some("<b>Exact HTML ž</b>")
        );
    }

    #[tokio::test]
    async fn invalid_facts_are_rejected_before_inbox_storage() {
        let client = poem::test::TestClient::new(crate::app(crate::test_pool()));
        for (field, value) in [
            ("was_price", json!({"amount":1999,"currency":"AUD"})),
            ("was_price", json!({"amount":2999,"currency":"USD"})),
            ("availability", json!({"other":" "})),
            ("availability", json!("invalid")),
            ("item_condition", json!({"other":"contains\u{0000}nul"})),
            ("was_price", json!([2999, "AUD"])),
            ("content", json!([null])),
            ("content", json!(null)),
            ("content", json!({})),
            ("content", json!({"description":"contains\u{0000}nul"})),
            ("content", json!({"description":null,"extra":true})),
        ] {
            let mut input = import_request_json();
            input[field] = value;
            client
                .post("/api/tmp-import")
                .body_json(&input)
                .send()
                .await
                .assert_status(StatusCode::BAD_REQUEST);
        }
    }

    #[test]
    fn request_converts_directly_to_a_command_compatible_inbox_message() {
        for amount in [0, 1999, i64::MAX] {
            let request = ImportOfferRequest {
                was_price: Default::default(),
                availability: Default::default(),
                item_condition: Default::default(),
                content: Default::default(),
                gtin: None,
                sku: None,
                external_identifiers: Default::default(),
                localization: Default::default(),
                scraped_at: None,
                images: Vec::new(),
                retailer_code: "ankerau".into(),
                title: " USB charger ".into(),
                price_amount: amount,
                currency: "AUD".into(),
                url: "https://example.com/Charger?q=red#details".into(),
            };

            let submission = inbox::SubmitInboxRequest::try_from(request).unwrap();

            assert_eq!(submission.routing_key, CREATE_FROM_OFFER_DETAILS_COMMAND);
            assert_eq!(submission.source.as_deref(), Some(IMPORT_SOURCE));
            assert_eq!(submission.message_id, None);
            assert_eq!(submission.correlation_id, None);
            submission.validate().unwrap();
            let payload = submission.payload.unwrap();
            assert_eq!(
                payload,
                json!({
                    "retailer_code": "AnkerAu",
                    "title": " USB charger ",
                    "price": { "amount": amount, "currency": "AUD" },
                    "url": "https://example.com/Charger?q=red#details",
                    "images": [],
                })
            );

            let decoded: ImportOfferDetailsCommand = serde_json::from_value(payload).unwrap();
            assert_eq!(decoded.retailer_code, RetailerCode::AnkerAu);
            assert_eq!(decoded.title, " USB charger ");
            assert_eq!(decoded.price, Money::new(amount, Currency::AUD));
            assert_eq!(decoded.url, "https://example.com/Charger?q=red#details");
        }
    }
}
