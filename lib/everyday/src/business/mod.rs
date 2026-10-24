//! Business-related types.
//!
//! This module provides types for business concepts such as currency,
//! money, contact information, and associated error types.

mod email_address;
mod error;
mod money;
mod phone_number;
mod phone_number_parser;
mod url;
mod working_hours;

pub mod prelude {
    //! Commonly used types from the business module.

    pub use super::email_address::{EmailAddress, EmailAddressError};
    pub use super::error::Error;
    pub use super::money::{Currency, Money, MoneyError};
    pub use super::phone_number::{PhoneNumber, PhoneNumberError};
    pub use super::phone_number_parser::prelude::*;
    pub use super::url::{Url, UrlError};
    pub use super::working_hours::{
        DayWorkingHours, WeeklyWorkingHours, WorkingHours, WorkingTime,
    };
}
