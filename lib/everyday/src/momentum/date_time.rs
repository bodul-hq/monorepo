use core::fmt;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// UTC date and time represented as seconds from the Unix epoch.
///
/// This type intentionally avoids external dependencies and sticks to simple,
/// comparable semantics defined in TS-002. It is immutable and provides
/// construction helpers along with a human-readable ISO 8601 formatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DateTime(i64);

/// Sentinel timestamp for 1970-02-02T00:00:00Z, returned when system time
/// cannot be determined. This date is easily recognizable as an error
/// indicator while remaining a valid timestamp.
const SENTINEL_TIMESTAMP: i64 = 2_764_800; // 32 days * 86400 seconds

impl DateTime {
    /// Returns the current UTC time.
    ///
    /// If the system clock cannot be read (e.g., time is before Unix epoch),
    /// returns `1970-02-02T00:00:00Z` as a sentinel value to indicate an error
    /// without panicking.
    pub fn now() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(SENTINEL_TIMESTAMP as u64));
        Self::from_timestamp(now.as_secs() as i64)
    }

    /// Creates a [`DateTime`] from a Unix timestamp expressed in seconds.
    pub fn from_timestamp(seconds: i64) -> Self {
        Self(seconds)
    }

    /// Returns the underlying Unix timestamp in seconds.
    pub fn timestamp(&self) -> i64 {
        self.0
    }

    /// Returns `true` if this is the sentinel value (`1970-02-02T00:00:00Z`).
    ///
    /// The sentinel value indicates that the system time could not be read
    /// when `now()` was called.
    pub fn is_sentinel(&self) -> bool {
        self.0 == SENTINEL_TIMESTAMP
    }

    fn components(self) -> (i32, i32, i32, i32, i32, i32) {
        const SECONDS_PER_DAY: i64 = 86_400;
        let days = self.0.div_euclid(SECONDS_PER_DAY);
        let mut seconds_of_day = self.0.rem_euclid(SECONDS_PER_DAY);

        let hour = (seconds_of_day / 3_600) as i32;
        seconds_of_day %= 3_600;
        let minute = (seconds_of_day / 60) as i32;
        let second = (seconds_of_day % 60) as i32;

        let (year, month, day) = civil_from_days(days);
        (year, month, day, hour, minute, second)
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (year, month, day, hour, minute, second) = self.components();
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
        )
    }
}

/// Converts a number of days since the Unix epoch to a calendar date in the
/// proleptic Gregorian calendar.
fn civil_from_days(days: i64) -> (i32, i32, i32) {
    // Algorithm from Howard Hinnant's civil calendar routines.
    let z = days + 719_468;
    let era = if z >= 0 {
        z / 146_097
    } else {
        (z - 146_096) / 146_097
    };
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let month = mp + if mp < 10 { 3 } else { -9 }; // [1, 12]
    let year = y + if month <= 2 { 1 } else { 0 };
    (year as i32, month as i32, day as i32)
}

#[cfg(test)]
mod tests {
    use super::{DateTime, SENTINEL_TIMESTAMP};

    #[test]
    fn constructs_from_timestamp() {
        let dt = DateTime::from_timestamp(1_705_314_600);
        assert_eq!(dt.timestamp(), 1_705_314_600);
    }

    #[test]
    fn formats_iso_8601_in_utc() {
        let dt = DateTime::from_timestamp(1_705_314_600); // 2024-01-15T10:30:00Z
        assert_eq!(dt.to_string(), "2024-01-15T10:30:00Z");
    }

    #[test]
    fn handles_negative_timestamps() {
        let dt = DateTime::from_timestamp(-1);
        assert_eq!(dt.to_string(), "1969-12-31T23:59:59Z");
    }

    #[test]
    fn orders_compare_by_timestamp() {
        let earlier = DateTime::from_timestamp(1_000);
        let later = DateTime::from_timestamp(2_000);
        assert!(earlier < later);
        assert!(later > earlier);
        assert_eq!(earlier, earlier);
    }

    #[test]
    fn sentinel_value_is_1970_02_02() {
        let dt = DateTime::from_timestamp(SENTINEL_TIMESTAMP);
        assert_eq!(dt.to_string(), "1970-02-02T00:00:00Z");
        assert!(dt.is_sentinel());
    }

    #[test]
    fn non_sentinel_value_returns_false() {
        let dt = DateTime::from_timestamp(0);
        assert!(!dt.is_sentinel());

        let dt = DateTime::from_timestamp(1_705_314_600);
        assert!(!dt.is_sentinel());
    }
}
