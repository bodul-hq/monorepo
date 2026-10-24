//! Error types for business operations.
//!
//! This module defines a unified error enum for business-layer
//! concepts such as money and currency.

use core::fmt;

use super::money::MoneyError;

/// Errors that can occur when working with business concepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Errors originating from money operations.
    Money(MoneyError),
}

/// Error type for phone number operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhoneNumberError {
    /// Area code length is invalid (not 1–2 characters)
    InvalidAreaCodeLength(String),
    /// Area code contains characters other than digits
    InvalidAreaCodeCharacters(String),
    /// Subscriber number length is invalid (not 6–7 characters)
    InvalidNumberLength(String),
    /// Subscriber number contains characters other than digits
    InvalidNumberCharacters(String),
    /// International string does not match the expected pattern
    InvalidInternationalFormat(String),
    /// Local string does not match the expected pattern
    InvalidLocalFormat(String),
    /// An international calling code is not defined for the given country
    UnsupportedCountryForInternationalFormat,
}

impl fmt::Display for PhoneNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAreaCodeLength(value) => {
                write!(f, "area code must be 1-2 digits, got: '{}'", value)
            }
            Self::InvalidAreaCodeCharacters(value) => write!(
                f,
                "area code must contain only digits and not start with 0, got: '{}'",
                value
            ),
            Self::InvalidNumberLength(value) => {
                write!(f, "subscriber number must be 6-7 digits, got: '{}'", value)
            }
            Self::InvalidNumberCharacters(value) => write!(
                f,
                "subscriber number must contain only digits and not start with 0, got: '{}'",
                value
            ),
            Self::InvalidInternationalFormat(value) => {
                write!(f, "invalid international phone number format: '{}'", value)
            }
            Self::InvalidLocalFormat(value) => {
                write!(f, "invalid local phone number format: '{}'", value)
            }
            Self::UnsupportedCountryForInternationalFormat => {
                write!(f, "country not supported for international format")
            }
        }
    }
}

impl std::error::Error for PhoneNumberError {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Money(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Money(err) => Some(err),
        }
    }
}

impl From<MoneyError> for Error {
    fn from(err: MoneyError) -> Self {
        Error::Money(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_delegates_to_inner_error() {
        let inner = MoneyError::Overflow;
        let err = Error::Money(inner.clone());
        assert_eq!(err.to_string(), inner.to_string());
    }

    #[test]
    fn from_money_error_creates_money_variant() {
        let inner = MoneyError::Underflow;
        let err: Error = inner.clone().into();
        assert_eq!(err, Error::Money(inner));
    }
}
