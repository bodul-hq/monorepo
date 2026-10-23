//! Error types for momentum operations.

use core::fmt;

/// Errors that can occur when working with dates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Month value is outside the valid range (1-12).
    InvalidMonth(u8),
    /// Day value is outside the valid range (1-31).
    InvalidDay(u8),
    /// Input string format is invalid.
    InvalidFormat(String),
    /// Weekday ordinal is outside the valid range (0-6).
    InvalidOrdinal(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidMonth(month) => write!(f, "Invalid month: {month}"),
            Error::InvalidDay(day) => write!(f, "Invalid day: {day}"),
            Error::InvalidFormat(input) => {
                write!(f, "Expected format yyyy-mm-dd, got: {input}")
            }
            Error::InvalidOrdinal(ordinal) => write!(f, "Invalid ordinal: {ordinal}"),
        }
    }
}

impl std::error::Error for Error {}
