//! Days of the week.
//!
//! This module provides [`Weekday`] for representing days of the week.

use super::error::Error;

/// An enumeration of the seven days of the week.
///
/// The week starts on Monday with ordinal 0, following ISO 8601 convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

impl Weekday {
    /// Returns the English name of the weekday.
    pub fn as_str(&self) -> &'static str {
        match self {
            Weekday::Monday => "Monday",
            Weekday::Tuesday => "Tuesday",
            Weekday::Wednesday => "Wednesday",
            Weekday::Thursday => "Thursday",
            Weekday::Friday => "Friday",
            Weekday::Saturday => "Saturday",
            Weekday::Sunday => "Sunday",
        }
    }

    /// Creates a weekday from Sakamoto's algorithm output (0 = Sunday, 6 = Saturday).
    ///
    /// Converts to ISO 8601 ordinals (0 = Monday, 6 = Sunday).
    ///
    /// # Errors
    ///
    /// Returns an error if `sakamoto_ordinal` is not in the range 0–6.
    pub(crate) fn from_sakamoto(sakamoto_ordinal: u8) -> Result<Self, Error> {
        // Sakamoto: 0=Sun, 1=Mon, 2=Tue, 3=Wed, 4=Thu, 5=Fri, 6=Sat
        // ISO 8601: 0=Mon, 1=Tue, 2=Wed, 3=Thu, 4=Fri, 5=Sat, 6=Sun
        match sakamoto_ordinal {
            0 => Ok(Weekday::Sunday),
            1 => Ok(Weekday::Monday),
            2 => Ok(Weekday::Tuesday),
            3 => Ok(Weekday::Wednesday),
            4 => Ok(Weekday::Thursday),
            5 => Ok(Weekday::Friday),
            6 => Ok(Weekday::Saturday),
            _ => Err(Error::InvalidOrdinal(sakamoto_ordinal)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_returns_correct_names() {
        assert_eq!(Weekday::Monday.as_str(), "Monday");
        assert_eq!(Weekday::Tuesday.as_str(), "Tuesday");
        assert_eq!(Weekday::Wednesday.as_str(), "Wednesday");
        assert_eq!(Weekday::Thursday.as_str(), "Thursday");
        assert_eq!(Weekday::Friday.as_str(), "Friday");
        assert_eq!(Weekday::Saturday.as_str(), "Saturday");
        assert_eq!(Weekday::Sunday.as_str(), "Sunday");
    }

    #[test]
    fn weekday_equality() {
        assert_eq!(Weekday::Monday, Weekday::Monday);
        assert_ne!(Weekday::Monday, Weekday::Tuesday);
    }

    #[test]
    fn weekday_is_copy() {
        let day = Weekday::Friday;
        let copy = day;
        assert_eq!(day, copy);
    }

    #[test]
    fn from_sakamoto_maps_correctly() {
        // Sakamoto output: 0=Sun, 1=Mon, 2=Tue, etc.
        assert_eq!(Weekday::from_sakamoto(0).unwrap(), Weekday::Sunday);
        assert_eq!(Weekday::from_sakamoto(1).unwrap(), Weekday::Monday);
        assert_eq!(Weekday::from_sakamoto(2).unwrap(), Weekday::Tuesday);
        assert_eq!(Weekday::from_sakamoto(3).unwrap(), Weekday::Wednesday);
        assert_eq!(Weekday::from_sakamoto(4).unwrap(), Weekday::Thursday);
        assert_eq!(Weekday::from_sakamoto(5).unwrap(), Weekday::Friday);
        assert_eq!(Weekday::from_sakamoto(6).unwrap(), Weekday::Saturday);
    }

    #[test]
    fn from_sakamoto_rejects_invalid_ordinal() {
        assert_eq!(
            Weekday::from_sakamoto(7).unwrap_err(),
            Error::InvalidOrdinal(7)
        );
        assert_eq!(
            Weekday::from_sakamoto(255).unwrap_err(),
            Error::InvalidOrdinal(255)
        );
    }

    #[test]
    fn ordinals_follow_iso_8601() {
        assert_eq!(Weekday::Monday as u8, 0);
        assert_eq!(Weekday::Tuesday as u8, 1);
        assert_eq!(Weekday::Wednesday as u8, 2);
        assert_eq!(Weekday::Thursday as u8, 3);
        assert_eq!(Weekday::Friday as u8, 4);
        assert_eq!(Weekday::Saturday as u8, 5);
        assert_eq!(Weekday::Sunday as u8, 6);
    }
}
