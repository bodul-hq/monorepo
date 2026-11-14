use shared::prelude::RetailerCode;

/// A step in the initial offer fill process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    DiscoverMarkets,
    AcquireStorefrontToken,
    ProductCatalogIdentifiers,
    OfferDetails,
}

/// The steps in a retailer's initial offer fill process.
#[derive(Debug, Clone)]
pub struct InitialFillProcess {
    pub steps: Vec<Step>,
}

/// Returns the initial offer fill steps for a retailer.
///
/// # Panics
///
/// Panics for MiCom and Anker Korea, whose processes are not implemented yet.
pub fn initial_fill_process(retailer_code: RetailerCode) -> InitialFillProcess {
    if let Some(config) = crate::initial_fill_config_for(retailer_code)
        && config.attempts_token()
    {
        let mut steps = vec![Step::AcquireStorefrontToken];
        if crate::all_market_contexts(retailer_code) {
            steps.push(Step::DiscoverMarkets);
        }
        steps.extend([Step::ProductCatalogIdentifiers, Step::OfferDetails]);
        return InitialFillProcess { steps };
    }
    if crate::all_market_contexts(retailer_code) {
        return InitialFillProcess {
            steps: vec![
                Step::DiscoverMarkets,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        };
    }
    if crate::initial_fill_config_for(retailer_code).is_some_and(|config| !config.requires_token())
    {
        return InitialFillProcess {
            steps: vec![Step::ProductCatalogIdentifiers, Step::OfferDetails],
        };
    }
    match retailer_code {
        RetailerCode::MinisForumEu => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::MinisForumUs => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::MinisForumUk => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::MinisForumFr => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::MinisForumCa => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::MinisForumAu => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::MinisForumKr => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::MinisForumJp => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::MinisForumRu => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::MinisForumHk => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::MiCom => todo!("initial offer fill for MiCom"),
        RetailerCode::UgreenCom => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenUs => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenCa => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenEu => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenDe => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenUk => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenFr => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNl => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenJp => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenKr => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenIn => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNas => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNasCa => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNasEu => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNasDe => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNasUk => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNasFr => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNasEs => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNasIt => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNasAu => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::UgreenNasJp => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerCom => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerJapanCom => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerKr => todo!("initial offer fill for AnkerKr"),
        RetailerCode::AnkerItalyCom => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerNordicsCom => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerSe => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerNo => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerFi => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerIs => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerDk => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerUk => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerCa => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerEu => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerDe => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerFr => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerPl => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerAu => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerNz => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
        RetailerCode::AnkerVn => InitialFillProcess {
            steps: vec![
                Step::AcquireStorefrontToken,
                Step::ProductCatalogIdentifiers,
                Step::OfferDetails,
            ],
        },
    }
}
