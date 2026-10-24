use chrono::{DateTime as ChronoDateTime, TimeZone, Utc};
use everyday::prelude::DateTime;
use serde::{Deserialize, Serialize};

/// Infrastructure wrapper around an external UTC date-time representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppDateTime(ChronoDateTime<Utc>);

impl AppDateTime {
    /// Access the wrapped `chrono` value.
    pub fn inner(&self) -> &ChronoDateTime<Utc> {
        &self.0
    }
}

impl From<&DateTime> for AppDateTime {
    fn from(value: &DateTime) -> Self {
        let timestamp = value.timestamp();
        let chrono_dt = Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .expect("valid unix timestamp");
        Self(chrono_dt)
    }
}

impl From<DateTime> for AppDateTime {
    fn from(value: DateTime) -> Self {
        Self::from(&value)
    }
}

impl From<&AppDateTime> for DateTime {
    fn from(value: &AppDateTime) -> Self {
        DateTime::from_timestamp(value.0.timestamp())
    }
}

impl From<AppDateTime> for DateTime {
    fn from(value: AppDateTime) -> Self {
        (&value).into()
    }
}

#[cfg(test)]
mod tests {
    use super::AppDateTime;
    use chrono::TimeZone;
    use everyday::prelude::DateTime;
    use serde_json::json;

    #[test]
    fn converts_from_domain() {
        let domain = DateTime::from_timestamp(1_705_314_600);
        let infra = AppDateTime::from(&domain);
        assert_eq!(infra.0.timestamp(), domain.timestamp());
    }

    #[test]
    fn converts_from_domain_by_value() {
        let domain = DateTime::from_timestamp(1_705_314_600);
        let infra: AppDateTime = domain.into();
        assert_eq!(infra.0.timestamp(), 1_705_314_600);
    }

    #[test]
    fn converts_to_domain() {
        let chrono_dt = chrono::Utc.timestamp_opt(1_705_314_600, 0).unwrap();
        let infra = AppDateTime(chrono_dt);
        let domain: DateTime = (&infra).into();
        assert_eq!(domain.timestamp(), 1_705_314_600);
    }

    #[test]
    fn converts_to_domain_by_value() {
        let chrono_dt = chrono::Utc.timestamp_opt(1_705_314_600, 0).unwrap();
        let infra = AppDateTime(chrono_dt);
        let domain: DateTime = infra.into();
        assert_eq!(domain.timestamp(), 1_705_314_600);
    }

    #[test]
    fn serializes_as_iso8601_string() {
        let infra = AppDateTime(chrono::Utc.timestamp_opt(1_705_314_600, 0).unwrap());
        let serialized = serde_json::to_value(&infra).unwrap();
        assert_eq!(serialized, json!("2024-01-15T10:30:00Z"));
    }

    #[test]
    fn deserializes_from_iso8601_string() {
        let value = json!("2024-01-15T10:30:00Z");
        let infra: AppDateTime = serde_json::from_value(value).unwrap();
        assert_eq!(infra.0.timestamp(), 1_705_314_600);
    }
}
