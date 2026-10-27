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
    use serde_json::json;

    use super::*;
    use crate::offers::command::{CREATE_FROM_OFFER_DETAILS_COMMAND, ImportOfferDetailsCommand};

    #[test]
    fn request_converts_directly_to_a_command_compatible_inbox_message() {
        for amount in [0, 1999, i64::MAX] {
            let request = ImportOfferRequest {
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
