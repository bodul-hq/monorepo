//! Offer discovery.
//!
//! Turns the product URLs of each consolidated sitemap into offers, downloads
//! their pages and reads the offer data out of them. Six feature slices run in
//! sequence, each started by the previous one's event:
//!
//! 1. [`discover_offers`] records an offer for every product URL of a
//!    consolidated sitemap.
//! 2. [`download_offer_page`] downloads each offer's page.
//! 3. [`extract_offer_page`] extracts the downloaded page into JSON with the
//!    retailer's architecture.
//! 4. [`normalize_offer`] normalizes the extracted JSON.
//! 5. [`structuralize_offer`] structuralizes the normalized JSON.
//! 6. [`processing_for_export`] prepares an immutable export of priced variations.
//!
//! This module holds the domain shared by the slices. Other contexts may only
//! use [`prelude`].

pub mod prelude;

mod schema;

mod discover_offers;
mod download_offer_page;
mod extract_offer_page;
mod normalize_offer;
mod processing_for_export;
mod structuralize_offer;

use std::any::Any;
use std::panic::{self, AssertUnwindSafe};

use bodul_shared::retailer::RetailerCode;

/// The retailer registry: how each retailer's offer data is extracted,
/// normalized and structuralized.
///
/// Every method returns an error rather than panicking: a retailer step that
/// is still a `todo!()` stub fails the command instead of the whole run.
#[derive(Debug, Clone, Copy, Default)]
struct Retailers;

impl Retailers {
    /// The page extracted into JSON text.
    fn extract(self, retailer_code: RetailerCode, html: &str) -> Result<String, String> {
        catch_panic(|| retailer_data_extractor::extract_for(retailer_code, html))
    }

    /// The normalized document, or `None` when the retailer produces none for
    /// this page.
    fn normalize(self, retailer_code: RetailerCode, json: &str) -> Result<Option<String>, String> {
        catch_panic(|| {
            retailer_data_extractor::normalize_for(retailer_code, json)
                .ok_or_else(|| format!("no normalizer for retailer {retailer_code:?}"))?
                .map(|data| data.document_json)
        })
    }

    /// The structuralized document, or `None` when the retailer produces none
    /// for this offer.
    fn structuralize(
        self,
        retailer_code: RetailerCode,
        json: &str,
    ) -> Result<Option<String>, String> {
        catch_panic(|| {
            retailer_data_extractor::structuralize_for(retailer_code, json)
                .ok_or_else(|| format!("no structuralizer for retailer {retailer_code:?}"))?
                .map(|data| data.document_json)
        })
    }
}

/// Runs `step`, turning a panic into an error with the panic's message.
fn catch_panic<T>(step: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    panic::catch_unwind(AssertUnwindSafe(step))
        .unwrap_or_else(|payload| Err(format!("panicked: {}", panic_message(&*payload))))
}

fn panic_message(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("non-string panic payload")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_panicking_step_becomes_an_error() {
        let result: Result<(), String> = catch_panic(|| todo!());

        assert_eq!(result, Err("panicked: not yet implemented".to_string()));
    }

    #[test]
    fn a_retailer_without_a_structuralizer_is_an_error() {
        let result = Retailers.structuralize(RetailerCode::MiCom, "{}");

        assert_eq!(
            result,
            Err("no structuralizer for retailer MiCom".to_string())
        );
    }
}
