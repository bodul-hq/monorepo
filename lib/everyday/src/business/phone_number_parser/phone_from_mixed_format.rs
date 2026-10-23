//! Parse phone numbers from mixed format (toll-free in local, others in international).

use crate::country_code::CountryCode;
use crate::prelude::{PhoneNumber, PhoneNumberError};

/// Parses a phone number from mixed format input.
///
/// This parser handles the common case where:
/// - Toll-free/special service numbers are in local format (e.g., "0800 123456")
/// - Regular numbers (mobile, landline) are in international format (e.g., "+385 91 1234567")
///
/// Currently supports Croatian (HR) and Slovenian (SI) numbers.
///
/// # Special Service Numbers (Local Format)
///
/// Croatian special service numbers:
/// - `0800` - Toll-free numbers
/// - `0801` - Shared cost numbers
///
/// These are expected in local format starting with `0`.
///
/// # Regular Numbers (International Format)
///
/// Regular mobile and landline numbers should be in international format:
/// - Croatian: `+385 {area_code} {number}`
/// - Slovenian: `+386 {area_code} {number}`
///
/// # Examples
///
/// ```
/// use everyday::prelude::phone_from_mixed_format;
///
/// // Toll-free in local format
/// let phone = phone_from_mixed_format("0800 123456").unwrap();
/// assert_eq!(phone.format_international().unwrap(), "+385 800 123456");
///
/// // Regular mobile in international format
/// let phone = phone_from_mixed_format("+385 91 1234567").unwrap();
/// assert_eq!(phone.format_international().unwrap(), "+385 91 1234567");
///
/// // Slovenian mobile in international format
/// let phone = phone_from_mixed_format("+386 31 234567").unwrap();
/// assert_eq!(phone.format_international().unwrap(), "+386 31 234567");
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The format doesn't match either local special service or international regular format
/// - The phone number components are invalid
/// - The country is not supported
pub fn phone_from_mixed_format(input: &str) -> Result<PhoneNumber, PhoneNumberError> {
    let trimmed = input.trim();

    // Check if it's a local format starting with 0
    if trimmed.starts_with('0') {
        // Extract digits only
        let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();

        // Check for Croatian special service numbers
        #[allow(clippy::manual_strip)]
        if digits.starts_with("0800") || digits.starts_with("0801") {
            // Parse as Croatian toll-free/special service number
            let area_and_subscriber = &digits[1..]; // Remove leading 0

            // Determine area code and subscriber
            let (area_code, subscriber) = if area_and_subscriber.starts_with("800") {
                // 0800 toll-free: 800 + 4-7 digits
                let len = area_and_subscriber.len();
                if (7..=10).contains(&len) {
                    (&area_and_subscriber[..3], &area_and_subscriber[3..])
                } else {
                    return Err(PhoneNumberError::InvalidNumberLength(trimmed.to_string()));
                }
            } else if area_and_subscriber.starts_with("801") {
                // 0801 shared cost: 801 + 4-7 digits
                let len = area_and_subscriber.len();
                if (7..=10).contains(&len) {
                    (&area_and_subscriber[..3], &area_and_subscriber[3..])
                } else {
                    return Err(PhoneNumberError::InvalidNumberLength(trimmed.to_string()));
                }
            } else {
                return Err(PhoneNumberError::InvalidLocalFormat(trimmed.to_string()));
            };

            let country = CountryCode::HR;
            return PhoneNumber::new(country, area_code.to_string(), subscriber.to_string());
        }

        // If it starts with 0 but is not a recognized special service number,
        // it's an invalid format (should be international)
        return Err(PhoneNumberError::InvalidLocalFormat(trimmed.to_string()));
    }

    // If it starts with +, parse as international format
    if trimmed.starts_with('+') {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 3 {
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
        // Join remaining parts as the number (handles numbers with spaces)
        let number = parts[2..].join("");

        return PhoneNumber::new(country, area_code, number);
    }

    // Doesn't match either format
    Err(PhoneNumberError::InvalidLocalFormat(trimmed.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Croatian toll-free numbers (local format)

    #[test]
    fn test_croatian_toll_free_6_digits() {
        let phone = phone_from_mixed_format("0800 123456").unwrap();
        assert_eq!(phone.area_code(), "800");
        assert_eq!(phone.number(), "123456");
        assert_eq!(phone.format_international().unwrap(), "+385 800 123456");
    }

    #[test]
    fn test_croatian_toll_free_7_digits() {
        let phone = phone_from_mixed_format("0800 400 000").unwrap();
        assert_eq!(phone.area_code(), "800");
        assert_eq!(phone.number(), "400000");
        assert_eq!(phone.format_international().unwrap(), "+385 800 400000");
    }

    #[test]
    fn test_croatian_toll_free_4_digits() {
        let phone = phone_from_mixed_format("0800 0801").unwrap();
        assert_eq!(phone.area_code(), "800");
        assert_eq!(phone.number(), "0801");
        assert_eq!(phone.format_international().unwrap(), "+385 800 0801");
    }

    #[test]
    fn test_croatian_shared_cost_801() {
        let phone = phone_from_mixed_format("0801 1234").unwrap();
        assert_eq!(phone.area_code(), "801");
        assert_eq!(phone.number(), "1234");
        assert_eq!(phone.format_international().unwrap(), "+385 801 1234");
    }

    #[test]
    fn test_croatian_toll_free_no_spaces() {
        let phone = phone_from_mixed_format("0800123456").unwrap();
        assert_eq!(phone.area_code(), "800");
        assert_eq!(phone.number(), "123456");
        assert_eq!(phone.format_international().unwrap(), "+385 800 123456");
    }

    // Croatian regular numbers (international format)

    #[test]
    fn test_croatian_mobile_international() {
        let phone = phone_from_mixed_format("+385 91 1234567").unwrap();
        assert_eq!(phone.area_code(), "91");
        assert_eq!(phone.number(), "1234567");
        assert_eq!(phone.format_international().unwrap(), "+385 91 1234567");
    }

    #[test]
    fn test_croatian_zagreb_international() {
        let phone = phone_from_mixed_format("+385 1 2345678").unwrap();
        assert_eq!(phone.area_code(), "1");
        assert_eq!(phone.number(), "2345678");
        assert_eq!(phone.format_international().unwrap(), "+385 1 2345678");
    }

    #[test]
    fn test_croatian_regional_international() {
        let phone = phone_from_mixed_format("+385 21 123456").unwrap();
        assert_eq!(phone.area_code(), "21");
        assert_eq!(phone.number(), "123456");
        assert_eq!(phone.format_international().unwrap(), "+385 21 123456");
    }

    // Slovenian numbers (international format only)

    #[test]
    fn test_slovenian_mobile_international() {
        let phone = phone_from_mixed_format("+386 31 234567").unwrap();
        assert_eq!(phone.area_code(), "31");
        assert_eq!(phone.number(), "234567");
        assert_eq!(phone.format_international().unwrap(), "+386 31 234567");
    }

    #[test]
    fn test_slovenian_ljubljana_international() {
        let phone = phone_from_mixed_format("+386 1 2345678").unwrap();
        assert_eq!(phone.area_code(), "1");
        assert_eq!(phone.number(), "2345678");
        assert_eq!(phone.format_international().unwrap(), "+386 1 2345678");
    }

    #[test]
    fn test_slovenian_regional_international() {
        let phone = phone_from_mixed_format("+386 2 123456").unwrap();
        assert_eq!(phone.area_code(), "2");
        assert_eq!(phone.number(), "123456");
        assert_eq!(phone.format_international().unwrap(), "+386 2 123456");
    }

    // Error cases

    #[test]
    fn test_local_format_regular_number_rejected() {
        // Regular Croatian mobile should be international, not local
        let result = phone_from_mixed_format("091 1234567");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PhoneNumberError::InvalidLocalFormat(_)
        ));
    }

    #[test]
    fn test_missing_plus_sign() {
        let result = phone_from_mixed_format("385 91 1234567");
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_country() {
        let result = phone_from_mixed_format("+49 30 12345678");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PhoneNumberError::InvalidInternationalFormat(_)
        ));
    }

    #[test]
    fn test_invalid_toll_free_length() {
        // Too short
        let result = phone_from_mixed_format("0800 12");
        assert!(result.is_err());
    }

    #[test]
    fn test_with_whitespace() {
        let phone = phone_from_mixed_format("  +385 91 1234567  ").unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 91 1234567");

        let phone = phone_from_mixed_format("  0800 123456  ").unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 800 123456");
    }

    #[test]
    fn test_number_with_spaces_in_subscriber() {
        // Handle numbers formatted like "+385 1 234 5678"
        let phone = phone_from_mixed_format("+385 1 234 5678").unwrap();
        assert_eq!(phone.area_code(), "1");
        assert_eq!(phone.number(), "2345678");
        assert_eq!(phone.format_international().unwrap(), "+385 1 2345678");
    }

    #[test]
    fn test_mobile_with_spaces_in_subscriber() {
        // Handle numbers formatted like "+385 91 123 4567"
        let phone = phone_from_mixed_format("+385 91 123 4567").unwrap();
        assert_eq!(phone.area_code(), "91");
        assert_eq!(phone.number(), "1234567");
        assert_eq!(phone.format_international().unwrap(), "+385 91 1234567");
    }
}
