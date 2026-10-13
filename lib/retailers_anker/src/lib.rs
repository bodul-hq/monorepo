pub mod ankerau;
pub mod ankerca;
pub mod ankercom;
pub mod ankerde;
pub mod ankereu;
pub mod ankerfr;
pub mod ankeritalycom;
pub mod ankerjapancom;
pub mod ankerkr;
pub mod ankernordicscom;
pub mod ankernz;
pub mod ankerpl;
pub mod ankeruk;
pub mod ankervn;

use bodul_shared::prelude::{NormalizedData, RetailerCode};
use html_extractor::io::Architecture;

pub fn architecture_for(retailer: RetailerCode) -> Option<Architecture> {
    match retailer {
        RetailerCode::AnkerAu => Some(ankerau::architecture()),
        RetailerCode::AnkerCa => Some(ankerca::architecture()),
        RetailerCode::AnkerCom => Some(ankercom::architecture()),
        RetailerCode::AnkerDe => Some(ankerde::architecture()),
        RetailerCode::AnkerEu => Some(ankereu::architecture()),
        RetailerCode::AnkerFr => Some(ankerfr::architecture()),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom::architecture()),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom::architecture()),
        RetailerCode::AnkerKr => Some(ankerkr::architecture()),
        RetailerCode::AnkerNordicsCom => Some(ankernordicscom::architecture()),
        RetailerCode::AnkerNz => Some(ankernz::architecture()),
        RetailerCode::AnkerPl => Some(ankerpl::architecture()),
        RetailerCode::AnkerUk => Some(ankeruk::architecture()),
        RetailerCode::AnkerVn => Some(ankervn::architecture()),
        _ => None,
    }
}

pub fn normalize_for(retailer: RetailerCode, json: &str) -> Option<Result<NormalizedData, String>> {
    match retailer {
        RetailerCode::AnkerAu => Some(ankerau::normalize(json)),
        RetailerCode::AnkerCa => Some(ankerca::normalize(json)),
        RetailerCode::AnkerCom => Some(ankercom::normalize(json)),
        RetailerCode::AnkerDe => Some(ankerde::normalize(json)),
        RetailerCode::AnkerEu => Some(ankereu::normalize(json)),
        RetailerCode::AnkerFr => Some(ankerfr::normalize(json)),
        RetailerCode::AnkerItalyCom => Some(ankeritalycom::normalize(json)),
        RetailerCode::AnkerJapanCom => Some(ankerjapancom::normalize(json)),
        RetailerCode::AnkerKr => Some(ankerkr::normalize(json)),
        RetailerCode::AnkerNordicsCom => Some(ankernordicscom::normalize(json)),
        RetailerCode::AnkerNz => Some(ankernz::normalize(json)),
        RetailerCode::AnkerPl => Some(ankerpl::normalize(json)),
        RetailerCode::AnkerUk => Some(ankeruk::normalize(json)),
        RetailerCode::AnkerVn => Some(ankervn::normalize(json)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    // Placeholder until the crate has code, so `make test` still builds and
    // runs it. Replace it with real tests.
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailers_anker");
    }
}
