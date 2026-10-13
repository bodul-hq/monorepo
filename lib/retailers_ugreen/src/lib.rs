pub mod ugreenca;
pub mod ugreencom;
pub mod ugreende;
pub mod ugreeneu;
pub mod ugreenfr;
pub mod ugreenin;
pub mod ugreenjp;
pub mod ugreenkr;
pub mod ugreennas;
pub mod ugreennasau;
pub mod ugreennasca;
pub mod ugreennasde;
pub mod ugreennases;
pub mod ugreennaseu;
pub mod ugreennasfr;
pub mod ugreennasit;
pub mod ugreennasjp;
pub mod ugreennasuk;
pub mod ugreennl;
pub mod ugreenuk;
pub mod ugreenus;

use bodul_shared::prelude::{NormalizedData, RetailerCode};
use html_extractor::io::Architecture;

pub fn architecture_for(retailer: RetailerCode) -> Option<Architecture> {
    match retailer {
        RetailerCode::UgreenCa => Some(ugreenca::architecture()),
        RetailerCode::UgreenCom => Some(ugreencom::architecture()),
        RetailerCode::UgreenDe => Some(ugreende::architecture()),
        RetailerCode::UgreenEu => Some(ugreeneu::architecture()),
        RetailerCode::UgreenFr => Some(ugreenfr::architecture()),
        RetailerCode::UgreenIn => Some(ugreenin::architecture()),
        RetailerCode::UgreenJp => Some(ugreenjp::architecture()),
        RetailerCode::UgreenKr => Some(ugreenkr::architecture()),
        RetailerCode::UgreenNas => Some(ugreennas::architecture()),
        RetailerCode::UgreenNasAu => Some(ugreennasau::architecture()),
        RetailerCode::UgreenNasCa => Some(ugreennasca::architecture()),
        RetailerCode::UgreenNasDe => Some(ugreennasde::architecture()),
        RetailerCode::UgreenNasEs => Some(ugreennases::architecture()),
        RetailerCode::UgreenNasEu => Some(ugreennaseu::architecture()),
        RetailerCode::UgreenNasFr => Some(ugreennasfr::architecture()),
        RetailerCode::UgreenNasIt => Some(ugreennasit::architecture()),
        RetailerCode::UgreenNasJp => Some(ugreennasjp::architecture()),
        RetailerCode::UgreenNasUk => Some(ugreennasuk::architecture()),
        RetailerCode::UgreenNl => Some(ugreennl::architecture()),
        RetailerCode::UgreenUk => Some(ugreenuk::architecture()),
        RetailerCode::UgreenUs => Some(ugreenus::architecture()),
        _ => None,
    }
}

pub fn normalize_for(retailer: RetailerCode, json: &str) -> Option<Result<NormalizedData, String>> {
    match retailer {
        RetailerCode::UgreenCa => Some(ugreenca::normalize(json)),
        RetailerCode::UgreenCom => Some(ugreencom::normalize(json)),
        RetailerCode::UgreenDe => Some(ugreende::normalize(json)),
        RetailerCode::UgreenEu => Some(ugreeneu::normalize(json)),
        RetailerCode::UgreenFr => Some(ugreenfr::normalize(json)),
        RetailerCode::UgreenIn => Some(ugreenin::normalize(json)),
        RetailerCode::UgreenJp => Some(ugreenjp::normalize(json)),
        RetailerCode::UgreenKr => Some(ugreenkr::normalize(json)),
        RetailerCode::UgreenNas => Some(ugreennas::normalize(json)),
        RetailerCode::UgreenNasAu => Some(ugreennasau::normalize(json)),
        RetailerCode::UgreenNasCa => Some(ugreennasca::normalize(json)),
        RetailerCode::UgreenNasDe => Some(ugreennasde::normalize(json)),
        RetailerCode::UgreenNasEs => Some(ugreennases::normalize(json)),
        RetailerCode::UgreenNasEu => Some(ugreennaseu::normalize(json)),
        RetailerCode::UgreenNasFr => Some(ugreennasfr::normalize(json)),
        RetailerCode::UgreenNasIt => Some(ugreennasit::normalize(json)),
        RetailerCode::UgreenNasJp => Some(ugreennasjp::normalize(json)),
        RetailerCode::UgreenNasUk => Some(ugreennasuk::normalize(json)),
        RetailerCode::UgreenNl => Some(ugreennl::normalize(json)),
        RetailerCode::UgreenUk => Some(ugreenuk::normalize(json)),
        RetailerCode::UgreenUs => Some(ugreenus::normalize(json)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    // Placeholder until the crate has code, so `make test` still builds and
    // runs it. Replace it with real tests.
    #[test]
    fn crate_builds() {
        assert_eq!(env!("CARGO_PKG_NAME"), "retailers_ugreen");
    }
}
