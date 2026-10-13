pub mod minisforumau;
pub mod minisforumca;
pub mod minisforumeu;
pub mod minisforumfr;
pub mod minisforumhk;
pub mod minisforumjp;
pub mod minisforumkr;
pub mod minisforumru;
pub mod minisforumuk;
pub mod minisforumus;

use bodul_shared::prelude::{NormalizedData, RetailerCode};
use html_extractor::io::Architecture;

pub fn architecture_for(retailer: RetailerCode) -> Option<Architecture> {
    match retailer {
        RetailerCode::MinisForumAu => Some(minisforumau::architecture()),
        RetailerCode::MinisForumCa => Some(minisforumca::architecture()),
        RetailerCode::MinisForumEu => Some(minisforumeu::architecture()),
        RetailerCode::MinisForumFr => Some(minisforumfr::architecture()),
        RetailerCode::MinisForumHk => Some(minisforumhk::architecture()),
        RetailerCode::MinisForumJp => Some(minisforumjp::architecture()),
        RetailerCode::MinisForumKr => Some(minisforumkr::architecture()),
        RetailerCode::MinisForumRu => Some(minisforumru::architecture()),
        RetailerCode::MinisForumUk => Some(minisforumuk::architecture()),
        RetailerCode::MinisForumUs => Some(minisforumus::architecture()),
        _ => None,
    }
}

pub fn normalize_for(retailer: RetailerCode, json: &str) -> Option<Result<NormalizedData, String>> {
    match retailer {
        RetailerCode::MinisForumAu => Some(minisforumau::normalize(json)),
        RetailerCode::MinisForumCa => Some(minisforumca::normalize(json)),
        RetailerCode::MinisForumEu => Some(minisforumeu::normalize(json)),
        RetailerCode::MinisForumFr => Some(minisforumfr::normalize(json)),
        RetailerCode::MinisForumHk => Some(minisforumhk::normalize(json)),
        RetailerCode::MinisForumJp => Some(minisforumjp::normalize(json)),
        RetailerCode::MinisForumKr => Some(minisforumkr::normalize(json)),
        RetailerCode::MinisForumRu => Some(minisforumru::normalize(json)),
        RetailerCode::MinisForumUk => Some(minisforumuk::normalize(json)),
        RetailerCode::MinisForumUs => Some(minisforumus::normalize(json)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    // Placeholder until the crate has code, so `make test` still builds and
    // runs it. Replace it with real tests.
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailers_minisforum");
    }
}
