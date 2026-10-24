//! Simple calendar date with year, month, and day components.
//!
//! This module provides [`Date`] for representing calendar dates.

use core::fmt;
use std::str::FromStr;

use super::error::Error;
use super::weekday::Weekday;
use serde::{Deserialize, Serialize};

/// A lightweight calendar date with year, month, and day components.
///
/// This type supports basic range validation but does not validate days against
/// specific months or leap years.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    /// Creates a new date with basic validation.
    ///
    /// # Errors
    ///
    /// Returns an error if month is not in 1–12 or day is not in 1–31.
    /// Does not validate day against the specific month or leap years.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, Error> {
        if !(1..=12).contains(&month) {
            return Err(Error::InvalidMonth(month));
        }
        if !(1..=31).contains(&day) {
            return Err(Error::InvalidDay(day));
        }
        Ok(Self { year, month, day })
    }

    /// Parses a date from strict `yyyy-mm-dd` format.
    ///
    /// # Errors
    ///
    /// Returns an error if the input does not match the expected format.
    pub fn from_string(s: &str) -> Result<Self, Error> {
        if s.len() != 10 {
            return Err(Error::InvalidFormat(s.to_string()));
        }

        let bytes = s.as_bytes();
        if bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(Error::InvalidFormat(s.to_string()));
        }

        let year_str = &s[0..4];
        let month_str = &s[5..7];
        let day_str = &s[8..10];

        if !year_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::InvalidFormat(s.to_string()));
        }
        if !month_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::InvalidFormat(s.to_string()));
        }
        if !day_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::InvalidFormat(s.to_string()));
        }

        let year: i32 = year_str
            .parse()
            .map_err(|_| Error::InvalidFormat(s.to_string()))?;
        let month: u8 = month_str
            .parse()
            .map_err(|_| Error::InvalidFormat(s.to_string()))?;
        let day: u8 = day_str
            .parse()
            .map_err(|_| Error::InvalidFormat(s.to_string()))?;

        Self::new(year, month, day)
    }

    /// Returns the year component.
    pub fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month component (1–12).
    pub fn month(&self) -> u8 {
        self.month
    }

    /// Returns the day component (1–31).
    pub fn day(&self) -> u8 {
        self.day
    }

    /// Computes the day of week using Sakamoto's algorithm.
    ///
    /// Accurate for dates in the Gregorian calendar (after 1582).
    pub fn weekday(&self) -> Weekday {
        // Sakamoto's algorithm
        const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];

        let mut y = self.year;
        if self.month < 3 {
            y -= 1;
        }

        let ordinal =
            (y + y / 4 - y / 100 + y / 400 + T[(self.month - 1) as usize] + self.day as i32) % 7;

        // rem_euclid(7) guarantees result is 0-6, so this cannot fail
        Weekday::from_sakamoto(ordinal.rem_euclid(7) as u8)
            .expect("rem_euclid(7) always produces 0-6")
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for Date {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod date_construction_tests {
        use super::*;

        #[test]
        fn new_accepts_valid_dates() {
            let date = Date::new(2024, 3, 15).unwrap();
            assert_eq!(date.year(), 2024);
            assert_eq!(date.month(), 3);
            assert_eq!(date.day(), 15);
        }

        #[test]
        fn new_rejects_month_zero() {
            let result = Date::new(2024, 0, 15);
            assert_eq!(result.unwrap_err(), Error::InvalidMonth(0));
        }

        #[test]
        fn new_rejects_month_thirteen() {
            let result = Date::new(2024, 13, 15);
            assert_eq!(result.unwrap_err(), Error::InvalidMonth(13));
        }

        #[test]
        fn new_rejects_day_zero() {
            let result = Date::new(2024, 3, 0);
            assert_eq!(result.unwrap_err(), Error::InvalidDay(0));
        }

        #[test]
        fn new_rejects_day_thirty_two() {
            let result = Date::new(2024, 3, 32);
            assert_eq!(result.unwrap_err(), Error::InvalidDay(32));
        }

        #[test]
        fn new_accepts_boundary_values() {
            assert!(Date::new(2024, 1, 1).is_ok());
            assert!(Date::new(2024, 12, 31).is_ok());
        }

        #[test]
        fn new_accepts_invalid_calendar_dates() {
            // Does not validate day against month
            assert!(Date::new(2024, 2, 31).is_ok());
            assert!(Date::new(2024, 4, 31).is_ok());
        }
    }

    mod date_parsing_tests {
        use super::*;

        #[test]
        fn from_string_parses_valid_date() {
            let date = Date::from_string("2024-03-15").unwrap();
            assert_eq!(date.year(), 2024);
            assert_eq!(date.month(), 3);
            assert_eq!(date.day(), 15);
        }

        #[test]
        fn from_string_rejects_wrong_length() {
            let result = Date::from_string("2024-3-15");
            assert!(matches!(result.unwrap_err(), Error::InvalidFormat(_)));
        }

        #[test]
        fn from_string_rejects_wrong_separators() {
            let result = Date::from_string("2024/03/15");
            assert!(matches!(result.unwrap_err(), Error::InvalidFormat(_)));
        }

        #[test]
        fn from_string_rejects_non_digit_year() {
            let result = Date::from_string("20a4-03-15");
            assert!(matches!(result.unwrap_err(), Error::InvalidFormat(_)));
        }

        #[test]
        fn from_string_rejects_non_digit_month() {
            let result = Date::from_string("2024-0a-15");
            assert!(matches!(result.unwrap_err(), Error::InvalidFormat(_)));
        }

        #[test]
        fn from_string_rejects_non_digit_day() {
            let result = Date::from_string("2024-03-1a");
            assert!(matches!(result.unwrap_err(), Error::InvalidFormat(_)));
        }

        #[test]
        fn from_str_trait_works() {
            let date: Date = "2024-03-15".parse().unwrap();
            assert_eq!(date.year(), 2024);
        }
    }

    mod date_formatting_tests {
        use super::*;

        #[test]
        fn display_formats_with_zero_padding() {
            let date = Date::new(2024, 3, 5).unwrap();
            assert_eq!(date.to_string(), "2024-03-05");
        }

        #[test]
        fn display_formats_year_with_leading_zeros() {
            let date = Date::new(1, 1, 1).unwrap();
            assert_eq!(date.to_string(), "0001-01-01");
        }

        #[test]
        fn roundtrip_parse_format() {
            let original = "2024-03-15";
            let date: Date = original.parse().unwrap();
            assert_eq!(date.to_string(), original);
        }
    }

    mod weekday_calculation_tests {
        use super::*;

        #[test]
        fn known_date_monday() {
            // 2024-01-01 was a Monday
            let date = Date::new(2024, 1, 1).unwrap();
            assert_eq!(date.weekday(), Weekday::Monday);
        }

        #[test]
        fn known_date_friday() {
            // 2024-03-15 was a Friday
            let date = Date::new(2024, 3, 15).unwrap();
            assert_eq!(date.weekday(), Weekday::Friday);
        }

        #[test]
        fn known_date_saturday() {
            // 2024-11-30 is a Saturday
            let date = Date::new(2024, 11, 30).unwrap();
            assert_eq!(date.weekday(), Weekday::Saturday);
        }

        #[test]
        fn known_date_sunday() {
            // 2024-12-01 is a Sunday
            let date = Date::new(2024, 12, 1).unwrap();
            assert_eq!(date.weekday(), Weekday::Sunday);
        }

        #[test]
        fn historical_date() {
            // 1776-07-04 was a Thursday
            let date = Date::new(1776, 7, 4).unwrap();
            assert_eq!(date.weekday(), Weekday::Thursday);
        }
    }

    mod error_tests {
        use super::*;

        #[test]
        fn error_display_invalid_month() {
            let err = Error::InvalidMonth(13);
            assert_eq!(err.to_string(), "Invalid month: 13");
        }

        #[test]
        fn error_display_invalid_day() {
            let err = Error::InvalidDay(32);
            assert_eq!(err.to_string(), "Invalid day: 32");
        }

        #[test]
        fn error_display_invalid_format() {
            let err = Error::InvalidFormat("bad".to_string());
            assert_eq!(err.to_string(), "Expected format yyyy-mm-dd, got: bad");
        }
    }
}
