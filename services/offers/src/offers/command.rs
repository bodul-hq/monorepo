use bodul_shared::retailer::RetailerCode;
use kernel::ApplicationCommand;
use money::Money;
use serde::{Deserialize, Serialize};

pub enum ProductOfferCommand {
    ImportOfferDetails(ImportOfferDetailsCommand),
}

#[derive(Serialize, Deserialize)]
pub struct ImportOfferDetailsCommand {
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
}
