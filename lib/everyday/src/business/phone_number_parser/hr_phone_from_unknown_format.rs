//! Croatian phone number parsing from various formats.

use crate::country_code::CountryCode;
use crate::prelude::{PhoneNumber, PhoneNumberError};

/// Parses a Croatian phone number from various formats into a `PhoneNumber`.
///
/// Accepts:
/// - International format with +385: `+385 98 123 4567`
/// - International format with 00385: `00385 98 123 4567`
/// - Local format: `098 123 4567`
/// - Toll-free numbers: `0800 123456`, `0801 123456`
///
/// Handles various separators (spaces, dashes, dots) and no separators.
pub fn hr_phone_from_unknown_format(i: &str) -> Result<PhoneNumber, PhoneNumberError> {
    let trimmed = i.trim();

    // Extract all digits first
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();

    // Determine the prefix and remaining digits
    let area_and_subscriber = if trimmed.starts_with('+') {
        // +385... format
        if let Some(rest) = digits.strip_prefix("385") {
            rest
        } else {
            return Err(PhoneNumberError::InvalidInternationalFormat(
                trimmed.to_string(),
            ));
        }
    } else if let Some(rest) = digits.strip_prefix("00385") {
        // 00385... format
        rest
    } else if let Some(rest) = digits.strip_prefix('0') {
        // Local format: 0XX...
        rest
    } else {
        return Err(PhoneNumberError::InvalidLocalFormat(trimmed.to_string()));
    };

    // Determine area code length
    // Zagreb has area code "1", others have 2-digit area codes
    // Special service numbers (0800, 0801) have 3-digit prefix
    // Total length should be 7-10 (area 1-3 + subscriber 4-7)
    let len = area_and_subscriber.len();
    if !(7..=10).contains(&len) {
        return Err(PhoneNumberError::InvalidNumberLength(trimmed.to_string()));
    }

    // Determine area code and subscriber based on the pattern
    let (area_code, subscriber) =
        if area_and_subscriber.starts_with("800") || area_and_subscriber.starts_with("801") {
            // Toll-free numbers: 0800/0801 XXXX to XXXXXXX (3 + 4-7 digits)
            if (7..=10).contains(&len) {
                (&area_and_subscriber[..3], &area_and_subscriber[3..])
            } else {
                return Err(PhoneNumberError::InvalidAreaCodeLength(trimmed.to_string()));
            }
        } else if area_and_subscriber.starts_with('1') && (len == 7 || len == 8) {
            // Zagreb: area code "1"
            (&area_and_subscriber[..1], &area_and_subscriber[1..])
        } else if len >= 8 {
            // Standard regional: 2-digit area code
            (&area_and_subscriber[..2], &area_and_subscriber[2..])
        } else {
            // 7 digits with non-Zagreb area code (2 + 5 is invalid, need 2 + 6 minimum)
            return Err(PhoneNumberError::InvalidAreaCodeLength(trimmed.to_string()));
        };

    let country = CountryCode::HR;
    PhoneNumber::new(country, area_code.to_string(), subscriber.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_hr_phone_from_unknown_format() {
        let cases = HashMap::from([
            // International format - mobile
            ("+385 98 123 4567", "+385 98 1234567"),
            ("+385 98 123 456", "+385 98 123456"),
            ("+38598123456", "+385 98 123456"),
            ("00385 98 123 456", "+385 98 123456"),
            ("0038598123456", "+385 98 123456"),
            // International format - Zagreb (area code 1)
            ("+385 1 12 34 567", "+385 1 1234567"),
            ("+385 1 1234 567", "+385 1 1234567"),
            ("+38511234567", "+385 1 1234567"),
            ("00385 1 1234 567", "+385 1 1234567"),
            ("0038511234567", "+385 1 1234567"),
            // International format - regional
            ("+385 21 123 456", "+385 21 123456"),
            ("+385 43 123 456", "+385 43 123456"),
            ("+385 40 123 456", "+385 40 123456"),
            // Local format - mobile
            ("091 123 4567", "+385 91 1234567"),
            ("091 1234 567", "+385 91 1234567"),
            ("099 123 4567", "+385 99 1234567"),
            // Local format - Zagreb (area code 1)
            ("01 1234 567", "+385 1 1234567"),
            ("01 123 45 67", "+385 1 1234567"),
            ("01 12 34 567", "+385 1 1234567"),
            ("01 123 4567", "+385 1 1234567"),
            // Local format - regional
            ("021 123 456", "+385 21 123456"),
            ("072 12 34 56", "+385 72 123456"),
            // Toll-free numbers (6 digits)
            ("0800 123456", "+385 800 123456"),
            ("0800123456", "+385 800 123456"),
            ("0800 12 34 56", "+385 800 123456"),
            ("+385 800 123456", "+385 800 123456"),
            ("00385 800 123456", "+385 800 123456"),
            // Toll-free numbers (7 digits)
            ("0800 400 000", "+385 800 400000"),
            ("0800400000", "+385 800 400000"),
            ("0800 40 00 00", "+385 800 400000"),
            // 0801 numbers (6 digits)
            ("0801 123456", "+385 801 123456"),
            ("0801123456", "+385 801 123456"),
            ("0801 12 34 56", "+385 801 123456"),
            ("+385 801 123456", "+385 801 123456"),
            ("00385 801 123456", "+385 801 123456"),
            // 0801 numbers (7 digits)
            ("0801 400 000", "+385 801 400000"),
            ("0801400000", "+385 801 400000"),
            ("0801 40 00 00", "+385 801 400000"),
            // Short toll-free numbers (4 digits)
            ("0800 0801", "+385 800 0801"),
            ("08000801", "+385 800 0801"),
        ]);

        for (input, expected) in cases {
            let phone = hr_phone_from_unknown_format(input)
                .unwrap_or_else(|e| panic!("Failed to parse '{}': {:?}", input, e));
            let result = phone.format_international().unwrap();
            assert_eq!(result, expected, "Wrong output for input '{}'", input);
        }
    }
}
