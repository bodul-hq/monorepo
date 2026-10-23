use testimony::prelude::NoOpEventPublisher;
use testimony_infra::prelude::{DbPool, PostgresEventStore};

use super::records::OfferEventRecord;
use crate::offers::{
    event::RetailerOfferEvent,
    handler::{ProcessOfferHandler, RetailerOfferRepository},
    retailer_offer_id::RetailerOfferId,
};

pub type OfferEventStore =
    PostgresEventStore<RetailerOfferId, RetailerOfferEvent, OfferEventRecord>;
pub type PostgresOfferHandler = ProcessOfferHandler<
    OfferEventStore,
    NoOpEventPublisher,
    PostgresRetailerOfferIdentifiersRepository,
>;

pub use super::identifiers_repository::PostgresRetailerOfferIdentifiersRepository;

pub fn offer_handler(pool: DbPool) -> PostgresOfferHandler {
    ProcessOfferHandler::new(
        RetailerOfferRepository::new(OfferEventStore::new(pool.clone())),
        PostgresRetailerOfferIdentifiersRepository::new(pool),
    )
}
