use crate::offers::aggregate::RetailerOfferAggregate;
use crate::offers::command::ImportOfferDetailsCommand;
use crate::offers::event::RetailerOfferEvent;
use crate::offers::retailer_offer_id::RetailerOfferId;
use crate::read_offers_identifiers::entity::RetailerOfferIdentifiers;
use bodul_shared::prelude::RetailerCode;
use money::Money;
use testimony::prelude::{
    EventPublisher, EventSourcedRepository, EventStore, NoOpEventPublisher, RepositoryError,
};
use uuid::Uuid;

pub struct ProcessOfferHandler<
    S: EventStore<RetailerOfferId, RetailerOfferEvent>,
    P: EventPublisher<RetailerOfferEvent>,
    R: RetailerOfferIdentifiersRepository,
> {
    repository: RetailerOfferRepository<S, P>,
    identifiers_repository: R,
}

impl<S, P, R> ProcessOfferHandler<S, P, R>
where
    S: EventStore<RetailerOfferId, RetailerOfferEvent>,
    P: EventPublisher<RetailerOfferEvent>,
    R: RetailerOfferIdentifiersRepository,
{
    pub fn new(repository: RetailerOfferRepository<S, P>, identifiers_repository: R) -> Self {
        Self {
            repository,
            identifiers_repository,
        }
    }

    pub fn create_offer(
        &self,
        command: ImportOfferDetailsCommand,
    ) -> Result<Vec<RetailerOfferEvent>, RepositoryError> {
        let exists = self
            .identifiers_repository
            .find(command.retailer_code, &command.url)
            .map_err(|error| testimony::prelude::EventStoreError::LoadFailed(error.to_string()))?;

        if let Some(read_row) = exists {
            let id = read_row.id;

            self.update_offer(
                id,
                //command.retailer_code,
                command.title,
                command.price,
                //command.url,
            )
        } else {
            let xxx = self.create_offer_event(
                command.retailer_code,
                command.title,
                command.price,
                command.url,
            )?;

            Ok(xxx)
        }
    }

    fn create_offer_event(
        &self,
        retailer_code: RetailerCode,
        title: String,
        price: Money,
        url: String,
    ) -> Result<Vec<RetailerOfferEvent>, RepositoryError> {
        let id = RetailerOfferId::new(Uuid::now_v7().to_string());

        let mut aggregate = RetailerOfferAggregate::create(
            id,
            retailer_code,
            title.clone(),
            price.clone(),
            url.clone(),
        );

        let events = aggregate.get_just_events().clone();

        self.repository.save(&mut aggregate)?;

        Ok(events)
    }

    fn update_offer(
        &self,
        id: RetailerOfferId,
        // retailer_code: RetailerCode,
        title: String,
        price: Money,
        // url: String,
    ) -> Result<Vec<RetailerOfferEvent>, RepositoryError> {
        let mut aggregate = self
            .repository
            .load(id.clone())
            // .map_err(|e| UpdateChannelsError::RepositoryError(e.to_string()))?
            // .ok_or_else(|| UpdateChannelsError::RetailerOfferNotFound(id.to_string()))?;
            .unwrap()
            .unwrap();

        let _ = aggregate.update(title, price);

        let events = aggregate.get_just_events().clone();

        self.repository
            .save(&mut aggregate)
            //.map_err(|e| UpdateChannelsError::RepositoryError(e.to_string()))?;
            .unwrap();

        Ok(events)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateChannelsError {
    /// The retailer could not be loaded from the repository
    #[error("retailer offer not found: {0}")]
    RetailerOfferNotFound(String),
    /// Repository error occurred
    #[error("repository error: {0}")]
    RepositoryError(String),
}

// Mulac publishes the returned events, so this adapter uses Testimony's no-op
// publisher to avoid publishing the same event through both systems.
impl<S, R> kernel::CommandHandlerPort<ImportOfferDetailsCommand, RetailerOfferEvent>
    for ProcessOfferHandler<S, NoOpEventPublisher, R>
where
    S: EventStore<RetailerOfferId, RetailerOfferEvent> + Send + Sync,
    R: RetailerOfferIdentifiersRepository + Send + Sync,
{
    fn execute(
        &self,
        command: ImportOfferDetailsCommand,
    ) -> Result<Vec<RetailerOfferEvent>, kernel::CommandError> {
        self.create_offer(command)
            .map_err(|error| kernel::CommandError::HandlerExecution(error.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("offer identifiers repository failed: {0}")]
pub struct RetailerOfferIdentifiersError(pub String);

pub trait RetailerOfferIdentifiersRepository {
    fn insert(
        &self,
        id: &RetailerOfferId,
        retailer_code: bodul_shared::retailer::RetailerCode,
        url: &str,
    ) -> Result<(), RetailerOfferIdentifiersError>;

    fn find(
        &self,
        retailer_code: bodul_shared::retailer::RetailerCode,
        url: &str,
    ) -> Result<Option<RetailerOfferIdentifiers>, RetailerOfferIdentifiersError>;
}

pub type RetailerOfferRepository<S, P = NoOpEventPublisher> =
    EventSourcedRepository<RetailerOfferAggregate, S, P>;
