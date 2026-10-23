//! Parse phone numbers from international format by auto-detecting country.

use crate::country_code::CountryCode;
use crate::prelude::{PhoneNumber, PhoneNumberError};

/// Parses a phone number from international format, automatically detecting the country.
///
/// Accepts international format strings starting with `+` followed by the country calling code.
/// Currently supports:
/// - Croatia: `+385 {area_code} {number}`
/// - Slovenia: `+386 {area_code} {number}`
///
/// # Examples
///
/// ```
/// use everyday::prelude::phone_from_international_format;
///
/// let phone = phone_from_international_format("+385 91 1234567").unwrap();
/// assert_eq!(phone.format_international().unwrap(), "+385 91 1234567");
///
/// let phone = phone_from_international_format("+386 31 234567").unwrap();
/// assert_eq!(phone.format_international().unwrap(), "+386 31 234567");
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The string doesn't start with `+`
/// - The country calling code is not supported
/// - The format doesn't match the expected pattern for that country
/// - The phone number components are invalid
pub fn phone_from_international_format(input: &str) -> Result<PhoneNumber, PhoneNumberError> {
    let trimmed = input.trim();

    if !trimmed.starts_with('+') {
        return Err(PhoneNumberError::InvalidInternationalFormat(
            trimmed.to_string(),
        ));
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(PhoneNumberError::InvalidInternationalFormat(
            trimmed.to_string(),
        ));
    }

    // Detect country from calling code
    let country = match parts[0] {
        "+385" => CountryCode::HR,
        "+386" => CountryCode::SI,
        _ => {
            return Err(PhoneNumberError::InvalidInternationalFormat(
                trimmed.to_string(),
            ));
        }
    };

    let area_code = parts[1].to_string();
    let number = parts[2].to_string();

    PhoneNumber::new(country, area_code, number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_croatian_mobile() {
        let phone = phone_from_international_format("+385 91 1234567").unwrap();
        assert_eq!(phone.area_code(), "91");
        assert_eq!(phone.number(), "1234567");
        assert_eq!(phone.format_international().unwrap(), "+385 91 1234567");
    }

    #[test]
    fn test_parse_croatian_zagreb() {
        let phone = phone_from_international_format("+385 1 2345678").unwrap();
        assert_eq!(phone.area_code(), "1");
        assert_eq!(phone.number(), "2345678");
        assert_eq!(phone.format_international().unwrap(), "+385 1 2345678");
    }

    #[test]
    fn test_parse_croatian_regional() {
        let phone = phone_from_international_format("+385 21 123456").unwrap();
        assert_eq!(phone.area_code(), "21");
        assert_eq!(phone.number(), "123456");
        assert_eq!(phone.format_international().unwrap(), "+385 21 123456");
    }

    #[test]
    fn test_parse_croatian_toll_free() {
        let phone = phone_from_international_format("+385 800 123456").unwrap();
        assert_eq!(phone.area_code(), "800");
        assert_eq!(phone.number(), "123456");
        assert_eq!(phone.format_international().unwrap(), "+385 800 123456");
    }

    #[test]
    fn test_parse_slovenian_mobile() {
        let phone = phone_from_international_format("+386 31 234567").unwrap();
        assert_eq!(phone.area_code(), "31");
        assert_eq!(phone.number(), "234567");
        assert_eq!(phone.format_international().unwrap(), "+386 31 234567");
    }

    #[test]
    fn test_parse_slovenian_ljubljana() {
        let phone = phone_from_international_format("+386 1 2345678").unwrap();
        assert_eq!(phone.area_code(), "1");
        assert_eq!(phone.number(), "2345678");
        assert_eq!(phone.format_international().unwrap(), "+386 1 2345678");
    }

    #[test]
    fn test_parse_slovenian_regional() {
        let phone = phone_from_international_format("+386 2 123456").unwrap();
        assert_eq!(phone.area_code(), "2");
        assert_eq!(phone.number(), "123456");
        assert_eq!(phone.format_international().unwrap(), "+386 2 123456");
    }

    #[test]
    fn test_missing_plus_sign() {
        let result = phone_from_international_format("385 91 1234567");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PhoneNumberError::InvalidInternationalFormat(_)
        ));
    }

    #[test]
    fn test_unsupported_country_code() {
        let result = phone_from_international_format("+49 30 12345678");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PhoneNumberError::InvalidInternationalFormat(_)
        ));
    }

    #[test]
    fn test_wrong_number_of_parts() {
        let result = phone_from_international_format("+385 91");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PhoneNumberError::InvalidInternationalFormat(_)
        ));
    }

    #[test]
    fn test_with_extra_whitespace() {
        let phone = phone_from_international_format("  +385 91 1234567  ").unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 91 1234567");
    }
}
