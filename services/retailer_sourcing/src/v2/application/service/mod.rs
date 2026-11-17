//! Retailer sourcing v2 application services: use-case orchestration that
//! coordinates the domain through its ports.

use crate::platform::bus::{DispatchError, Dispatcher, KeyedCommand};
use crate::v2::application::model::{
    FillRun, InitialFillOrder, InitialFillStatus, PreparationFailure, StartFailure,
};
use crate::v2::application::prelude::product_catalog_ids::prelude::{
    FetchProductCatalogIdentifiers, initial,
};
use crate::v2::prelude::AcquireStorefrontToken;
use retailer_data_extractor::get_storefront_config;
use retailer_sauce::prelude::StorefrontConfig;
use shared::retailer::RetailerCode;
use std::collections::HashSet;
use tokio::task::spawn_blocking;
use tracing::error;
use uuid::Uuid;

pub struct StorefrontConfigRepository;

#[derive(Debug, thiserror::Error)]
pub enum StorefrontConfigRepositoryError {
    #[error("no storefront configuration for retailer {0:?}")]
    UnsupportedRetailer(RetailerCode),
}

impl From<StorefrontConfigRepositoryError> for PreparationFailure {
    fn from(error: StorefrontConfigRepositoryError) -> Self {
        match error {
            StorefrontConfigRepositoryError::UnsupportedRetailer(code) => {
                Self::UnsupportedRetailer(code.as_str())
            }
        }
    }
}

impl StorefrontConfigRepository {
    pub fn fetch(
        retailer_codes: &[RetailerCode],
    ) -> Result<Vec<StorefrontConfig>, StorefrontConfigRepositoryError> {
        retailer_codes
            .iter()
            .map(|&code| {
                get_storefront_config(code)
                    .ok_or(StorefrontConfigRepositoryError::UnsupportedRetailer(code))
            })
            .collect()
    }
}

pub async fn run(request: InitialFillOrder, dispatcher: Dispatcher) -> InitialFillStatus {
    if request.retailers.is_empty() {
        return InitialFillStatus::InvalidRequest(PreparationFailure::EmptyRetailers);
    }

    let prepared = match build_fills(&request.retailers) {
        Ok(prepared) => prepared,
        Err(error) => return InitialFillStatus::InvalidRequest(error),
    };

    match start_fills(dispatcher, prepared).await {
        Ok(fills) => InitialFillStatus::Queued(fills),
        Err(failure) => InitialFillStatus::QueueFailed(failure),
    }
}

fn build_fills(retailers: &[String]) -> Result<Vec<PlannedRun>, PreparationFailure> {
    let retailer_codes = convert_to_retailer_codes(retailers)?;
    reject_duplicate_retailers(&retailer_codes)?;

    let retailer_configs = StorefrontConfigRepository::fetch(&retailer_codes)?;

    let mut prepared: Vec<PlannedRun> = Vec::new();

    for config in retailer_configs {
        let run_id = Uuid::now_v7();
        // Resolve the locale fan-out now so inventory problems reject the request.

        let x = |message| PreparationFailure::invalid_inventory(config.retailer_code(), message);

        let catalog = initial(run_id, config.retailer_code(), None).map_err(x)?;

        let command = if config.attempts_token() {
            let url = config.storefront_url().into();

            let cmd = AcquireStorefrontToken::new(run_id, config.retailer_code(), url);

            StartingCommands::Token(cmd)
        } else {
            StartingCommands::Catalog(catalog)
        };

        let fill = PlannedRun::new(config.retailer_code().into(), run_id, command);
        prepared.push(fill);
    }
    Ok(prepared)
}

fn convert_to_retailer_codes(codes: &[String]) -> Result<Vec<RetailerCode>, PreparationFailure> {
    let parse = |code: &String| {
        RetailerCode::from_str(code).map_err(|_| PreparationFailure::UnknownRetailer(code.clone()))
    };

    codes.iter().map(parse).collect()
}

/// Rejects the request when two entries resolve to the same retailer, so one
/// request never starts two fills for it.
fn reject_duplicate_retailers(codes: &[RetailerCode]) -> Result<(), PreparationFailure> {
    let mut seen = HashSet::new();

    match codes.iter().find(|code| !seen.insert(code.as_str())) {
        Some(code) => Err(PreparationFailure::DuplicateRetailer(code.as_str())),
        None => Ok(()),
    }
}

#[derive(Debug)]
pub(crate) enum StartingCommands {
    Token(AcquireStorefrontToken),
    /// Tokenless runs start with one catalog page per supported locale.
    Catalog(Vec<KeyedCommand<FetchProductCatalogIdentifiers>>),
}

#[derive(Debug)]
pub(crate) struct PlannedRun {
    pub(crate) identity: FillRun,
    command: StartingCommands,
}

impl PlannedRun {
    pub(crate) fn new(retailer: String, run_id: Uuid, command: StartingCommands) -> Self {
        let identity = FillRun::new(retailer, run_id);

        Self { identity, command }
    }
}

/// Queues each prepared fill on the command bus, off the async runtime.
pub(crate) async fn start_fills(
    dispatcher: Dispatcher,
    prepared: Vec<PlannedRun>,
) -> Result<Vec<FillRun>, StartFailure> {
    let queued = spawn_blocking(move || {
        dispatch_until_failure(prepared, |command, run_id| {
            dispatch_starting_commands(&dispatcher, command, run_id)
        })
    })
    .await;

    match queued {
        Ok(result) => result.map_err(|(failure, fills)| {
            let failed = StartFailure::new(fills);
            error!(error_id=%failed.error_id, error=?failure, "failed to queue initial fill");
            failed
        }),
        Err(failure) => {
            let failed = StartFailure::new(Vec::new());
            error!(error_id=%failed.error_id, error=?failure, "initial fill queue task failed; enqueue outcomes unknown");
            Err(failed)
        }
    }
}

/// Each dispatch is independent. Stop on failure and retain confirmed successes.
fn dispatch_until_failure<E>(
    prepared: Vec<PlannedRun>,
    mut dispatch: impl FnMut(&StartingCommands, Uuid) -> Result<(), E>,
) -> Result<Vec<FillRun>, (E, Vec<FillRun>)> {
    let mut fills = Vec::with_capacity(prepared.len());

    for prepared in prepared {
        if let Err(failure) = dispatch(&prepared.command, prepared.identity.run_id) {
            return Err((failure, fills));
        }
        fills.push(prepared.identity);
    }
    Ok(fills)
}

/// A token run starts with one command. A tokenless run starts with one
/// catalog command per locale, stopping at the first failure.
fn dispatch_starting_commands(
    dispatcher: &Dispatcher,
    command: &StartingCommands,
    run_id: Uuid,
) -> Result<(), DispatchError> {
    match command {
        StartingCommands::Token(cmd) => dispatcher.dispatch_run(cmd, run_id),
        StartingCommands::Catalog(cmds) => cmds
            .iter()
            .try_for_each(|cmd| dispatcher.dispatch_keyed(cmd)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_retailers() {
        let code = RetailerCode::from_str("ankernordicscom").unwrap();

        let result = reject_duplicate_retailers(&[code, code]);

        assert!(matches!(
            result,
            Err(PreparationFailure::DuplicateRetailer(ref found)) if *found == code.as_str()
        ));
    }

    #[test]
    fn accepts_distinct_retailers() {
        let code = RetailerCode::from_str("ankernordicscom").unwrap();

        assert!(reject_duplicate_retailers(&[code]).is_ok());
    }
}

#[cfg(test)]
mod queue_tests {
    use super::*;
    use crate::v2::incoming::InitialFillContexts;

    #[test]
    fn disconnected_retailers_cannot_be_queued_for_initial_fill() {
        {
            let code = "micom";
            assert_eq!(
                build_fills(&[code.into()]).unwrap_err().to_string(),
                format!("initial fill is not implemented for retailer {code}")
            );
        }
    }

    #[test]
    fn experimental_ankerau_can_be_prepared() {
        let fills = build_fills(&["ankerau".into()]).unwrap();
        assert_eq!(fills.len(), 1);
        assert!(matches!(fills[0].command, StartingCommands::Token(_)));
    }

    fn build_fills_for(codes: &[&str]) -> Vec<PlannedRun> {
        build_fills(
            &codes
                .iter()
                .map(|code| (*code).to_owned())
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    #[test]
    fn tokenless_stores_start_at_each_inventory_locale() {
        for prepared in build_fills_for(&[
            "ugreenkr",
            "minisforumhk",
            "ugreenfr",
            "minisforumjp",
            "minisforumkr",
            "ugreeneu",
            "ugreenin",
            "ugreennas",
            "ugreennasca",
            "ugreennaseu",
            "ugreennl",
            "ugreenuk",
            "ugreenus",
        ]) {
            let StartingCommands::Catalog(commands) = prepared.command else {
                panic!("tokenless store must start with catalog commands");
            };
            let retailer = RetailerCode::from_str(&prepared.identity.retailer).unwrap();
            let contexts = get_storefront_config(retailer)
                .unwrap()
                .initial_fill_contexts()
                .unwrap();
            assert_eq!(commands.len(), contexts.len());
            for (queued, context) in commands.iter().zip(contexts) {
                assert_eq!(queued.command.retailer_code, retailer);
                assert_eq!(queued.command.context.as_ref(), Some(&context));
                assert_eq!(queued.command.run_id, prepared.identity.run_id);
                assert!(queued.command.storefront_token.is_none());
            }
        }
    }

    #[test]
    fn queue_preserves_order_and_dispatches_each_start_with_its_run_id() {
        let prepared = build_fills_for(&["ankerau", "ankeritalycom", "ankerjapancom"]);
        let mut dispatched = Vec::new();
        let fills = dispatch_until_failure(prepared, |command, run_id| {
            match command {
                StartingCommands::Token(command) => {
                    assert!(matches!(
                        command.retailer_code,
                        RetailerCode::AnkerAu
                            | RetailerCode::AnkerItalyCom
                            | RetailerCode::AnkerJapanCom
                    ));
                    assert_eq!(
                        command.url,
                        get_storefront_config(command.retailer_code)
                            .unwrap()
                            .storefront_url()
                    );
                    assert_eq!(command.run_id, run_id);
                }
                StartingCommands::Catalog(commands) => {
                    assert!(!commands.is_empty());
                    for keyed in commands {
                        let command = &keyed.command;
                        assert!(matches!(
                            command.retailer_code,
                            RetailerCode::AnkerItalyCom | RetailerCode::AnkerJapanCom
                        ));
                        assert_eq!(keyed.run_id, run_id);
                        assert_eq!(command.run_id, run_id);
                        assert!(command.context.is_some());
                        assert!(command.storefront_token.is_none());
                    }
                }
            }
            dispatched.push(run_id);
            Ok::<(), ()>(())
        })
        .unwrap();
        assert_eq!(
            fills
                .iter()
                .map(|f| f.retailer.as_str())
                .collect::<Vec<_>>(),
            ["ankerau", "ankeritalycom", "ankerjapancom"]
        );
        assert_eq!(
            fills.iter().map(|f| f.run_id).collect::<Vec<_>>(),
            dispatched
        );
        assert_eq!(dispatched.iter().collect::<HashSet<_>>().len(), 3);
        let next = build_fills_for(&["ankerau"]);
        assert!(!dispatched.contains(&next[0].identity.run_id));
    }

    #[test]
    fn queue_failure_returns_prior_successes_and_stops_dispatching() {
        for fail_at in 0..3 {
            let prepared = build_fills_for(&["ankerau", "ankerca", "ankeritalycom"]);
            let ids: Vec<_> = prepared.iter().map(|p| p.identity.run_id).collect();
            let mut calls = 0;
            let (failure, fills) = dispatch_until_failure(prepared, |_, _| {
                let current = calls;
                calls += 1;
                if current == fail_at {
                    Err("fixture failure")
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
            assert_eq!(failure, "fixture failure");
            assert_eq!(calls, fail_at + 1);
            assert_eq!(
                fills.iter().map(|f| f.run_id).collect::<Vec<_>>(),
                ids[..fail_at]
            );
        }
    }
}

#[cfg(test)]
mod market_tests {
    use super::*;
    #[test]
    fn each_retailer_starts_with_its_configured_access_policy() {
        let codes: Vec<String> = RetailerCode::ALL
            .iter()
            .copied()
            .filter(|r| get_storefront_config(*r).is_some())
            .map(|r| r.as_str())
            .collect();
        assert_eq!(codes.len(), 48);
        let fills = build_fills(&codes).unwrap();
        assert_eq!(fills.len(), 48);
        for fill in fills {
            if matches!(
                fill.identity.retailer.as_str(),
                "ugreenkr"
                    | "minisforumhk"
                    | "ugreenfr"
                    | "minisforumjp"
                    | "minisforumkr"
                    | "ugreeneu"
                    | "ugreenin"
                    | "ugreennas"
                    | "ugreennasca"
                    | "ugreennaseu"
                    | "ugreennl"
                    | "ugreenuk"
                    | "ugreenus"
            ) {
                assert!(matches!(fill.command, StartingCommands::Catalog(_)));
            } else {
                assert!(matches!(fill.command, StartingCommands::Token(_)));
            }
        }
    }
}
