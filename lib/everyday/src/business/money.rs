//! Money representation with currency support
//!
//! This module provides a type-safe, immutable representation of money
//! that enforces currency safety and prevents floating-point precision errors
//! by storing amounts in minor units (cents, pence, etc.).

use std::cmp::Ordering;
use std::fmt;

/// Supported currencies using ISO 4217 three-letter codes
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Currency {
    /// Euro (€) - Symbol after amount
    EUR,
    /// US Dollar ($) - Symbol before amount
    USD,
    /// Bosnia and Herzegovina Convertible Mark (KM) - Symbol before amount
    BAM,
    /// Serbian Dinar (дин.) - Symbol after amount
    RSD,
}

impl Currency {
    fn symbol(self) -> &'static str {
        match self {
            Currency::EUR => "€",
            Currency::USD => "$",
            Currency::BAM => "KM",
            Currency::RSD => "дин.",
        }
    }

    fn symbol_before(self) -> bool {
        matches!(self, Currency::USD | Currency::BAM)
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::EUR => write!(f, "EUR"),
            Currency::USD => write!(f, "USD"),
            Currency::BAM => write!(f, "BAM"),
            Currency::RSD => write!(f, "RSD"),
        }
    }
}

/// Errors that can occur during money operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    /// Operation attempted between different currencies
    CurrencyMismatch { left: Currency, right: Currency },
    /// Operation would exceed the maximum value (u32::MAX)
    Overflow,
    /// Operation would result in a negative value
    Underflow,
    /// Unsupported currency code
    InvalidCurrency(String),
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoneyError::CurrencyMismatch { left, right } => {
                write!(f, "currency mismatch: {} and {}", left, right)
            }
            MoneyError::Overflow => {
                write!(f, "operation would exceed maximum value (u32::MAX)")
            }
            MoneyError::Underflow => {
                write!(f, "operation would result in negative value")
            }
            MoneyError::InvalidCurrency(code) => {
                write!(f, "unsupported currency code: {}", code)
            }
        }
    }
}

impl std::error::Error for MoneyError {}

/// Immutable representation of money with currency support
///
/// Money is stored as an unsigned 32-bit integer representing minor units
/// (cents for USD/EUR, pence for GBP, etc.) along with a currency type.
/// This avoids floating-point precision errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    amount: u32,
    currency: Currency,
}

impl Money {
    pub fn from_minor_units(amount: u32, currency: Currency) -> Result<Self, MoneyError> {
        Ok(Money { amount, currency })
    }

    #[inline]
    pub const fn amount(&self) -> u32 {
        self.amount
    }

    #[inline]
    pub const fn currency(&self) -> Currency {
        self.currency
    }

    /// Returns error if currencies don't match
    pub fn add(&self, other: &Money) -> Result<Money, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }

        self.amount
            .checked_add(other.amount)
            .map(|amount| Money {
                amount,
                currency: self.currency,
            })
            .ok_or(MoneyError::Overflow)
    }

    /// Returns error if currencies don't match or result would be negative
    pub fn subtract(&self, other: &Money) -> Result<Money, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }

        self.amount
            .checked_sub(other.amount)
            .map(|amount| Money {
                amount,
                currency: self.currency,
            })
            .ok_or(MoneyError::Underflow)
    }

    /// Rounds up (ceiling) to avoid losing fractional amounts in displayed prices
    pub fn add_percent(&self, percent: u8) -> Result<Money, MoneyError> {
        let increase = (self.amount as f64 * percent as f64 / 100.0).ceil() as u32;

        self.amount
            .checked_add(increase)
            .map(|amount| Money {
                amount,
                currency: self.currency,
            })
            .ok_or(MoneyError::Overflow)
    }

    /// Rounds up (ceiling) to avoid losing fractional amounts in displayed prices
    pub fn subtract_percent(&self, percent: u8) -> Result<Money, MoneyError> {
        let result_f64 = self.amount as f64 * (100.0 - percent as f64) / 100.0;

        if result_f64 < 0.0 {
            return Err(MoneyError::Underflow);
        }

        let result = result_f64.ceil() as u32;

        Ok(Money {
            amount: result,
            currency: self.currency,
        })
    }

    /// Returns error if currencies don't match
    #[allow(clippy::should_implement_trait)]
    pub fn cmp(&self, other: &Money) -> Result<Ordering, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }
        Ok(self.amount.cmp(&other.amount))
    }
}

impl PartialOrd<Money> for Money {
    fn partial_cmp(&self, other: &Money) -> Option<Ordering> {
        if self.currency != other.currency {
            return None;
        }
        Some(self.amount.cmp(&other.amount))
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = self.currency.symbol();
        let symbol_before = self.currency.symbol_before();

        // Convert minor units to major units with 2 decimal places
        let major = self.amount / 100;
        let minor = self.amount % 100;

        if symbol_before {
            write!(f, "{}{}.{:02}", symbol, major, minor)
        } else {
            write!(f, "{}.{:02} {}", major, minor, symbol)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_money() {
        let money = Money::from_minor_units(1299, Currency::USD).unwrap();
        assert_eq!(money.amount(), 1299);
        assert_eq!(money.currency(), Currency::USD);
    }

    #[test]
    fn test_equality() {
        let m1 = Money::from_minor_units(100, Currency::USD).unwrap();
        let m2 = Money::from_minor_units(100, Currency::USD).unwrap();
        let m3 = Money::from_minor_units(200, Currency::USD).unwrap();

        assert_eq!(m1, m2);
        assert_ne!(m1, m3);
    }

    #[test]
    fn test_cmp_same_currency() {
        let m1 = Money::from_minor_units(100, Currency::USD).unwrap();
        let m2 = Money::from_minor_units(200, Currency::USD).unwrap();

        assert_eq!(m1.cmp(&m2).unwrap(), Ordering::Less);
        assert_eq!(m2.cmp(&m1).unwrap(), Ordering::Greater);
        assert_eq!(m1.cmp(&m1).unwrap(), Ordering::Equal);
    }

    #[test]
    fn test_cmp_different_currencies_returns_error() {
        let usd = Money::from_minor_units(100, Currency::USD).unwrap();
        let eur = Money::from_minor_units(100, Currency::EUR).unwrap();

        let result = usd.cmp(&eur);
        assert!(matches!(
            result,
            Err(MoneyError::CurrencyMismatch {
                left: Currency::USD,
                right: Currency::EUR
            })
        ));
    }

    #[test]
    fn test_partial_cmp_different_currencies() {
        let usd = Money::from_minor_units(100, Currency::USD).unwrap();
        let eur = Money::from_minor_units(100, Currency::EUR).unwrap();

        // partial_cmp returns None for different currencies
        assert!(usd.partial_cmp(&eur).is_none());
    }

    #[test]
    fn test_add_same_currency() {
        let m1 = Money::from_minor_units(100, Currency::USD).unwrap();
        let m2 = Money::from_minor_units(50, Currency::USD).unwrap();

        let sum = m1.add(&m2).unwrap();
        assert_eq!(sum.amount(), 150);
        assert_eq!(sum.currency(), Currency::USD);
    }

    #[test]
    fn test_add_different_currencies() {
        let usd = Money::from_minor_units(100, Currency::USD).unwrap();
        let eur = Money::from_minor_units(100, Currency::EUR).unwrap();

        let result = usd.add(&eur);
        assert!(matches!(
            result,
            Err(MoneyError::CurrencyMismatch {
                left: Currency::USD,
                right: Currency::EUR
            })
        ));
    }

    #[test]
    fn test_add_overflow() {
        let m1 = Money::from_minor_units(u32::MAX, Currency::USD).unwrap();
        let m2 = Money::from_minor_units(1, Currency::USD).unwrap();

        let result = m1.add(&m2);
        assert_eq!(result, Err(MoneyError::Overflow));
    }

    #[test]
    fn test_subtract_same_currency() {
        let m1 = Money::from_minor_units(100, Currency::USD).unwrap();
        let m2 = Money::from_minor_units(30, Currency::USD).unwrap();

        let diff = m1.subtract(&m2).unwrap();
        assert_eq!(diff.amount(), 70);
        assert_eq!(diff.currency(), Currency::USD);
    }

    #[test]
    fn test_subtract_to_zero() {
        let m1 = Money::from_minor_units(100, Currency::USD).unwrap();
        let m2 = Money::from_minor_units(100, Currency::USD).unwrap();

        let diff = m1.subtract(&m2).unwrap();
        assert_eq!(diff.amount(), 0);
    }

    #[test]
    fn test_subtract_different_currencies() {
        let usd = Money::from_minor_units(100, Currency::USD).unwrap();
        let eur = Money::from_minor_units(50, Currency::EUR).unwrap();

        let result = usd.subtract(&eur);
        assert!(matches!(
            result,
            Err(MoneyError::CurrencyMismatch {
                left: Currency::USD,
                right: Currency::EUR
            })
        ));
    }

    #[test]
    fn test_subtract_underflow() {
        let m1 = Money::from_minor_units(100, Currency::USD).unwrap();
        let m2 = Money::from_minor_units(200, Currency::USD).unwrap();

        let result = m1.subtract(&m2);
        assert_eq!(result, Err(MoneyError::Underflow));
    }

    #[test]
    fn test_add_percent_whole() {
        let m = Money::from_minor_units(100, Currency::USD).unwrap();
        let with_markup = m.add_percent(10).unwrap();
        assert_eq!(with_markup.amount(), 110);
    }

    #[test]
    fn test_add_percent_with_rounding() {
        let m = Money::from_minor_units(333, Currency::USD).unwrap();
        let with_markup = m.add_percent(10).unwrap();
        // 333 + 33.3 = 366.3, rounded up to 367
        assert_eq!(with_markup.amount(), 367);
    }

    #[test]
    fn test_add_percent_zero() {
        let m = Money::from_minor_units(100, Currency::USD).unwrap();
        let result = m.add_percent(0).unwrap();
        assert_eq!(result.amount(), 100);
    }

    #[test]
    fn test_add_percent_overflow() {
        let m = Money::from_minor_units(u32::MAX, Currency::USD).unwrap();
        let result = m.add_percent(1);
        assert_eq!(result, Err(MoneyError::Overflow));
    }

    #[test]
    fn test_subtract_percent_whole() {
        let m = Money::from_minor_units(100, Currency::USD).unwrap();
        let discounted = m.subtract_percent(10).unwrap();
        assert_eq!(discounted.amount(), 90);
    }

    #[test]
    fn test_subtract_percent_with_rounding() {
        let m = Money::from_minor_units(333, Currency::USD).unwrap();
        let discounted = m.subtract_percent(10).unwrap();
        // 333 - 33.3 = 299.7, rounded up to 300
        assert_eq!(discounted.amount(), 300);
    }

    #[test]
    fn test_subtract_percent_zero() {
        let m = Money::from_minor_units(100, Currency::USD).unwrap();
        let result = m.subtract_percent(0).unwrap();
        assert_eq!(result.amount(), 100);
    }

    #[test]
    fn test_subtract_percent_underflow() {
        let m = Money::from_minor_units(100, Currency::USD).unwrap();
        let result = m.subtract_percent(101);
        assert_eq!(result, Err(MoneyError::Underflow));
    }

    #[test]
    fn test_display_usd() {
        let m = Money::from_minor_units(1299, Currency::USD).unwrap();
        assert_eq!(m.to_string(), "$12.99");
    }

    #[test]
    fn test_display_eur() {
        let m = Money::from_minor_units(1299, Currency::EUR).unwrap();
        assert_eq!(m.to_string(), "12.99 €");
    }

    #[test]
    fn test_display_bam() {
        let m = Money::from_minor_units(1299, Currency::BAM).unwrap();
        assert_eq!(m.to_string(), "KM12.99");
    }

    #[test]
    fn test_display_rsd() {
        let m = Money::from_minor_units(1299, Currency::RSD).unwrap();
        assert_eq!(m.to_string(), "12.99 дин.");
    }

    #[test]
    fn test_display_zero() {
        let m = Money::from_minor_units(0, Currency::USD).unwrap();
        assert_eq!(m.to_string(), "$0.00");
    }

    #[test]
    fn test_display_large_amount() {
        let m = Money::from_minor_units(4294967295, Currency::USD).unwrap();
        assert_eq!(m.to_string(), "$42949672.95");
    }

    #[test]
    fn test_all_currencies_supported() {
        let currencies = [Currency::EUR, Currency::USD, Currency::BAM, Currency::RSD];

        for currency in currencies {
            let money = Money::from_minor_units(1000, currency).unwrap();
            assert_eq!(money.currency(), currency);
        }
    }

    #[test]
    fn test_immutability() {
        let m1 = Money::from_minor_units(100, Currency::USD).unwrap();
        let m2 = m1
            .add(&Money::from_minor_units(50, Currency::USD).unwrap())
            .unwrap();

        // Original should be unchanged
        assert_eq!(m1.amount(), 100);
        assert_eq!(m2.amount(), 150);
    }

    #[test]
    fn test_currency_mismatch_error_contains_both_currencies() {
        let usd = Money::from_minor_units(100, Currency::USD).unwrap();
        let eur = Money::from_minor_units(100, Currency::EUR).unwrap();

        let result = usd.add(&eur);
        match result {
            Err(MoneyError::CurrencyMismatch { left, right }) => {
                assert_eq!(left, Currency::USD);
                assert_eq!(right, Currency::EUR);
            }
            _ => panic!("Expected CurrencyMismatch error"),
        }
    }
}
