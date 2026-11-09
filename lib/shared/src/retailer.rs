use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RetailerCode {
    MinisForumEu,
    MinisForumUs,
    MinisForumUk,
    MinisForumFr,
    MinisForumCa,
    MinisForumAu,
    MinisForumKr,
    MinisForumJp,
    MinisForumRu,
    MinisForumHk,

    // Xiaomi
    MiCom,

    // UGREEN
    UgreenCom,
    UgreenUs,
    UgreenCa,
    UgreenEu,
    UgreenDe,
    UgreenUk,
    UgreenFr,
    UgreenNl,
    UgreenJp,
    UgreenKr,
    UgreenIn,
    UgreenNas,
    UgreenNasCa,
    UgreenNasEu,
    UgreenNasDe,
    UgreenNasUk,
    UgreenNasFr,
    UgreenNasEs,
    UgreenNasIt,
    UgreenNasAu,
    UgreenNasJp,

    // Anker
    AnkerCom,
    AnkerJapanCom,
    AnkerKr,
    AnkerItalyCom,
    AnkerNordicsCom,
    AnkerUk,
    AnkerCa,
    AnkerEu,
    AnkerDe,
    AnkerFr,
    AnkerPl,
    AnkerAu,
    AnkerNz,
    // AnkerMy,
    AnkerVn,
}

/// Every retailer processed by the `destructure`/`valueless` binaries, as
/// `(directory name, code)`.
pub const RETAILERS: &[(&str, RetailerCode)] = &[
    ("MinisForumAu", RetailerCode::MinisForumAu),
    ("MinisForumCa", RetailerCode::MinisForumCa),
    ("MinisForumEu", RetailerCode::MinisForumEu),
    ("MinisForumUs", RetailerCode::MinisForumUs),
    ("MinisForumUk", RetailerCode::MinisForumUk),
    ("MinisForumFr", RetailerCode::MinisForumFr),
    ("MinisForumKr", RetailerCode::MinisForumKr),
    ("MinisForumJp", RetailerCode::MinisForumJp),
    ("MinisForumRu", RetailerCode::MinisForumRu),
    ("MinisForumHk", RetailerCode::MinisForumHk),
    ("MiCom", RetailerCode::MiCom),
    ("UgreenCom", RetailerCode::UgreenCom),
    ("UgreenUs", RetailerCode::UgreenUs),
    ("UgreenCa", RetailerCode::UgreenCa),
    ("UgreenEu", RetailerCode::UgreenEu),
    ("UgreenDe", RetailerCode::UgreenDe),
    ("UgreenUk", RetailerCode::UgreenUk),
    ("UgreenFr", RetailerCode::UgreenFr),
    ("UgreenNl", RetailerCode::UgreenNl),
    ("UgreenJp", RetailerCode::UgreenJp),
    ("UgreenKr", RetailerCode::UgreenKr),
    ("UgreenIn", RetailerCode::UgreenIn),
    ("UgreenNas", RetailerCode::UgreenNas),
    ("UgreenNasCa", RetailerCode::UgreenNasCa),
    ("UgreenNasEu", RetailerCode::UgreenNasEu),
    ("UgreenNasDe", RetailerCode::UgreenNasDe),
    ("UgreenNasUk", RetailerCode::UgreenNasUk),
    ("UgreenNasFr", RetailerCode::UgreenNasFr),
    ("UgreenNasEs", RetailerCode::UgreenNasEs),
    ("UgreenNasIt", RetailerCode::UgreenNasIt),
    ("UgreenNasAu", RetailerCode::UgreenNasAu),
    ("UgreenNasJp", RetailerCode::UgreenNasJp),
    ("AnkerCom", RetailerCode::AnkerCom),
    ("AnkerJapanCom", RetailerCode::AnkerJapanCom),
    ("AnkerKr", RetailerCode::AnkerKr),
    ("AnkerItalyCom", RetailerCode::AnkerItalyCom),
    ("AnkerNordicsCom", RetailerCode::AnkerNordicsCom),
    ("AnkerUk", RetailerCode::AnkerUk),
    ("AnkerCa", RetailerCode::AnkerCa),
    ("AnkerEu", RetailerCode::AnkerEu),
    ("AnkerDe", RetailerCode::AnkerDe),
    ("AnkerFr", RetailerCode::AnkerFr),
    ("AnkerPl", RetailerCode::AnkerPl),
    ("AnkerAu", RetailerCode::AnkerAu),
    ("AnkerNz", RetailerCode::AnkerNz),
    // ("AnkerMy", RetailerCode::AnkerMy),
    ("AnkerVn", RetailerCode::AnkerVn),
];

/// Resolves a stored retailer directory name to its registered code.
///
/// Directory names on disk are lowercase slugs, so this is case-insensitive and
/// delegates to the slug table rather than the CamelCase `RETAILERS` names.
pub fn code_for_name(name: &str) -> Option<RetailerCode> {
    RetailerCode::from_str(&name.to_lowercase()).ok()
}

#[derive(Error, Debug, PartialEq)]
#[error("unknown retailer code: {0}")]
pub struct RetailerCodeConversionError(String);

impl TryFrom<&str> for RetailerCode {
    type Error = RetailerCodeConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "minisforumeu" => Ok(Self::MinisForumEu),
            "minisforumus" => Ok(Self::MinisForumUs),
            "minisforumuk" => Ok(Self::MinisForumUk),
            "minisforumfr" => Ok(Self::MinisForumFr),
            "minisforumca" => Ok(Self::MinisForumCa),
            "minisforumau" => Ok(Self::MinisForumAu),
            "minisforumkr" => Ok(Self::MinisForumKr),
            "minisforumjp" => Ok(Self::MinisForumJp),
            "minisforumru" => Ok(Self::MinisForumRu),
            "minisforumhk" => Ok(Self::MinisForumHk),

            "micom" => Ok(Self::MiCom),

            "ugreencom" => Ok(Self::UgreenCom),
            "ugreenus" => Ok(Self::UgreenUs),
            "ugreenca" => Ok(Self::UgreenCa),
            "ugreeneu" => Ok(Self::UgreenEu),
            "ugreende" => Ok(Self::UgreenDe),
            "ugreenuk" => Ok(Self::UgreenUk),
            "ugreenfr" => Ok(Self::UgreenFr),
            "ugreennl" => Ok(Self::UgreenNl),
            "ugreenjp" => Ok(Self::UgreenJp),
            "ugreenkr" => Ok(Self::UgreenKr),
            "ugreenin" => Ok(Self::UgreenIn),
            "ugreennas" => Ok(Self::UgreenNas),
            "ugreennasca" => Ok(Self::UgreenNasCa),
            "ugreennaseu" => Ok(Self::UgreenNasEu),
            "ugreennasde" => Ok(Self::UgreenNasDe),
            "ugreennasuk" => Ok(Self::UgreenNasUk),
            "ugreennasfr" => Ok(Self::UgreenNasFr),
            "ugreennases" => Ok(Self::UgreenNasEs),
            "ugreennasit" => Ok(Self::UgreenNasIt),
            "ugreennasau" => Ok(Self::UgreenNasAu),
            "ugreennasjp" => Ok(Self::UgreenNasJp),

            "ankercom" => Ok(Self::AnkerCom),
            "ankerjapancom" => Ok(Self::AnkerJapanCom),
            "ankerkr" => Ok(Self::AnkerKr),
            "ankeritalycom" => Ok(Self::AnkerItalyCom),
            "ankernordicscom" => Ok(Self::AnkerNordicsCom),
            "ankeruk" => Ok(Self::AnkerUk),
            "ankerca" => Ok(Self::AnkerCa),
            "ankereu" => Ok(Self::AnkerEu),
            "ankerde" => Ok(Self::AnkerDe),
            "ankerfr" => Ok(Self::AnkerFr),
            "ankerpl" => Ok(Self::AnkerPl),
            "ankerau" => Ok(Self::AnkerAu),
            "ankernz" => Ok(Self::AnkerNz),
            // "ankermy" => Ok(Self::AnkerMy),
            "ankervn" => Ok(Self::AnkerVn),

            unknown => Err(RetailerCodeConversionError(unknown.to_string())),
        }
    }
}

impl TryFrom<String> for RetailerCode {
    type Error = RetailerCodeConversionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl From<RetailerCode> for String {
    fn from(retailer_code: RetailerCode) -> Self {
        format!("{retailer_code:?}").to_lowercase()
    }
}

impl RetailerCode {
    /// Every known retailer code.
    pub const ALL: [RetailerCode; 46] = [
        RetailerCode::AnkerCom,
        RetailerCode::AnkerJapanCom,
        RetailerCode::AnkerKr,
        RetailerCode::AnkerItalyCom,
        RetailerCode::AnkerNordicsCom,
        RetailerCode::AnkerUk,
        RetailerCode::AnkerCa,
        RetailerCode::AnkerEu,
        RetailerCode::AnkerDe,
        RetailerCode::AnkerFr,
        RetailerCode::AnkerPl,
        RetailerCode::AnkerAu,
        RetailerCode::AnkerNz,
        // They moved everything but the content pages to some local distributer. Since Malaysia is not a target market, skipping it for now
        // RetailerCode::AnkerMy,
        RetailerCode::AnkerVn,
        RetailerCode::MiCom,
        RetailerCode::MinisForumEu,
        RetailerCode::MinisForumUs,
        RetailerCode::MinisForumUk,
        RetailerCode::MinisForumFr,
        RetailerCode::MinisForumCa,
        RetailerCode::MinisForumAu,
        RetailerCode::MinisForumKr,
        RetailerCode::MinisForumJp,
        RetailerCode::MinisForumRu,
        RetailerCode::MinisForumHk,
        RetailerCode::UgreenCom,
        RetailerCode::UgreenUs,
        RetailerCode::UgreenCa,
        RetailerCode::UgreenEu,
        RetailerCode::UgreenDe,
        RetailerCode::UgreenUk,
        RetailerCode::UgreenFr,
        RetailerCode::UgreenNl,
        RetailerCode::UgreenJp,
        RetailerCode::UgreenKr,
        RetailerCode::UgreenIn,
        RetailerCode::UgreenNas,
        RetailerCode::UgreenNasCa,
        RetailerCode::UgreenNasEu,
        RetailerCode::UgreenNasDe,
        RetailerCode::UgreenNasUk,
        RetailerCode::UgreenNasFr,
        RetailerCode::UgreenNasEs,
        RetailerCode::UgreenNasIt,
        RetailerCode::UgreenNasAu,
        RetailerCode::UgreenNasJp,
    ];

    // DEPRECATED: use from_str() instead
    pub fn from_slug(slug: &str) -> Option<RetailerCode> {
        Self::try_from(slug).ok()
    }

    // DEPRECATED: use as_str() instead
    pub fn slug(self) -> String {
        self.into()
    }

    // Preserve the existing inherent API and its String error type for callers.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(slug: &str) -> Result<RetailerCode, String> {
        Self::try_from(slug).map_err(|error| error.to_string())
    }

    pub fn as_str(self) -> String {
        self.into()
    }
}
#[cfg(test)]
mod tests {
    use super::code_for_name;

    #[test]
    fn resolves_a_code_from_a_lowercase_directory_name() {
        assert_eq!(code_for_name("ankercom"), Some(RetailerCode::AnkerCom));
        assert_eq!(code_for_name("AnkerCom"), Some(RetailerCode::AnkerCom));
        assert_eq!(code_for_name("not-a-retailer"), None);
    }
    use super::{RetailerCode, RetailerCodeConversionError};

    #[test]
    fn resolves_code_from_slug() {
        assert_eq!(
            RetailerCode::from_slug("minisforumeu"),
            Some(RetailerCode::MinisForumEu)
        );
        assert_eq!(RetailerCode::from_slug("nope"), None);
    }

    #[test]
    fn converts_string_to_retailer_code() {
        let retailer_code = RetailerCode::try_from("minisforumeu".to_string()).unwrap();

        assert_eq!(retailer_code, RetailerCode::MinisForumEu);
    }

    // Proves invalid retailer slugs are rejected with the expected error payload.
    #[test]
    fn rejects_unknown_retailer_code() {
        let error = RetailerCode::try_from("unknown").unwrap_err();

        assert_eq!(error, RetailerCodeConversionError("unknown".to_string()));
    }

    // Proves a retailer enum converts back to its lowercase slug string.
    #[test]
    fn converts_retailer_code_to_string() {
        let retailer_code: String = RetailerCode::MinisForumEu.into();

        assert_eq!(retailer_code, "minisforumeu");
    }

    // Proves the Australian retailer slug maps to the correct enum variant.
    #[test]
    fn converts_minisforumau_string_to_retailer_code() {
        let retailer_code = RetailerCode::try_from("minisforumau".to_string()).unwrap();

        assert_eq!(retailer_code, RetailerCode::MinisForumAu);
    }
}
