use bodul_shared::retailer::RetailerCode;
use kernel::ApplicationCommand;
use money::Money;
use serde::{Deserialize, Serialize};

pub enum ProductOfferCommand {
    ImportOfferDetails(ImportOfferDetailsCommand),
}

#[derive(Serialize, Deserialize)]
pub struct ImportOfferDetailsCommand {
    #[serde(default)]
    pub images: Vec<String>,
    pub retailer_code: RetailerCode,
    pub title: String,
    pub price: Money,
    pub url: String,
}

pub const CREATE_FROM_OFFER_DETAILS_COMMAND: &str = "ImportOfferDetails";

impl ApplicationCommand for ImportOfferDetailsCommand {
    fn command_type(&self) -> &'static str {
        CREATE_FROM_OFFER_DETAILS_COMMAND
    }
}

#[cfg(test)]
mod export_contract_tests {
    use super::*;

    #[test]
    fn accepts_the_sourcing_export_contract_fixture() {
        let fixture = include_str!(
            "../../../retailer_sourcing/tests/fixtures/offer_export/import_payload.json"
        );
        let command: ImportOfferDetailsCommand = serde_json::from_str(fixture).unwrap();
        assert_eq!(command.command_type(), "ImportOfferDetails");
        assert_eq!(command.retailer_code, RetailerCode::AnkerAu);
        assert_eq!(command.title, " USB charger ž ");
        assert_eq!(command.price, Money::new(1999, money::Currency::AUD));
        assert_eq!(
            command.url,
            "https://shop.example/Charger?q=red&variant=1#details"
        );
    }

    #[test]
    fn legacy_and_v2_imports_have_strict_array_types() {
        let mut input: serde_json::Value = serde_json::from_str(include_str!(
            "../../../retailer_sourcing/tests/fixtures/offer_export/import_payload.json"
        ))
        .unwrap();
        assert!(
            serde_json::from_value::<ImportOfferDetailsCommand>(input.clone())
                .unwrap()
                .images
                .is_empty()
        );
        input["images"] = serde_json::json!(["https://shop.example/a", "/invalid"]);
        assert_eq!(
            serde_json::from_value::<ImportOfferDetailsCommand>(input.clone())
                .unwrap()
                .images
                .len(),
            2
        );
        for value in [
            serde_json::json!(null),
            serde_json::json!("url"),
            serde_json::json!([3]),
        ] {
            input["images"] = value;
            assert!(serde_json::from_value::<ImportOfferDetailsCommand>(input.clone()).is_err());
        }
    }

    #[test]
    fn accepts_sourcing_v2_fixture() {
        let command: ImportOfferDetailsCommand = serde_json::from_str(include_str!(
            "../../../retailer_sourcing/tests/fixtures/offer_export/import_payload_v2.json"
        ))
        .unwrap();
        assert_eq!(command.images, ["https://example.com/a.png"]);
    }
}
