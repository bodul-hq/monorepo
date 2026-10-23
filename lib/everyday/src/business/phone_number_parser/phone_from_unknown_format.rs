//! Generic phone number parsing that dispatches to country-specific parsers.

use crate::country_code::CountryCode;
use crate::prelude::{PhoneNumber, PhoneNumberError};

use super::hr_phone_from_unknown_format::hr_phone_from_unknown_format;
use super::si_phone_from_unknown_format::si_phone_from_unknown_format;

/// Parses a phone number from unknown format based on the country code.
///
/// Dispatches to the appropriate country-specific parser:
/// - `HR` (Croatia): Uses [`hr_phone_from_unknown_format`]
/// - `SI` (Slovenia): Uses [`si_phone_from_unknown_format`]
///
/// # Panics
///
/// Panics if the country code is not supported. Currently only Croatia (HR)
/// and Slovenia (SI) are supported.
///
/// # Examples
///
/// ```
/// use everyday::prelude::{CountryCode, phone_from_unknown_format};
///
/// let hr = CountryCode::new("HR").unwrap();
/// let phone = phone_from_unknown_format(&hr, "091 123 4567").unwrap();
/// assert_eq!(phone.format_international().unwrap(), "+385 91 1234567");
///
/// let si = CountryCode::new("SI").unwrap();
/// let phone = phone_from_unknown_format(&si, "031 234 567").unwrap();
/// assert_eq!(phone.format_international().unwrap(), "+386 31 234567");
/// ```
pub fn phone_from_unknown_format(
    country: &CountryCode,
    number: &str,
) -> Result<PhoneNumber, PhoneNumberError> {
    match country.as_str() {
        "HR" => hr_phone_from_unknown_format(number),
        "SI" => si_phone_from_unknown_format(number),
        code => panic!("Unsupported country code for phone parsing: {}", code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_from_unknown_format_hr() {
        let hr = CountryCode::new("HR").unwrap();

        let phone = phone_from_unknown_format(&hr, "091 123 4567").unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 91 1234567");

        let phone = phone_from_unknown_format(&hr, "+385 1 234 5678").unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 1 2345678");

        let phone = phone_from_unknown_format(&hr, "0800 123456").unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 800 123456");
    }

    #[test]
    fn test_phone_from_unknown_format_si() {
        let si = CountryCode::new("SI").unwrap();

        let phone = phone_from_unknown_format(&si, "031 234 567").unwrap();
        assert_eq!(phone.format_international().unwrap(), "+386 31 234567");

        let phone = phone_from_unknown_format(&si, "+386 1 234 5678").unwrap();
        assert_eq!(phone.format_international().unwrap(), "+386 1 2345678");

        let phone = phone_from_unknown_format(&si, "02 123 456").unwrap();
        assert_eq!(phone.format_international().unwrap(), "+386 2 123456");
    }

    #[test]
    #[should_panic(expected = "Unsupported country code")]
    fn test_phone_from_unknown_format_unsupported() {
        let de = CountryCode::new("DE").unwrap();
        let _ = phone_from_unknown_format(&de, "030 12345678");
    }
}
