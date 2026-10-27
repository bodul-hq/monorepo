use crate::offers::aggregate::RetailerOfferAggregate;
use crate::offers::command::ImportOfferDetailsCommand;
use crate::offers::event::{RetailerOfferEvent, SequencedOfferEvent};
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
        self.process_offer(command)
            .map(|events| events.into_iter().map(|event| event.event).collect())
    }

    fn process_offer(
        &self,
        command: ImportOfferDetailsCommand,
    ) -> Result<Vec<SequencedOfferEvent>, RepositoryError> {
        let observation = super::command::OfferObservation {
            scraped_at: command.scraped_at,
            localization: command.localization,
        };
        observation.guard().map_err(err_update)?;
        let exists = self
            .identifiers_repository
            .find(command.retailer_code, &command.url)
            .map_err(err_update)?;

        if let Some(read_row) = exists {
            let id = read_row.id;

            self.update_offer(
                id,
                command.title,
                command.price,
                command.images,
                observation,
            )
        } else {
            self.create_offer_event(
                command.retailer_code,
                command.title,
                command.price,
                command.url,
                command.images,
                observation,
            )
        }
    }

    fn create_offer_event(
        &self,
        retailer_code: RetailerCode,
        title: String,
        price: Money,
        url: String,
        images: Vec<String>,
        observation: super::command::OfferObservation,
    ) -> Result<Vec<SequencedOfferEvent>, RepositoryError> {
        let id = RetailerOfferId::new(Uuid::now_v7().to_string());

        let mut aggregate = RetailerOfferAggregate::create_with_observation(
            id,
            retailer_code,
            title,
            price,
            url,
            images,
            observation,
        )
        .map_err(err_update)?;
        let events = aggregate
            .uncommitted_events
            .iter()
            .map(|envelope| SequencedOfferEvent {
                event: envelope.event().clone(),
                sequence: envelope.sequence(),
            })
            .collect();

        self.repository.save(&mut aggregate)?;

        Ok(events)
    }

    fn update_offer(
        &self,
        id: RetailerOfferId,
        title: String,
        price: Money,
        images: Vec<String>,
        observation: super::command::OfferObservation,
    ) -> Result<Vec<SequencedOfferEvent>, RepositoryError> {
        let mut aggregate = self
            .repository
            .load(id.clone())?
            .ok_or_else(|| err_update(format!("offer {id} not found")))?;
        aggregate
            .update_with_observation(title, price, images, observation)
            .map_err(err_update)?;
        let events = aggregate
            .uncommitted_events
            .iter()
            .map(|envelope| SequencedOfferEvent {
                event: envelope.event().clone(),
                sequence: envelope.sequence(),
            })
            .collect();
        self.repository.save(&mut aggregate)?;

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
impl<S, R> kernel::CommandHandlerPort<ImportOfferDetailsCommand, SequencedOfferEvent>
    for ProcessOfferHandler<S, NoOpEventPublisher, R>
where
    S: EventStore<RetailerOfferId, RetailerOfferEvent> + Send + Sync,
    R: RetailerOfferIdentifiersRepository + Send + Sync,
{
    fn execute(
        &self,
        command: ImportOfferDetailsCommand,
    ) -> Result<Vec<SequencedOfferEvent>, kernel::CommandError> {
        self.process_offer(command)
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

fn err_update(error: impl std::fmt::Display) -> RepositoryError {
    testimony::prelude::EventStoreError::LoadFailed(error.to_string()).into()
}
