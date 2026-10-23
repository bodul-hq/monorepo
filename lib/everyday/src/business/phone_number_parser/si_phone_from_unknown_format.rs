//! Slovenian phone number parsing from various formats.

use crate::country_code::CountryCode;
use crate::prelude::{PhoneNumber, PhoneNumberError};

/// Parses a Slovenian phone number from various formats into a `PhoneNumber`.
///
/// Accepts:
/// - International format with +386: `+386 31 234 567`
/// - International format with 00386: `00386 31 234 567`
/// - Local format with leading 0: `031 234 567`
///
/// Handles various separators (spaces, dashes, dots) and no separators.
///
/// Slovenian phone numbers are 9 digits total (including leading 0 in local format):
/// - Mobile: 3-digit prefix (030, 031, 040, 041, 051, 068, 069, 070, 071) + 6-digit subscriber
/// - Landline: 1-2 digit area code + 6-7 digit subscriber
///   - Ljubljana area code: 1 (1 + 7 digits)
///   - Other cities: 2-digit area codes (2 + 6 digits)
pub fn si_phone_from_unknown_format(i: &str) -> Result<PhoneNumber, PhoneNumberError> {
    let trimmed = i.trim();

    // Extract all digits first
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();

    // Determine the prefix and remaining digits
    let area_and_subscriber = if trimmed.starts_with('+') {
        // +386... format
        if let Some(rest) = digits.strip_prefix("386") {
            rest
        } else {
            return Err(PhoneNumberError::InvalidInternationalFormat(
                trimmed.to_string(),
            ));
        }
    } else if let Some(rest) = digits.strip_prefix("00386") {
        // 00386... format
        rest
    } else if let Some(rest) = digits.strip_prefix('0') {
        // Local format: 0XX...
        rest
    } else {
        return Err(PhoneNumberError::InvalidLocalFormat(trimmed.to_string()));
    };

    // Total length should be 7-9 digits
    // - Mobile: 9 (3-digit prefix + 6-digit subscriber)
    // - Ljubljana: 8 (1-digit area + 7-digit subscriber)
    // - Regional (1-digit): 7 (1-digit area + 6-digit subscriber)
    // - Regional (2-digit): 8 (2-digit area + 6-digit subscriber)
    let len = area_and_subscriber.len();
    if !(7..=9).contains(&len) {
        return Err(PhoneNumberError::InvalidNumberLength(trimmed.to_string()));
    }

    // Determine area code and subscriber based on the pattern
    let (area_code, subscriber) = if is_mobile_prefix(area_and_subscriber) {
        // Mobile numbers: 3-digit prefix + 6-digit subscriber
        if len == 9 {
            (&area_and_subscriber[..3], &area_and_subscriber[3..])
        } else {
            return Err(PhoneNumberError::InvalidAreaCodeLength(trimmed.to_string()));
        }
    } else if area_and_subscriber.starts_with('1') && len == 8 {
        // Ljubljana: area code "1" + 7-digit subscriber
        (&area_and_subscriber[..1], &area_and_subscriber[1..])
    } else if len == 8 {
        // Standard regional with 2-digit area code: 2 + 6-digit subscriber
        (&area_and_subscriber[..2], &area_and_subscriber[2..])
    } else if len == 7 {
        // Regional with 1-digit area code: 1 + 6-digit subscriber
        (&area_and_subscriber[..1], &area_and_subscriber[1..])
    } else {
        return Err(PhoneNumberError::InvalidAreaCodeLength(trimmed.to_string()));
    };

    let country = CountryCode::SI;
    PhoneNumber::new(country, area_code.to_string(), subscriber.to_string())
}

/// Checks if the given string starts with a known Slovenian mobile prefix.
fn is_mobile_prefix(s: &str) -> bool {
    // Mobile prefixes for Slovenia
    matches!(
        &s[..3.min(s.len())],
        "030" | "031" | "040" | "041" | "051" | "068" | "069" | "070" | "071"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_si_phone_from_unknown_format() {
        let cases = HashMap::from([
            // International format - mobile (A1 Slovenia)
            ("+386 30 123 456", "+386 30 123456"),
            ("+386 40 123 456", "+386 40 123456"),
            ("+38630123456", "+386 30 123456"),
            ("00386 30 123 456", "+386 30 123456"),
            ("0038630123456", "+386 30 123456"),
            // International format - mobile (Telekom Slovenije)
            ("+386 31 234 567", "+386 31 234567"),
            ("+386 41 234 567", "+386 41 234567"),
            ("+386 51 234 567", "+386 51 234567"),
            ("+38631234567", "+386 31 234567"),
            ("00386 31 234 567", "+386 31 234567"),
            ("0038631234567", "+386 31 234567"),
            // International format - mobile (other)
            ("+386 68 123 456", "+386 68 123456"),
            ("+386 69 123 456", "+386 69 123456"),
            ("+386 70 123 456", "+386 70 123456"),
            ("+386 71 123 456", "+386 71 123456"),
            // International format - Ljubljana (area code 1)
            ("+386 1 234 5678", "+386 1 2345678"),
            ("+386 1 234 56 78", "+386 1 2345678"),
            ("+38612345678", "+386 1 2345678"),
            ("00386 1 234 5678", "+386 1 2345678"),
            ("0038612345678", "+386 1 2345678"),
            // International format - regional (2-digit area codes)
            ("+386 2 123 456", "+386 2 123456"), // Maribor
            ("+386 3 123 456", "+386 3 123456"), // Celje
            ("+386 4 123 456", "+386 4 123456"), // Kranj
            ("+386 5 123 456", "+386 5 123456"), // Koper
            ("+386 7 123 456", "+386 7 123456"), // Novo Mesto
            // Local format - mobile
            ("030 123 456", "+386 30 123456"),
            ("030123456", "+386 30 123456"),
            ("031 234 567", "+386 31 234567"),
            ("041 234 567", "+386 41 234567"),
            ("051 234 567", "+386 51 234567"),
            ("068 123 456", "+386 68 123456"),
            ("069 123 456", "+386 69 123456"),
            ("070 123 456", "+386 70 123456"),
            ("071 123 456", "+386 71 123456"),
            // Local format - Ljubljana (area code 1)
            ("01 234 5678", "+386 1 2345678"),
            ("01 234 56 78", "+386 1 2345678"),
            ("012345678", "+386 1 2345678"),
            // Local format - regional
            ("02 123 456", "+386 2 123456"), // Maribor
            ("03 123 456", "+386 3 123456"), // Celje
            ("04 123 456", "+386 4 123456"), // Kranj
            ("05 123 456", "+386 5 123456"), // Koper
            ("07 123 456", "+386 7 123456"), // Novo Mesto
        ]);

        for (input, expected) in cases {
            let phone = si_phone_from_unknown_format(input)
                .unwrap_or_else(|e| panic!("Failed to parse '{}': {:?}", input, e));
            let result = phone.format_international().unwrap();
            assert_eq!(result, expected, "Wrong output for input '{}'", input);
        }
    }

    #[test]
    fn test_si_mobile_prefix_detection() {
        assert!(is_mobile_prefix("030123456"));
        assert!(is_mobile_prefix("031234567"));
        assert!(is_mobile_prefix("040123456"));
        assert!(is_mobile_prefix("041234567"));
        assert!(is_mobile_prefix("051234567"));
        assert!(is_mobile_prefix("068123456"));
        assert!(is_mobile_prefix("069123456"));
        assert!(is_mobile_prefix("070123456"));
        assert!(is_mobile_prefix("071123456"));

        // Not mobile prefixes
        assert!(!is_mobile_prefix("12345678"));
        assert!(!is_mobile_prefix("2123456"));
        assert!(!is_mobile_prefix("3123456"));
    }
}
