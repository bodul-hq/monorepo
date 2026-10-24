//! ISO 3166-1 alpha-2 country codes.
//!
//! This module provides [`CountryCode`] for representing and validating
//! two-letter country codes according to the ISO 3166-1 alpha-2 standard.

use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Macro for defining country code enum variants and their string representations.
///
/// This macro generates:
/// - The enum definition with all 249 ISO 3166-1 alpha-2 variants
/// - The `as_str()` method that returns the uppercase string representation
/// - The `from_str_normalized()` method that parses from an uppercase normalized string
///
/// # Example
///
/// ```ignore
/// define_country_codes! {
///     AD, AE, AF, AG, ...
/// }
/// ```
macro_rules! define_country_codes {
    ( $( $code:ident ),* $(,)? ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum CountryCode {
            $( $code, )*
        }

        impl CountryCode {
            /// Returns the uppercase string representation of the country code.
            pub fn as_str(&self) -> &'static str {
                match self {
                    $( CountryCode::$code => stringify!($code), )*
                }
            }

            /// Creates a CountryCode from an already-normalized uppercase string.
            ///
            /// Returns `None` if the string doesn't match any known country code.
            /// For construction with normalization, use `CountryCode::new()` instead.
            fn from_str_normalized(s: &str) -> Option<Self> {
                match s {
                    $( stringify!($code) => Some(CountryCode::$code), )*
                    _ => None,
                }
            }
        }
    };
}

define_country_codes! {
    AD, AE, AF, AG, AI, AL, AM, AO, AQ, AR, AS, AT, AU, AW, AX, AZ,
    BA, BB, BD, BE, BF, BG, BH, BI, BJ, BL, BM, BN, BO, BQ, BR, BS,
    BT, BV, BW, BY, BZ, CA, CC, CD, CF, CG, CH, CI, CK, CL, CM, CN,
    CO, CR, CU, CV, CW, CX, CY, CZ, DE, DJ, DK, DM, DO, DZ, EC, EE,
    EG, EH, ER, ES, ET, FI, FJ, FK, FM, FO, FR, GA, GB, GD, GE, GF,
    GG, GH, GI, GL, GM, GN, GP, GQ, GR, GS, GT, GU, GW, GY, HK, HM,
    HN, HR, HT, HU, ID, IE, IL, IM, IN, IO, IQ, IR, IS, IT, JE, JM,
    JO, JP, KE, KG, KH, KI, KM, KN, KP, KR, KW, KY, KZ, LA, LB, LC,
    LI, LK, LR, LS, LT, LU, LV, LY, MA, MC, MD, ME, MF, MG, MH, MK,
    ML, MM, MN, MO, MP, MQ, MR, MS, MT, MU, MV, MW, MX, MY, MZ, NA,
    NC, NE, NF, NG, NI, NL, NO, NP, NR, NU, NZ, OM, PA, PE, PF, PG,
    PH, PK, PL, PM, PN, PR, PS, PT, PW, PY, QA, RE, RO, RS, RU, RW,
    SA, SB, SC, SD, SE, SG, SH, SI, SJ, SK, SL, SM, SN, SO, SR, SS,
    ST, SV, SX, SY, SZ, TC, TD, TF, TG, TH, TJ, TK, TL, TM, TN, TO,
    TR, TT, TV, TW, TZ, UA, UG, UM, US, UY, UZ, VA, VC, VE, VG, VI,
    VN, VU, WF, WS, YE, YT, ZA, ZM, ZW,
}

/// Error type for country code validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountryCodeError {
    /// Input was empty or whitespace-only.
    Empty,
    /// Input did not have exactly 2 ASCII letters.
    InvalidFormat(String),
    /// Code format was valid but not in the ISO 3166-1 list.
    UnknownCode(String),
}

impl fmt::Display for CountryCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CountryCodeError::Empty => write!(f, "Country code cannot be empty"),
            CountryCodeError::InvalidFormat(input) => {
                write!(f, "Country code must be exactly 2 letters, got: {input}")
            }
            CountryCodeError::UnknownCode(code) => {
                write!(f, "Unknown country code: {code}")
            }
        }
    }
}

impl std::error::Error for CountryCodeError {}

impl CountryCode {
    /// Creates a new country code from a string.
    ///
    /// # Normalization
    ///
    /// - Whitespace is trimmed from both ends
    /// - Letters are converted to uppercase
    /// - "UK" is normalized to "GB"
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input is empty or whitespace-only
    /// - Input is not exactly 2 ASCII letters
    /// - The code is not in the ISO 3166-1 list
    pub fn new(input: &str) -> Result<Self, CountryCodeError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(CountryCodeError::Empty);
        }

        let bytes = trimmed.as_bytes();
        if bytes.len() != 2 {
            return Err(CountryCodeError::InvalidFormat(trimmed.to_string()));
        }

        // Check both characters are ASCII letters
        if !bytes[0].is_ascii_alphabetic() || !bytes[1].is_ascii_alphabetic() {
            return Err(CountryCodeError::InvalidFormat(trimmed.to_string()));
        }

        // Normalize to uppercase
        let mut code = [bytes[0].to_ascii_uppercase(), bytes[1].to_ascii_uppercase()];

        // Special case: UK -> GB
        if code == *b"UK" {
            code = *b"GB";
        }

        // Safety: we just created valid ASCII uppercase letters
        let code_str = std::str::from_utf8(&code).unwrap();

        CountryCode::from_str_normalized(code_str)
            .ok_or_else(|| CountryCodeError::UnknownCode(code_str.to_string()))
    }
}

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CountryCode {
    type Err = CountryCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod valid_input_tests {
        use super::*;

        #[test]
        fn accepts_uppercase_codes() {
            assert!(CountryCode::new("HR").is_ok());
            assert!(CountryCode::new("DE").is_ok());
            assert!(CountryCode::new("GB").is_ok());
            assert!(CountryCode::new("US").is_ok());
        }

        #[test]
        fn normalizes_lowercase_to_uppercase() {
            let code = CountryCode::new("hr").unwrap();
            assert_eq!(code.as_str(), "HR");
        }

        #[test]
        fn normalizes_mixed_case() {
            let code = CountryCode::new("Hr").unwrap();
            assert_eq!(code.as_str(), "HR");
        }

        #[test]
        fn normalizes_uk_to_gb() {
            let code = CountryCode::new("UK").unwrap();
            assert_eq!(code.as_str(), "GB");
        }

        #[test]
        fn normalizes_lowercase_uk_to_gb() {
            let code = CountryCode::new("uk").unwrap();
            assert_eq!(code.as_str(), "GB");
        }

        #[test]
        fn trims_whitespace() {
            let code = CountryCode::new(" SI ").unwrap();
            assert_eq!(code.as_str(), "SI");
        }

        #[test]
        fn trims_leading_whitespace() {
            let code = CountryCode::new("  HR").unwrap();
            assert_eq!(code.as_str(), "HR");
        }

        #[test]
        fn trims_trailing_whitespace() {
            let code = CountryCode::new("HR  ").unwrap();
            assert_eq!(code.as_str(), "HR");
        }

        #[test]
        fn combined_normalization() {
            let code = CountryCode::new(" uk ").unwrap();
            assert_eq!(code.as_str(), "GB");
        }

        #[test]
        fn accepts_codes_from_different_regions() {
            // Europe
            assert!(CountryCode::new("AD").is_ok());
            assert!(CountryCode::new("HR").is_ok());
            // Americas
            assert!(CountryCode::new("BR").is_ok());
            assert!(CountryCode::new("US").is_ok());
            // Asia
            assert!(CountryCode::new("CN").is_ok());
            assert!(CountryCode::new("JP").is_ok());
            assert!(CountryCode::new("IN").is_ok());
            // Africa
            assert!(CountryCode::new("NG").is_ok());
            assert!(CountryCode::new("ZA").is_ok());
            // Oceania
            assert!(CountryCode::new("AU").is_ok());
            assert!(CountryCode::new("NZ").is_ok());
        }

        #[test]
        fn accepts_special_political_status_codes() {
            // Disputed territories
            assert!(CountryCode::new("EH").is_ok()); // Western Sahara
            assert!(CountryCode::new("FK").is_ok()); // Falkland Islands
            assert!(CountryCode::new("PS").is_ok()); // Palestine
            assert!(CountryCode::new("TW").is_ok()); // Taiwan
        }
    }

    mod invalid_input_tests {
        use super::*;

        #[test]
        fn rejects_empty_string() {
            let result = CountryCode::new("");
            assert_eq!(result.unwrap_err(), CountryCodeError::Empty);
        }

        #[test]
        fn rejects_whitespace_only() {
            let result = CountryCode::new("   ");
            assert_eq!(result.unwrap_err(), CountryCodeError::Empty);
        }

        #[test]
        fn rejects_single_character() {
            let result = CountryCode::new("H");
            assert!(matches!(
                result.unwrap_err(),
                CountryCodeError::InvalidFormat(_)
            ));
        }

        #[test]
        fn rejects_three_characters() {
            let result = CountryCode::new("USA");
            assert!(matches!(
                result.unwrap_err(),
                CountryCodeError::InvalidFormat(_)
            ));
        }

        #[test]
        fn rejects_digits() {
            let result = CountryCode::new("1A");
            assert!(matches!(
                result.unwrap_err(),
                CountryCodeError::InvalidFormat(_)
            ));

            let result = CountryCode::new("A1");
            assert!(matches!(
                result.unwrap_err(),
                CountryCodeError::InvalidFormat(_)
            ));

            let result = CountryCode::new("12");
            assert!(matches!(
                result.unwrap_err(),
                CountryCodeError::InvalidFormat(_)
            ));
        }

        #[test]
        fn rejects_unknown_code() {
            let result = CountryCode::new("XX");
            assert_eq!(
                result.unwrap_err(),
                CountryCodeError::UnknownCode("XX".to_string())
            );
        }

        #[test]
        fn rejects_user_assigned_codes() {
            assert!(matches!(
                CountryCode::new("AA").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
            assert!(matches!(
                CountryCode::new("QQ").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
            assert!(matches!(
                CountryCode::new("XY").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
            assert!(matches!(
                CountryCode::new("ZZ").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
        }

        #[test]
        fn rejects_exceptionally_reserved_codes() {
            // EU is reserved for European Union
            assert!(matches!(
                CountryCode::new("EU").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
            // UN is reserved for United Nations
            assert!(matches!(
                CountryCode::new("UN").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
        }

        #[test]
        fn rejects_transitionally_reserved_codes() {
            // AN - Netherlands Antilles (dissolved 2010)
            assert!(matches!(
                CountryCode::new("AN").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
            // CS - Czechoslovakia
            assert!(matches!(
                CountryCode::new("CS").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
            // SU - USSR
            assert!(matches!(
                CountryCode::new("SU").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
            // YU - Yugoslavia
            assert!(matches!(
                CountryCode::new("YU").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
        }

        #[test]
        fn rejects_language_code_en() {
            // EN is often confused with a country code
            assert!(matches!(
                CountryCode::new("EN").unwrap_err(),
                CountryCodeError::UnknownCode(_)
            ));
        }
    }

    mod display_tests {
        use super::*;

        #[test]
        fn display_shows_uppercase() {
            let code = CountryCode::new("hr").unwrap();
            assert_eq!(format!("{}", code), "HR");
        }

        #[test]
        fn to_string_returns_code() {
            let code = CountryCode::new("DE").unwrap();
            assert_eq!(code.to_string(), "DE");
        }
    }

    mod from_str_tests {
        use super::*;

        #[test]
        fn from_str_works() {
            let code: CountryCode = "HR".parse().unwrap();
            assert_eq!(code.as_str(), "HR");
        }

        #[test]
        fn from_str_normalizes() {
            let code: CountryCode = "uk".parse().unwrap();
            assert_eq!(code.as_str(), "GB");
        }
    }

    mod error_message_tests {
        use super::*;

        #[test]
        fn empty_error_message() {
            let err = CountryCodeError::Empty;
            assert_eq!(err.to_string(), "Country code cannot be empty");
        }

        #[test]
        fn invalid_format_error_message() {
            let err = CountryCodeError::InvalidFormat("USA".to_string());
            assert_eq!(
                err.to_string(),
                "Country code must be exactly 2 letters, got: USA"
            );
        }

        #[test]
        fn unknown_code_error_message() {
            let err = CountryCodeError::UnknownCode("XX".to_string());
            assert_eq!(err.to_string(), "Unknown country code: XX");
        }
    }

    mod equality_tests {
        use super::*;

        #[test]
        fn same_codes_are_equal() {
            let a = CountryCode::new("HR").unwrap();
            let b = CountryCode::new("HR").unwrap();
            assert_eq!(a, b);
        }

        #[test]
        fn different_cases_normalize_to_equal() {
            let a = CountryCode::new("HR").unwrap();
            let b = CountryCode::new("hr").unwrap();
            assert_eq!(a, b);
        }

        #[test]
        fn uk_and_gb_are_equal() {
            let uk = CountryCode::new("UK").unwrap();
            let gb = CountryCode::new("GB").unwrap();
            assert_eq!(uk, gb);
        }

        #[test]
        fn different_codes_are_not_equal() {
            let hr = CountryCode::new("HR").unwrap();
            let de = CountryCode::new("DE").unwrap();
            assert_ne!(hr, de);
        }
    }

    mod copy_tests {
        use super::*;

        #[test]
        fn country_code_is_copy() {
            let code = CountryCode::new("HR").unwrap();
            let copy = code;
            assert_eq!(code, copy);
        }
    }
}
