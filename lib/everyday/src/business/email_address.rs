//! Email address representation.
//!
//! Provides a value object for contact email addresses with basic validation.

use core::fmt;
use serde::{Deserialize, Serialize};

/// Errors that can occur when parsing an email address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailAddressError {
    /// Input does not contain the `@` separator.
    MissingAt,
    /// Input contains more than one `@`.
    MultipleAt,
    /// Characters before `@` are empty after trimming.
    EmptyLocalPart,
    /// Characters after `@` are empty after trimming.
    EmptyDomain,
    /// Local part contains whitespace characters.
    InvalidLocalPart,
    /// Domain contains whitespace, starts or ends with `.`, or has no `.`.
    InvalidDomain,
}

impl fmt::Display for EmailAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailAddressError::MissingAt => write!(f, "email address must contain '@'"),
            EmailAddressError::MultipleAt => {
                write!(f, "email address must contain exactly one '@'")
            }
            EmailAddressError::EmptyLocalPart => {
                write!(f, "email address local part cannot be empty")
            }
            EmailAddressError::EmptyDomain => write!(f, "email address domain cannot be empty"),
            EmailAddressError::InvalidLocalPart => {
                write!(f, "email address local part contains invalid characters")
            }
            EmailAddressError::InvalidDomain => write!(f, "email address domain is invalid"),
        }
    }
}

impl std::error::Error for EmailAddressError {}

/// EmailAddress represents a contact email address.
///
/// The address is stored in normalized form (trimmed and lowercase).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress {
    local_part: String,
    domain: String,
}

impl EmailAddress {
    /// Creates a new EmailAddress from a raw string.
    ///
    /// The input is trimmed and converted to lowercase.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input does not contain exactly one `@`
    /// - Local part is empty or contains whitespace
    /// - Domain is empty, contains whitespace, starts or ends with `.`,
    ///   or does not contain at least one `.`
    pub fn new(email: impl Into<String>) -> Result<Self, EmailAddressError> {
        let email = email.into();
        let trimmed = email.trim();

        let at_count = trimmed.chars().filter(|&c| c == '@').count();
        if at_count == 0 {
            return Err(EmailAddressError::MissingAt);
        }
        if at_count > 1 {
            return Err(EmailAddressError::MultipleAt);
        }

        let (local_part, domain) = trimmed.split_once('@').unwrap();

        if local_part.is_empty() {
            return Err(EmailAddressError::EmptyLocalPart);
        }
        if local_part.chars().any(|c| c.is_whitespace()) {
            return Err(EmailAddressError::InvalidLocalPart);
        }

        if domain.is_empty() {
            return Err(EmailAddressError::EmptyDomain);
        }
        if domain.chars().any(|c| c.is_whitespace()) {
            return Err(EmailAddressError::InvalidDomain);
        }
        if domain.starts_with('.') || domain.ends_with('.') {
            return Err(EmailAddressError::InvalidDomain);
        }
        if !domain.contains('.') {
            return Err(EmailAddressError::InvalidDomain);
        }

        Ok(EmailAddress {
            local_part: local_part.to_lowercase(),
            domain: domain.to_lowercase(),
        })
    }

    /// Returns the full email address as a string.
    pub fn value(&self) -> String {
        format!("{}@{}", self.local_part, self.domain)
    }

    /// Returns the local part (portion before `@`).
    pub fn local_part(&self) -> &str {
        &self.local_part
    }

    /// Returns the domain (portion after `@`).
    pub fn domain(&self) -> &str {
        &self.domain
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.local_part, self.domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid addresses

    #[test]
    fn valid_simple_address() {
        let email = EmailAddress::new("user@example.com").unwrap();
        assert_eq!(email.local_part(), "user");
        assert_eq!(email.domain(), "example.com");
        assert_eq!(email.value(), "user@example.com");
    }

    #[test]
    fn valid_with_trimming_and_lowercase() {
        let email = EmailAddress::new(" User.Name+tag@Example.CoM  ").unwrap();
        assert_eq!(email.local_part(), "user.name+tag");
        assert_eq!(email.domain(), "example.com");
        assert_eq!(email.value(), "user.name+tag@example.com");
    }

    #[test]
    fn valid_uppercase_normalized() {
        let email = EmailAddress::new("USER@EXAMPLE.COM").unwrap();
        assert_eq!(email.value(), "user@example.com");
    }

    #[test]
    fn valid_with_plus_tag() {
        let email = EmailAddress::new("user+tag@example.com").unwrap();
        assert_eq!(email.local_part(), "user+tag");
    }

    #[test]
    fn valid_with_dots_in_local_part() {
        let email = EmailAddress::new("first.last@example.com").unwrap();
        assert_eq!(email.local_part(), "first.last");
    }

    #[test]
    fn valid_with_subdomain() {
        let email = EmailAddress::new("user@mail.example.com").unwrap();
        assert_eq!(email.domain(), "mail.example.com");
    }

    // Invalid addresses

    #[test]
    fn invalid_missing_at() {
        let result = EmailAddress::new("userexample.com");
        assert_eq!(result.unwrap_err(), EmailAddressError::MissingAt);
    }

    #[test]
    fn invalid_multiple_at() {
        let result = EmailAddress::new("user@@example.com");
        assert_eq!(result.unwrap_err(), EmailAddressError::MultipleAt);
    }

    #[test]
    fn invalid_multiple_at_different_positions() {
        let result = EmailAddress::new("user@name@example.com");
        assert_eq!(result.unwrap_err(), EmailAddressError::MultipleAt);
    }

    #[test]
    fn invalid_empty_local_part() {
        let result = EmailAddress::new("@example.com");
        assert_eq!(result.unwrap_err(), EmailAddressError::EmptyLocalPart);
    }

    #[test]
    fn invalid_empty_domain() {
        let result = EmailAddress::new("user@");
        assert_eq!(result.unwrap_err(), EmailAddressError::EmptyDomain);
    }

    #[test]
    fn invalid_whitespace_in_local_part() {
        let result = EmailAddress::new("user name@example.com");
        assert_eq!(result.unwrap_err(), EmailAddressError::InvalidLocalPart);
    }

    #[test]
    fn invalid_whitespace_in_domain() {
        let result = EmailAddress::new("user@ example.com");
        assert_eq!(result.unwrap_err(), EmailAddressError::InvalidDomain);
    }

    #[test]
    fn invalid_domain_without_dot() {
        let result = EmailAddress::new("user@example");
        assert_eq!(result.unwrap_err(), EmailAddressError::InvalidDomain);
    }

    #[test]
    fn invalid_domain_starts_with_dot() {
        let result = EmailAddress::new("user@.example.com");
        assert_eq!(result.unwrap_err(), EmailAddressError::InvalidDomain);
    }

    #[test]
    fn invalid_domain_ends_with_dot() {
        let result = EmailAddress::new("user@example.com.");
        assert_eq!(result.unwrap_err(), EmailAddressError::InvalidDomain);
    }

    // Equality

    #[test]
    fn equality_case_insensitive() {
        let email1 = EmailAddress::new("User@Example.com").unwrap();
        let email2 = EmailAddress::new("user@example.com").unwrap();
        assert_eq!(email1, email2);
    }

    #[test]
    fn equality_different_addresses() {
        let email1 = EmailAddress::new("user1@example.com").unwrap();
        let email2 = EmailAddress::new("user2@example.com").unwrap();
        assert_ne!(email1, email2);
    }

    // Display

    #[test]
    fn display_formats_correctly() {
        let email = EmailAddress::new("User@Example.com").unwrap();
        assert_eq!(format!("{}", email), "user@example.com");
    }

    // Error display

    #[test]
    fn error_display_messages() {
        assert!(EmailAddressError::MissingAt.to_string().contains("@"));
        assert!(
            EmailAddressError::MultipleAt
                .to_string()
                .contains("exactly one")
        );
        assert!(
            EmailAddressError::EmptyLocalPart
                .to_string()
                .contains("local part")
        );
        assert!(
            EmailAddressError::EmptyDomain
                .to_string()
                .contains("domain")
        );
        assert!(
            EmailAddressError::InvalidLocalPart
                .to_string()
                .contains("local part")
        );
        assert!(
            EmailAddressError::InvalidDomain
                .to_string()
                .contains("domain")
        );
    }
}
