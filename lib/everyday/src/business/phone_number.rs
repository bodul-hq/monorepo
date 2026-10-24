//! Phone number representation with country-aware formatting.
//!
//! Provides a structured phone number with support for international and local formatting,
//! initially focused on Croatian phone numbers.

use crate::country_code::CountryCode;
use serde::{Deserialize, Serialize};

pub use super::error::PhoneNumberError;

/// PhoneNumber represents a contact phone number with country-aware formatting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhoneNumber {
    country: CountryCode,
    area_code: String,
    number: String,
}

impl PhoneNumber {
    /// Creates a new PhoneNumber from normalized components.
    ///
    /// Validates that `area_code` and `number` satisfy the invariants:
    /// - area_code: 1–3 characters, digits only, standard codes don't start with '0'
    /// - number: 6–7 characters, digits only, standard numbers don't start with '0'
    ///
    /// Special service numbers (e.g., 0800) have relaxed validation rules.
    pub fn new(
        country: CountryCode,
        area_code: String,
        number: String,
    ) -> Result<PhoneNumber, PhoneNumberError> {
        validate_area_code(&area_code)?;
        validate_number(&number, &area_code)?;

        Ok(PhoneNumber {
            country,
            area_code,
            number,
        })
    }

    /// Parses a phone number from international format.
    ///
    /// Supports:
    /// - Croatian numbers: `+385 {area_code} {number}`
    /// - Slovenian numbers: `+386 {area_code} {number}`
    pub fn from_international(
        input: &str,
        country: CountryCode,
    ) -> Result<PhoneNumber, PhoneNumberError> {
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

        // Verify country code matches the expected calling code
        let expected_calling_code = match country.as_str() {
            "HR" => "+385",
            "SI" => "+386",
            _ => return Err(PhoneNumberError::UnsupportedCountryForInternationalFormat),
        };

        if parts[0] != expected_calling_code {
            return Err(PhoneNumberError::InvalidInternationalFormat(
                trimmed.to_string(),
            ));
        }

        let area_code = parts[1].to_string();
        let number = parts[2].to_string();

        Self::new(country, area_code, number)
    }

    /// Parses a phone number from local format.
    ///
    /// For Croatian numbers, expects format: `0{area_code} {number}`
    pub fn from_local(input: &str, country: CountryCode) -> Result<PhoneNumber, PhoneNumberError> {
        let trimmed = input.trim();

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(PhoneNumberError::InvalidLocalFormat(trimmed.to_string()));
        }

        let first_part = parts[0];
        if !first_part.starts_with('0') || first_part.len() < 2 {
            return Err(PhoneNumberError::InvalidLocalFormat(trimmed.to_string()));
        }

        let area_code = first_part[1..].to_string();
        let number = parts[1].to_string();

        Self::new(country, area_code, number)
    }

    /// Formats the phone number to international format.
    ///
    /// For Croatian numbers, produces: `+385 {area_code} {number}`
    pub fn format_international(&self) -> Result<String, PhoneNumberError> {
        match self.country.as_str() {
            "HR" => Ok(format!("+385 {} {}", self.area_code, self.number)),
            "SI" => Ok(format!("+386 {} {}", self.area_code, self.number)),
            _ => Err(PhoneNumberError::UnsupportedCountryForInternationalFormat),
        }
    }

    /// Formats the phone number to local format.
    ///
    /// For Croatian numbers, produces: `0{area_code} {number}`
    pub fn format_local(&self) -> String {
        format!("0{} {}", self.area_code, self.number)
    }

    /// Returns the country code.
    pub fn country(&self) -> CountryCode {
        self.country
    }

    /// Returns the area code.
    pub fn area_code(&self) -> &str {
        &self.area_code
    }

    /// Returns the subscriber number.
    pub fn number(&self) -> &str {
        &self.number
    }
}

/// Validates that the area code satisfies the invariants.
fn validate_area_code(area_code: &str) -> Result<(), PhoneNumberError> {
    let len = area_code.len();

    // Special service numbers (e.g., 800 for toll-free) can have 3 digits
    if !(1..=3).contains(&len) {
        return Err(PhoneNumberError::InvalidAreaCodeLength(
            area_code.to_string(),
        ));
    }

    if !area_code.chars().all(|c| c.is_ascii_digit()) {
        return Err(PhoneNumberError::InvalidAreaCodeCharacters(
            area_code.to_string(),
        ));
    }

    // Standard area codes shouldn't start with 0
    // But special service codes like "800" are allowed
    if len <= 2 && area_code.starts_with('0') {
        return Err(PhoneNumberError::InvalidAreaCodeCharacters(
            area_code.to_string(),
        ));
    }

    Ok(())
}

/// Validates that the subscriber number satisfies the invariants.
fn validate_number(number: &str, area_code: &str) -> Result<(), PhoneNumberError> {
    let len = number.len();

    // Special service numbers (800, 801) can have 4-7 digit subscribers
    // Standard numbers must have 6-7 digit subscribers
    let is_special_service = area_code == "800" || area_code == "801";
    let valid_length = if is_special_service {
        (4..=7).contains(&len)
    } else {
        (6..=7).contains(&len)
    };

    if !valid_length {
        return Err(PhoneNumberError::InvalidNumberLength(number.to_string()));
    }

    if !number.chars().all(|c| c.is_ascii_digit()) {
        return Err(PhoneNumberError::InvalidNumberCharacters(
            number.to_string(),
        ));
    }

    // Standard numbers shouldn't start with 0
    // But special service numbers (e.g., toll-free 0800) may have different patterns
    if !is_special_service && number.starts_with('0') {
        return Err(PhoneNumberError::InvalidNumberCharacters(
            number.to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid_hr_number() {
        let hr = CountryCode::HR;
        let result = PhoneNumber::new(hr, "99".to_string(), "1234567".to_string());
        assert!(result.is_ok());

        let phone = result.unwrap();
        assert_eq!(phone.area_code(), "99");
        assert_eq!(phone.number(), "1234567");
        assert_eq!(phone.country(), hr);
    }

    #[test]
    fn test_format_international() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "99".to_string(), "1234567".to_string()).unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 99 1234567");
    }

    #[test]
    fn test_format_local() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "99".to_string(), "1234567".to_string()).unwrap();
        assert_eq!(phone.format_local(), "099 1234567");
    }

    #[test]
    fn test_from_international() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::from_international("+385 99 1234567", hr).unwrap();
        assert_eq!(phone.area_code(), "99");
        assert_eq!(phone.number(), "1234567");
    }

    #[test]
    fn test_from_local() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::from_local("099 1234567", hr).unwrap();
        assert_eq!(phone.area_code(), "99");
        assert_eq!(phone.number(), "1234567");
    }

    #[test]
    fn test_invalid_area_code_length() {
        let hr = CountryCode::HR;
        // Area codes must be 1-3 digits, so 4 digits should fail
        let result = PhoneNumber::new(hr, "9999".to_string(), "1234567".to_string());
        assert_eq!(
            result,
            Err(PhoneNumberError::InvalidAreaCodeLength("9999".to_string()))
        );
    }

    #[test]
    fn test_invalid_area_code_starting_with_zero() {
        let hr = CountryCode::HR;
        let result = PhoneNumber::new(hr, "09".to_string(), "1234567".to_string());
        assert_eq!(
            result,
            Err(PhoneNumberError::InvalidAreaCodeCharacters(
                "09".to_string()
            ))
        );
    }

    #[test]
    fn test_invalid_number_length() {
        let hr = CountryCode::HR;
        let result = PhoneNumber::new(hr, "99".to_string(), "12345".to_string());
        assert_eq!(
            result,
            Err(PhoneNumberError::InvalidNumberLength("12345".to_string()))
        );
    }

    #[test]
    fn test_invalid_number_starting_with_zero() {
        let hr = CountryCode::HR;
        let result = PhoneNumber::new(hr, "99".to_string(), "0234567".to_string());
        assert_eq!(
            result,
            Err(PhoneNumberError::InvalidNumberCharacters(
                "0234567".to_string()
            ))
        );
    }

    #[test]
    fn test_invalid_international_format_missing_plus() {
        let hr = CountryCode::HR;
        let result = PhoneNumber::from_international("385 99 1234567", hr);
        assert_eq!(
            result,
            Err(PhoneNumberError::InvalidInternationalFormat(
                "385 99 1234567".to_string()
            ))
        );
    }

    #[test]
    fn test_invalid_international_format_wrong_parts() {
        let hr = CountryCode::HR;
        let result = PhoneNumber::from_international("+385 99", hr);
        assert_eq!(
            result,
            Err(PhoneNumberError::InvalidInternationalFormat(
                "+385 99".to_string()
            ))
        );
    }

    #[test]
    fn test_invalid_local_format_missing_zero() {
        let hr = CountryCode::HR;
        let result = PhoneNumber::from_local("99 1234567", hr);
        assert_eq!(
            result,
            Err(PhoneNumberError::InvalidLocalFormat(
                "99 1234567".to_string()
            ))
        );
    }

    #[test]
    fn test_invalid_local_format_wrong_parts() {
        let hr = CountryCode::HR;
        let result = PhoneNumber::from_local("099", hr);
        assert_eq!(
            result,
            Err(PhoneNumberError::InvalidLocalFormat("099".to_string()))
        );
    }

    #[test]
    fn test_single_digit_area_code() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "9".to_string(), "1234567".to_string()).unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 9 1234567");
        assert_eq!(phone.format_local(), "09 1234567");
    }

    #[test]
    fn test_zagreb_area_code_1() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "1".to_string(), "1234567".to_string()).unwrap();
        assert_eq!(phone.area_code(), "1");
        assert_eq!(phone.number(), "1234567");
    }

    #[test]
    fn test_zagreb_format_international() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "1".to_string(), "1234567".to_string()).unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 1 1234567");
    }

    #[test]
    fn test_zagreb_format_local() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "1".to_string(), "1234567".to_string()).unwrap();
        assert_eq!(phone.format_local(), "01 1234567");
    }

    #[test]
    fn test_zagreb_from_international() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::from_international("+385 1 1234567", hr).unwrap();
        assert_eq!(phone.area_code(), "1");
        assert_eq!(phone.number(), "1234567");
    }

    #[test]
    fn test_zagreb_from_local() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::from_local("01 1234567", hr).unwrap();
        assert_eq!(phone.area_code(), "1");
        assert_eq!(phone.number(), "1234567");
    }

    #[test]
    fn test_zagreb_roundtrip_international() {
        let hr = CountryCode::HR;
        let original = "+385 1 1234567";
        let phone = PhoneNumber::from_international(original, hr).unwrap();
        let formatted = phone.format_international().unwrap();
        assert_eq!(formatted, original);
    }

    #[test]
    fn test_zagreb_roundtrip_local() {
        let hr = CountryCode::HR;
        let original = "01 1234567";
        let phone = PhoneNumber::from_local(original, hr).unwrap();
        let formatted = phone.format_local();
        assert_eq!(formatted, original);
    }

    #[test]
    fn test_zagreb_roundtrip_conversion() {
        let hr = CountryCode::HR;
        let phone_intl = PhoneNumber::from_international("+385 1 1234567", hr).unwrap();
        let phone_local = PhoneNumber::from_local("01 1234567", hr).unwrap();

        // Both should represent the same number
        assert_eq!(phone_intl.area_code(), phone_local.area_code());
        assert_eq!(phone_intl.number(), phone_local.number());

        // Formatters should produce consistent results
        assert_eq!(phone_intl.format_local(), phone_local.format_local());
        assert_eq!(
            phone_intl.format_international().unwrap(),
            phone_local.format_international().unwrap()
        );
    }

    #[test]
    fn test_zagreb_with_max_length_number() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "1".to_string(), "9999999".to_string()).unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 1 9999999");
        assert_eq!(phone.format_local(), "01 9999999");
    }

    #[test]
    fn test_zagreb_with_min_length_number() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "1".to_string(), "123456".to_string()).unwrap();
        assert_eq!(phone.format_international().unwrap(), "+385 1 123456");
        assert_eq!(phone.format_local(), "01 123456");
    }

    #[test]
    fn test_toll_free_number_6_digits() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "800".to_string(), "123456".to_string()).unwrap();
        assert_eq!(phone.area_code(), "800");
        assert_eq!(phone.number(), "123456");
        assert_eq!(phone.format_international().unwrap(), "+385 800 123456");
        assert_eq!(phone.format_local(), "0800 123456");
    }

    #[test]
    fn test_toll_free_number_7_digits() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "800".to_string(), "4000000".to_string()).unwrap();
        assert_eq!(phone.area_code(), "800");
        assert_eq!(phone.number(), "4000000");
        assert_eq!(phone.format_international().unwrap(), "+385 800 4000000");
        assert_eq!(phone.format_local(), "0800 4000000");
    }

    #[test]
    fn test_toll_free_number_4_digits() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "800".to_string(), "0801".to_string()).unwrap();
        assert_eq!(phone.area_code(), "800");
        assert_eq!(phone.number(), "0801");
        assert_eq!(phone.format_international().unwrap(), "+385 800 0801");
        assert_eq!(phone.format_local(), "0800 0801");
    }

    #[test]
    fn test_toll_free_801_number_4_digits() {
        let hr = CountryCode::HR;
        let phone = PhoneNumber::new(hr, "801".to_string(), "1234".to_string()).unwrap();
        assert_eq!(phone.area_code(), "801");
        assert_eq!(phone.number(), "1234");
        assert_eq!(phone.format_international().unwrap(), "+385 801 1234");
        assert_eq!(phone.format_local(), "0801 1234");
    }

    // Slovenia tests

    #[test]
    fn test_si_from_international_mobile() {
        let si = CountryCode::SI;
        let phone = PhoneNumber::from_international("+386 31 234567", si).unwrap();
        assert_eq!(phone.area_code(), "31");
        assert_eq!(phone.number(), "234567");
        assert_eq!(phone.format_international().unwrap(), "+386 31 234567");
    }

    #[test]
    fn test_si_from_international_ljubljana() {
        let si = CountryCode::SI;
        let phone = PhoneNumber::from_international("+386 1 2345678", si).unwrap();
        assert_eq!(phone.area_code(), "1");
        assert_eq!(phone.number(), "2345678");
        assert_eq!(phone.format_international().unwrap(), "+386 1 2345678");
    }

    #[test]
    fn test_si_from_international_regional() {
        let si = CountryCode::SI;
        let phone = PhoneNumber::from_international("+386 2 123456", si).unwrap();
        assert_eq!(phone.area_code(), "2");
        assert_eq!(phone.number(), "123456");
        assert_eq!(phone.format_international().unwrap(), "+386 2 123456");
    }

    #[test]
    fn test_si_from_international_wrong_country_code() {
        let si = CountryCode::SI;
        let result = PhoneNumber::from_international("+385 1 2345678", si);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PhoneNumberError::InvalidInternationalFormat(_)
        ));
    }

    #[test]
    fn test_si_format_international() {
        let si = CountryCode::SI;
        let phone = PhoneNumber::new(si, "31".to_string(), "234567".to_string()).unwrap();
        assert_eq!(phone.format_international().unwrap(), "+386 31 234567");
    }
}
