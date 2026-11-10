//! Records an offer for every product URL of a consolidated sitemap.

pub mod prelude;

mod command;
mod entity;
mod event;
mod handler;
#[cfg(test)]
mod postgres_tests;
mod reacts_to;
mod store;

#[cfg(test)]
mod serialization_tests {
    use super::{command::DiscoverOffers, event::OfferDiscovered};
    use crate::platform::bus::{Command, Event};

    #[test]
    fn historical_command_and_event_write_lowercase_retailers() {
        for (historical, slug) in [
            ("AnkerAu", "ankerau"),
            ("MiCom", "micom"),
            ("MinisForumEu", "minisforumeu"),
            ("AnkerNordicsCom", "ankernordicscom"),
        ] {
            let command_json = serde_json::json!({
                "consolidated_sitemap_id": "00000000-0000-0000-0000-000000000001",
                "retailer_code": historical
            });
            let command: DiscoverOffers = serde_json::from_value(command_json.clone()).unwrap();
            let mut canonical = command_json;
            canonical["retailer_code"] = serde_json::json!(slug);
            assert_eq!(serde_json::to_value(&command).unwrap(), canonical);
            assert_eq!(
                serde_json::from_value::<DiscoverOffers>(canonical).unwrap(),
                command
            );

            let event_json = serde_json::json!({
                "offer_id": "00000000-0000-0000-0000-000000000002",
                "consolidated_sitemap_id": "00000000-0000-0000-0000-000000000001",
                "retailer_code": historical,
                "url": "https://example.com/product"
            });
            let event: OfferDiscovered = serde_json::from_value(event_json.clone()).unwrap();
            let mut canonical = event_json;
            canonical["retailer_code"] = serde_json::json!(slug);
            assert_eq!(serde_json::to_value(&event).unwrap(), canonical);
            assert_eq!(
                serde_json::from_value::<OfferDiscovered>(canonical).unwrap(),
                event
            );
        }
        assert_eq!(DiscoverOffers::NAME, "DiscoverOffers");
        assert_eq!(OfferDiscovered::NAME, "OfferDiscovered");
    }
}
